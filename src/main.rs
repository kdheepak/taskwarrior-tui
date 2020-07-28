#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

mod util;

#[allow(dead_code)]
mod app;

use crate::util::{destruct_terminal, setup_terminal, Event, EventConfig, Events};
use std::env;
use std::error::Error;
use std::io;
use std::io::{stdout, Write};
use std::process::Command;
use std::time::{Duration, Instant};
use tui::backend::Backend;
use unicode_width::UnicodeWidthStr;

use app::{App, AppMode};
use crate::util::Key;

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

const APP_VERSION: &'static str = env!("CARGO_PKG_VERSION");
const APP_NAME: &'static str = env!("CARGO_PKG_NAME");


fn main() -> Result<(), Box<dyn Error>> {
    // Terminal initialization
    let mut terminal = setup_terminal();
    terminal.clear()?;

    // Setup event handlers
    let events = Events::with_config(EventConfig {
        tick_rate: Duration::from_millis(250),
    });

    let mut app = App::new();
    app.next();

    loop {
        terminal.draw(|mut frame| app.draw(&mut frame)).unwrap();

        // Handle input
        match events.next()? {
            Event::Input(input) => {
                    match app.mode {
                        AppMode::Report => match input {
                            Key::Ctrl('c') | Key::Char('q') => app.should_quit = true,
                            Key::Char('r') => app.update(),
                            Key::Down | Key::Char('j') => app.next(),
                            Key::Up | Key::Char('k') => app.previous(),
                            Key::Char('d') => app.task_done(),
                            Key::Char('x') => app.task_delete(),
                            Key::Char('s') => app.task_start_or_stop(),
                            Key::Char('u') => app.task_undo(),
                            Key::Char('e') => {
                                events.pause_event_loop(&mut terminal);
                                app.task_edit();
                                events.resume_event_loop(&mut terminal);
                            },
                            Key::Char('m') => {
                                app.mode = AppMode::ModifyTask;
                                match app.task_current() {
                                    Some(t) => app.modify = t.description().to_string(),
                                    None => app.modify = "".to_string(),
                                }
                            }
                            Key::Char('l') => {
                                app.mode = AppMode::LogTask;
                            }
                            Key::Char('a') => {
                                app.mode = AppMode::AddTask;
                            }
                            Key::Char('/') => {
                                app.mode = AppMode::Filter;
                            }
                            _ => {}
                        },
                        AppMode::ModifyTask => match input {
                            Key::Char('\n') => {
                                app.task_modify();
                                app.mode = AppMode::Report;
                            }
                            Key::Esc => {
                                app.modify = "".to_string();
                                app.mode = AppMode::Report;
                            }
                            Key::Char(c) => {
                                app.modify.push(c);
                            }
                            Key::Backspace => {
                                app.modify.pop();
                            }
                            _ => {}
                        },
                        AppMode::LogTask => match input {
                            Key::Char('\n') => {
                                app.task_log();
                                app.mode = AppMode::Report;
                            }
                            Key::Esc => {
                                app.command = "".to_string();
                                app.mode = AppMode::Report;
                            }
                            Key::Char(c) => {
                                app.command.push(c);
                            }
                            Key::Backspace => {
                                app.command.pop();
                            }
                            _ => {}
                        },
                        AppMode::AddTask => match input {
                            Key::Char('\n') => {
                                app.task_add();
                                app.mode = AppMode::Report;
                            }
                            Key::Esc => {
                                app.command = "".to_string();
                                app.mode = AppMode::Report;
                            }
                            Key::Char(c) => {
                                app.command.push(c);
                            }
                            Key::Backspace => {
                                app.command.pop();
                            }
                            _ => {}
                        },
                        AppMode::Filter => match input {
                            Key::Char('\n') | Key::Esc => {
                                app.mode = AppMode::Report;
                            }
                            Key::Char(c) => {
                                app.filter.push(c);
                            }
                            Key::Backspace => {
                                app.filter.pop();
                            }
                            _ => {}
                        },
                    }
                }
            Event::Tick => app.handle_tick(),
        }

        if app.should_quit {
            destruct_terminal(terminal);
            break;
        }
    }
    Ok(())
}
