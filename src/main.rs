#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

mod app;
mod calendar;
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
                AppMode::TaskReport => match input {
                    Key::Ctrl('c') | Key::Char('q') => app.should_quit = true,
                    Key::Char(']') => {
                        app.mode = AppMode::Calendar;
                    }
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
                        app.mode = AppMode::TaskModify;
                        match app.task_current() {
                            Some(t) => app.modify = t.description().to_string(),
                            None => app.modify = "".to_string(),
                        }
                        app.cursor_location = app.modify.chars().count();
                    }
                    Key::Char('l') => {
                        app.mode = AppMode::TaskLog;
                    }
                    Key::Char('a') => {
                        app.mode = AppMode::TaskAdd;
                        app.cursor_location = app.command.chars().count();
                    }
                    Key::Char('A') => {
                        app.mode = AppMode::TaskAnnotate;
                        app.cursor_location = app.command.chars().count();
                    }
                    Key::Char('?') => {
                        app.mode = AppMode::TaskHelpPopup;
                    }
                    Key::Char('/') => {
                        app.mode = AppMode::TaskFilter;
                        app.cursor_location = app.filter.chars().count();
                    }
                    _ => {}
                },
                AppMode::TaskHelpPopup => match input {
                    Key::Esc => {
                        app.mode = AppMode::TaskReport;
                    }
                    _ => {}
                },
                AppMode::TaskModify => match input {
                    Key::Char('\n') => match app.task_modify() {
                        Ok(_) => {
                            app.mode = AppMode::TaskReport;
                            app.update();
                        }
                        Err(e) => {
                            app.mode = AppMode::TaskError;
                            app.error = e;
                        }
                    },
                    Key::Esc => {
                        app.modify = "".to_string();
                        app.mode = AppMode::TaskReport;
                    }
                    Key::Right => {
                        if app.cursor_location < app.modify.chars().count() {
                            app.cursor_location += 1;
                        }
                    }
                    Key::Left => {
                        if app.cursor_location > 0 {
                            app.cursor_location -= 1;
                        }
                    }
                    Key::Char(c) => {
                        if app.cursor_location < app.modify.chars().count() {
                            app.modify.insert_str(app.cursor_location, &c.to_string());
                        } else {
                            app.modify.push(c);
                        }
                        app.cursor_location += 1;
                    }
                    Key::Backspace => {
                        if app.cursor_location > 0 {
                            app.cursor_location -= 1;
                            let mut cs = app.modify.chars().collect::<Vec<char>>();
                            cs.remove(app.cursor_location);
                            app.modify = cs.into_iter().collect();
                        }
                    }
                    _ => {}
                },
                AppMode::TaskLog => match input {
                    Key::Char('\n') => match app.task_log() {
                        Ok(_) => {
                            app.mode = AppMode::TaskReport;
                            app.update();
                        }
                        Err(e) => {
                            app.mode = AppMode::TaskError;
                            app.error = e;
                        }
                    },
                    Key::Esc => {
                        app.command = "".to_string();
                        app.mode = AppMode::TaskReport;
                    }
                    Key::Right => {
                        if app.cursor_location < app.command.chars().count() {
                            app.cursor_location += 1;
                        }
                    }
                    Key::Left => {
                        if app.cursor_location > 0 {
                            app.cursor_location -= 1;
                        }
                    }
                    Key::Char(c) => {
                        if app.cursor_location < app.command.chars().count() {
                            app.command.insert_str(app.cursor_location, &c.to_string());
                        } else {
                            app.command.push(c);
                        }
                        app.cursor_location += 1;
                    }
                    Key::Backspace => {
                        if app.cursor_location > 0 {
                            app.cursor_location -= 1;
                            let mut cs = app.command.chars().collect::<Vec<char>>();
                            cs.remove(app.cursor_location);
                            app.command = cs.into_iter().collect();
                        }
                    }
                    _ => {}
                },
                AppMode::TaskAnnotate => match input {
                    Key::Char('\n') => match app.task_annotate() {
                        Ok(_) => {
                            app.mode = AppMode::TaskReport;
                            app.update();
                        }
                        Err(e) => {
                            app.mode = AppMode::TaskError;
                            app.error = e;
                        }
                    },
                    Key::Esc => {
                        app.command = "".to_string();
                        app.mode = AppMode::TaskReport;
                    }
                    Key::Right => {
                        if app.cursor_location < app.command.chars().count() {
                            app.cursor_location += 1;
                        }
                    }
                    Key::Left => {
                        if app.cursor_location > 0 {
                            app.cursor_location -= 1;
                        }
                    }
                    Key::Char(c) => {
                        if app.cursor_location < app.command.chars().count() {
                            app.command.insert_str(app.cursor_location, &c.to_string());
                        } else {
                            app.command.push(c);
                        }
                        app.cursor_location += 1;
                    }
                    Key::Backspace => {
                        if app.cursor_location > 0 {
                            app.cursor_location -= 1;
                            let mut cs = app.command.chars().collect::<Vec<char>>();
                            cs.remove(app.cursor_location);
                            app.command = cs.into_iter().collect();
                        }
                    }
                    _ => {}
                },
                AppMode::TaskAdd => match input {
                    Key::Char('\n') => match app.task_add() {
                        Ok(_) => {
                            app.mode = AppMode::TaskReport;
                            app.update();
                        }
                        Err(e) => {
                            app.mode = AppMode::TaskError;
                            app.error = e;
                        }
                    },
                    Key::Esc => {
                        app.command = "".to_string();
                        app.mode = AppMode::TaskReport;
                    }
                    Key::Right => {
                        if app.cursor_location < app.command.chars().count() {
                            app.cursor_location += 1;
                        }
                    }
                    Key::Left => {
                        if app.cursor_location > 0 {
                            app.cursor_location -= 1;
                        }
                    }
                    Key::Char(c) => {
                        if app.cursor_location < app.command.chars().count() {
                            app.command.insert_str(app.cursor_location, &c.to_string());
                        } else {
                            app.command.push(c);
                        }
                        app.cursor_location += 1;
                    }
                    Key::Backspace => {
                        if app.cursor_location > 0 {
                            app.cursor_location -= 1;
                            let mut cs = app.command.chars().collect::<Vec<char>>();
                            cs.remove(app.cursor_location);
                            app.command = cs.into_iter().collect();
                        }
                    }
                    _ => {}
                },
                AppMode::TaskFilter => match input {
                    Key::Char('\n') | Key::Esc => {
                        app.mode = AppMode::TaskReport;
                        app.update();
                    }
                    Key::Right => {
                        if app.cursor_location < app.filter.chars().count() {
                            app.cursor_location += 1;
                        }
                    }
                    Key::Left => {
                        if app.cursor_location > 0 {
                            app.cursor_location -= 1;
                        }
                    }
                    Key::Char(c) => {
                        if app.cursor_location < app.filter.chars().count() {
                            app.filter.insert_str(app.cursor_location, &c.to_string());
                        } else {
                            app.filter.push(c);
                        }
                        app.cursor_location += 1;
                    }
                    Key::Backspace => {
                        if app.cursor_location > 0 {
                            app.cursor_location -= 1;
                            let mut cs = app.filter.chars().collect::<Vec<char>>();
                            cs.remove(app.cursor_location);
                            app.filter = cs.into_iter().collect();
                        }
                    }
                    _ => {}
                },
                AppMode::TaskError => match input {
                    _ => {
                        app.mode = AppMode::TaskReport;
                    }
                },
                AppMode::Calendar => match input {
                    Key::Char('[') => {
                        app.mode = AppMode::TaskReport;
                    }
                    Key::Ctrl('c') | Key::Char('q') => app.should_quit = true,
                    _ => {}
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
