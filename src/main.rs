#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

mod app;
mod color;
mod util;

use crate::util::{destruct_terminal, setup_terminal, Event, EventConfig, Events};
use clap::{App, Arg};
use std::env;
use std::error::Error;
use std::time::Duration;

use crate::util::Key;
use app::{AppMode, TTApp};

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const APP_NAME: &str = env!("CARGO_PKG_NAME");

fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new(APP_NAME)
        .version(APP_VERSION)
        .author("Dheepak Krishnamurthy <@kdheepak>")
        .about("A taskwarrior terminal user interface")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file")
                .takes_value(true),
        )
        .get_matches();

    let config = matches.value_of("config").unwrap_or("default.conf");
    tui_main(config)?;
    Ok(())
}

fn tui_main(_config: &str) -> Result<(), Box<dyn Error>> {
    // Terminal initialization
    let mut terminal = setup_terminal();
    terminal.clear()?;

    // Setup event handlers
    let events = Events::with_config(EventConfig {
        tick_rate: Duration::from_millis(1000),
    });

    let mut app = TTApp::new();
    app.next();

    loop {
        terminal.draw(|mut frame| app.draw(&mut frame)).unwrap();

        // Handle input
        match events.next()? {
            Event::Input(input) => match app.mode {
                AppMode::Report => match input {
                    Key::Ctrl('c') | Key::Char('q') => app.should_quit = true,
                    Key::Char('r') => app.update(),
                    Key::Down | Key::Char('j') => app.next(),
                    Key::Up | Key::Char('k') => app.previous(),
                    Key::Char('d') => match app.task_done() {
                        Ok(_) => app.update(),
                        Err(e) => {
                            app.mode = AppMode::TaskError;
                            app.error = e;
                        }
                    },
                    Key::Char('x') => match app.task_delete() {
                        Ok(_) => app.update(),
                        Err(e) => {
                            app.mode = AppMode::TaskError;
                            app.error = e;
                        }
                    },
                    Key::Char('s') => match app.task_start_or_stop() {
                        Ok(_) => app.update(),
                        Err(e) => {
                            app.mode = AppMode::TaskError;
                            app.error = e;
                        }
                    },
                    Key::Char('u') => match app.task_undo() {
                        Ok(_) => app.update(),
                        Err(e) => {
                            app.mode = AppMode::TaskError;
                            app.error = e;
                        }
                    },
                    Key::Char('e') => {
                        events.pause_event_loop(&mut terminal);
                        let r = app.task_edit();
                        events.resume_event_loop(&mut terminal);
                        match r {
                            Ok(_) => app.update(),
                            Err(e) => {
                                app.mode = AppMode::TaskError;
                                app.error = e;
                            }
                        }
                    }
                    Key::Char('m') => {
                        app.mode = AppMode::ModifyTask;
                        match app.task_current() {
                            Some(t) => app.modify = t.description().to_string(),
                            None => app.modify = "".to_string(),
                        }
                        app.cursor_location = app.modify.len();
                    }
                    Key::Char('l') => {
                        app.mode = AppMode::LogTask;
                    }
                    Key::Char('a') => {
                        app.mode = AppMode::AddTask;
                        app.cursor_location = app.command.len();
                    }
                    Key::Char('?') => {
                        app.mode = AppMode::HelpPopup;
                    }
                    Key::Char('/') => {
                        app.mode = AppMode::Filter;
                        app.cursor_location = app.filter.len();
                    }
                    _ => {}
                },
                AppMode::HelpPopup => match input {
                    Key::Esc => {
                        app.mode = AppMode::Report;
                    }
                    _ => {}
                },
                AppMode::ModifyTask => match input {
                    Key::Char('\n') => match app.task_modify() {
                        Ok(_) => {
                            app.mode = AppMode::Report;
                            app.update();
                        }
                        Err(e) => {
                            app.mode = AppMode::TaskError;
                            app.error = e;
                        }
                    },
                    Key::Esc => {
                        app.modify = "".to_string();
                        app.mode = AppMode::Report;
                    }
                    Key::Right => {
                        if app.cursor_location < app.modify.len() {
                            app.cursor_location += 1;
                        }
                    }
                    Key::Left => {
                        if app.cursor_location > 0 {
                            app.cursor_location -= 1;
                        }
                    }
                    Key::Char(c) => {
                        if app.cursor_location < app.modify.len() {
                            app.modify.insert(app.cursor_location, c);
                        } else {
                            app.modify.push(c);
                        }
                        app.cursor_location += 1;
                    }
                    Key::Backspace => {
                        if app.cursor_location > 0 {
                            app.cursor_location -= 1;
                            app.modify.remove(app.cursor_location);
                        }
                    }
                    _ => {}
                },
                AppMode::LogTask => match input {
                    Key::Char('\n') => match app.task_log() {
                        Ok(_) => {
                            app.mode = AppMode::Report;
                            app.update();
                        }
                        Err(e) => {
                            app.mode = AppMode::TaskError;
                            app.error = e;
                        }
                    },
                    Key::Esc => {
                        app.command = "".to_string();
                        app.mode = AppMode::Report;
                    }
                    Key::Right => {
                        if app.cursor_location < app.command.len() {
                            app.cursor_location += 1;
                        }
                    }
                    Key::Left => {
                        if app.cursor_location > 0 {
                            app.cursor_location -= 1;
                        }
                    }
                    Key::Char(c) => {
                        if app.cursor_location < app.command.len() {
                            app.command.insert(app.cursor_location, c);
                        } else {
                            app.command.push(c);
                        }
                        app.cursor_location += 1;
                    }
                    Key::Backspace => {
                        if app.cursor_location > 0 {
                            app.cursor_location -= 1;
                            app.command.remove(app.cursor_location);
                        }
                    }
                    _ => {}
                },
                AppMode::AddTask => match input {
                    Key::Char('\n') => match app.task_add() {
                        Ok(_) => {
                            app.mode = AppMode::Report;
                            app.update();
                        }
                        Err(e) => {
                            app.mode = AppMode::TaskError;
                            app.error = e;
                        }
                    },
                    Key::Esc => {
                        app.command = "".to_string();
                        app.mode = AppMode::Report;
                    }
                    Key::Right => {
                        if app.cursor_location < app.command.len() {
                            app.cursor_location += 1;
                        }
                    }
                    Key::Left => {
                        if app.cursor_location > 0 {
                            app.cursor_location -= 1;
                        }
                    }
                    Key::Char(c) => {
                        if app.cursor_location < app.command.len() {
                            app.command.insert(app.cursor_location, c);
                        } else {
                            app.command.push(c);
                        }
                        app.cursor_location += 1;
                    }
                    Key::Backspace => {
                        if app.cursor_location > 0 {
                            app.cursor_location -= 1;
                            app.command.remove(app.cursor_location);
                        }
                    }
                    _ => {}
                },
                AppMode::Filter => match input {
                    Key::Char('\n') | Key::Esc => {
                        app.mode = AppMode::Report;
                        app.update();
                    }
                    Key::Right => {
                        if app.cursor_location < app.filter.len() {
                            app.cursor_location += 1;
                        }
                    }
                    Key::Left => {
                        if app.cursor_location > 0 {
                            app.cursor_location -= 1;
                        }
                    }
                    Key::Char(c) => {
                        if app.cursor_location < app.filter.len() {
                            app.filter.insert(app.cursor_location, c);
                        } else {
                            app.filter.push(c);
                        }
                        app.cursor_location += 1;
                    }
                    Key::Backspace => {
                        if app.cursor_location > 0 {
                            app.cursor_location -= 1;
                            app.filter.remove(app.cursor_location);
                        }
                    }
                    _ => {}
                },
                AppMode::TaskError => match input {
                    _ => {
                        app.mode = AppMode::Report;
                    }
                },
            },
            Event::Tick => app.update(),
        }

        if app.should_quit {
            destruct_terminal(terminal);
            break;
        }
    }
    Ok(())
}
