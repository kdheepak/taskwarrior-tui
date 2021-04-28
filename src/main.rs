#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

mod app;
mod calendar;
mod completion;
mod config;
mod context;
mod event;
mod help;
mod history;
mod keyconfig;
mod table;
mod task_report;

use crate::event::{Event, EventConfig, Events, Key};
use anyhow::Result;
use clap::{App, Arg};
use std::env;
use std::error::Error;
use std::io::{self, Write};
use std::panic;
use std::time::Duration;

use async_std::prelude::*;
use async_std::sync::{Arc, Mutex};
use async_std::task;
use futures::stream::{FuturesUnordered, StreamExt};

use crossterm::{
    cursor,
    event::{DisableMouseCapture, EnableMouseCapture, EventStream},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{backend::CrosstermBackend, Terminal};

use app::{AppMode, TaskwarriorTuiApp};

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const APP_NAME: &str = env!("CARGO_PKG_NAME");

pub fn setup_terminal() -> Terminal<CrosstermBackend<io::Stdout>> {
    enable_raw_mode().unwrap();
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).unwrap();
    execute!(stdout, Clear(ClearType::All)).unwrap();
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend).unwrap()
}

pub fn destruct_terminal() {
    disable_raw_mode().unwrap();
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture).unwrap();
    execute!(io::stdout(), cursor::Show).unwrap();
}

fn main() {
    better_panic::install();
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
        .arg(
            Arg::with_name("report")
                .short("r")
                .long("report")
                .value_name("STRING")
                .help("Sets default report")
                .takes_value(true),
        )
        .get_matches();

    let config = matches.value_of("config").unwrap_or("~/.taskrc");
    let report = matches.value_of("report").unwrap_or("next");
    task::block_on(tui_main(config, report)).expect(
        "[taskwarrior-tui error].  Please report as a github issue on https://github.com/kdheepak/taskwarrior-tui",
    );
}

async fn tui_main(_config: &str, report: &str) -> Result<()> {
    panic::set_hook(Box::new(|panic_info| {
        destruct_terminal();
        better_panic::Settings::auto().create_panic_handler()(panic_info);
    }));

    let maybeapp = TaskwarriorTuiApp::new(report);
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
                if input == app.keyconfig.edit && app.mode == AppMode::TaskReport {
                    events.leave_tui_mode(&mut terminal);
                }

                let r = app.handle_input(input);

                if input == app.keyconfig.edit && app.mode == AppMode::TaskReport {
                    events.enter_tui_mode(&mut terminal);
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
