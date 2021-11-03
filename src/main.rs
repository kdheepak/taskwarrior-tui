#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

mod action;
mod app;
mod calendar;
mod cli;
mod completion;
mod config;
mod event;
mod help;
mod history;
mod keyconfig;
mod pane;
mod table;
mod task_report;

use std::env;
use std::error::Error;
use std::io::{self, Write};
use std::panic;
use std::time::Duration;

use anyhow::Result;
use async_std::prelude::*;
use async_std::sync::{Arc, Mutex};
use async_std::task;
use crossterm::{
    cursor,
    event::{DisableMouseCapture, EnableMouseCapture, EventStream},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::stream::{FuturesUnordered, StreamExt};
use tui::{backend::CrosstermBackend, Terminal};

use app::{Mode, TaskwarriorTui};

use crate::action::Action;
use crate::event::{Event, EventConfig, Events, Key};
use crate::keyconfig::KeyConfig;

/// # Panics
/// Will panic if could not obtain terminal
pub fn setup_terminal() -> Terminal<CrosstermBackend<io::Stdout>> {
    enable_raw_mode().expect("Running not in terminal");
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).unwrap();
    execute!(stdout, Clear(ClearType::All)).unwrap();
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend).unwrap()
}
/// # Panics
/// Will panic if could not `disable_raw_mode`
pub fn destruct_terminal() {
    disable_raw_mode().unwrap();
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture).unwrap();
    execute!(io::stdout(), cursor::Show).unwrap();
}

fn main() {
    better_panic::install();
    let matches = cli::generate_cli_app().get_matches();

    let config = matches.value_of("config").unwrap_or("~/.taskrc");
    let report = matches.value_of("report").unwrap_or("next");
    let r = task::block_on(tui_main(config, report));
    if let Err(err) = r {
        eprintln!("\x1b[0;31m[taskwarrior-tui error]\x1b[0m: {}\n\nIf you need additional help, please report as a github issue on https://github.com/kdheepak/taskwarrior-tui", err);
        std::process::exit(1);
    }
}

async fn tui_main(_config: &str, report: &str) -> Result<()> {
    panic::set_hook(Box::new(|panic_info| {
        destruct_terminal();
        better_panic::Settings::auto().create_panic_handler()(panic_info);
    }));

    let maybeapp = TaskwarriorTui::new(report);
    if maybeapp.is_err() {
        destruct_terminal();
        return Err(maybeapp.err().unwrap());
    }

    let mut app = maybeapp.unwrap();
    let mut terminal = setup_terminal();

    app.render(&mut terminal).unwrap();

    // Setup event handlers
    let events = Events::with_config(EventConfig {
        tick_rate: Duration::from_millis(app.config.uda_tick_rate),
    });

    loop {
        app.render(&mut terminal).unwrap();
        // Handle input
        match events.next().await? {
            Event::Input(input) => {
                if (input == app.keyconfig.edit
                    || input == app.keyconfig.shortcut1
                    || input == app.keyconfig.shortcut2
                    || input == app.keyconfig.shortcut3
                    || input == app.keyconfig.shortcut4
                    || input == app.keyconfig.shortcut5
                    || input == app.keyconfig.shortcut6
                    || input == app.keyconfig.shortcut7
                    || input == app.keyconfig.shortcut8
                    || input == app.keyconfig.shortcut9)
                    && app.mode == Mode::Tasks(Action::Report)
                {
                    Events::leave_tui_mode(&mut terminal);
                }

                let r = app.handle_input(input);

                if (input == app.keyconfig.edit
                    || input == app.keyconfig.shortcut1
                    || input == app.keyconfig.shortcut2
                    || input == app.keyconfig.shortcut3
                    || input == app.keyconfig.shortcut4
                    || input == app.keyconfig.shortcut5
                    || input == app.keyconfig.shortcut6
                    || input == app.keyconfig.shortcut7
                    || input == app.keyconfig.shortcut8
                    || input == app.keyconfig.shortcut9)
                    && app.mode == Mode::Tasks(Action::Report)
                    || app.mode == Mode::Tasks(Action::Error)
                {
                    Events::enter_tui_mode(&mut terminal);
                }
                if r.is_err() {
                    destruct_terminal();
                    return r;
                }
            }
            Event::Tick => {
                let r = app.update(false);
                if r.is_err() {
                    destruct_terminal();
                    return r;
                }
            }
        }

        if app.should_quit {
            destruct_terminal();
            break;
        }
    }
    Ok(())
}
