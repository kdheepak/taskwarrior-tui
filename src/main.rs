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
mod scrollbar;
mod table;
mod task_report;

use log::{debug, error, info, log_enabled, trace, warn, Level, LevelFilter};
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Logger, Root};
use log4rs::encode::pattern::PatternEncoder;
use std::env;
use std::error::Error;
use std::io::{self, Write};
use std::panic;
use std::path::{Path, PathBuf};
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

use path_clean::PathClean;

use app::{Mode, TaskwarriorTui};

use crate::action::Action;
use crate::event::{Event, EventConfig, Events, Key};
use crate::keyconfig::KeyConfig;

const LOG_PATTERN: &str = "{d(%Y-%m-%d %H:%M:%S)} | {l} | {f}:{L} | {m}{n}";

pub fn setup_terminal() -> Terminal<CrosstermBackend<io::Stdout>> {
    enable_raw_mode().expect("Running not in terminal");
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

pub fn initialize_logging() {
    let data_local_dir = if let Ok(s) = std::env::var("TASKWARRIOR_TUI_DATA") {
        PathBuf::from(s)
    } else {
        dirs::data_local_dir()
            .expect("Unable to find data directory for taskwarrior-tui")
            .join("taskwarrior-tui")
    };

    std::fs::create_dir_all(&data_local_dir).unwrap_or_else(|_| panic!("Unable to create {:?}", data_local_dir));

    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(LOG_PATTERN)))
        .append(false)
        .build(data_local_dir.join("taskwarrior-tui.log"))
        .expect("Failed to build log file appender.");

    let levelfilter = match std::env::var("TASKWARRIOR_TUI_LOG_LEVEL")
        .unwrap_or_else(|_| "info".to_string())
        .as_str()
    {
        "off" => LevelFilter::Off,
        "warn" => LevelFilter::Warn,
        "info" => LevelFilter::Info,
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        _ => LevelFilter::Info,
    };
    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .logger(Logger::builder().build("taskwarrior_tui", levelfilter))
        .build(Root::builder().appender("logfile").build(LevelFilter::Info))
        .expect("Failed to build logging config.");

    log4rs::init_config(config).expect("Failed to initialize logging.");
}

pub fn absolute_path(path: impl AsRef<Path>) -> io::Result<PathBuf> {
    let path = path.as_ref();

    let absolute_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir()?.join(path)
    }
    .clean();

    Ok(absolute_path)
}

fn main() {
    better_panic::install();

    let matches = cli::generate_cli_app().get_matches();

    let config = matches.value_of("config");
    let data = matches.value_of("data");
    let taskrc = matches.value_of("taskrc");
    let taskdata = matches.value_of("taskdata");
    let report = matches.value_of("report").unwrap_or("next");

    if let Some(e) = config {
        if env::var("TASKWARRIOR_TUI_CONFIG").is_err() {
            // if environment variable is not set, this env::var returns an error
            env::set_var(
                "TASKWARRIOR_TUI_CONFIG",
                absolute_path(PathBuf::from(e)).expect("Unable to get path for config"),
            )
        } else {
            warn!("TASKWARRIOR_TUI_CONFIG environment variable cannot be set.")
        }
    }

    if let Some(e) = data {
        if env::var("TASKWARRIOR_TUI_DATA").is_err() {
            // if environment variable is not set, this env::var returns an error
            env::set_var(
                "TASKWARRIOR_TUI_DATA",
                absolute_path(PathBuf::from(e)).expect("Unable to get path for data"),
            )
        } else {
            warn!("TASKWARRIOR_TUI_DATA environment variable cannot be set.")
        }
    }

    if let Some(e) = taskrc {
        if env::var("TASKRC").is_err() {
            // if environment variable is not set, this env::var returns an error
            env::set_var(
                "TASKRC",
                absolute_path(PathBuf::from(e)).expect("Unable to get path for taskrc"),
            )
        } else {
            warn!("TASKRC environment variable cannot be set.")
        }
    }

    if let Some(e) = taskdata {
        if env::var("TASKDATA").is_err() {
            // if environment variable is not set, this env::var returns an error
            env::set_var(
                "TASKDATA",
                absolute_path(PathBuf::from(e)).expect("Unable to get path for taskdata"),
            )
        } else {
            warn!("TASKDATA environment variable cannot be set.")
        }
    }

    initialize_logging();

    debug!("getting matches from clap...");
    debug!("report = {:?}", &report);
    debug!("config = {:?}", &config);
    let r = task::block_on(tui_main(report));
    if let Err(err) = r {
        eprintln!("\x1b[0;31m[taskwarrior-tui error]\x1b[0m: {}\n\nIf you need additional help, please report as a github issue on https://github.com/kdheepak/taskwarrior-tui", err);
        std::process::exit(1);
    }
}

async fn tui_main(report: &str) -> Result<()> {
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
    let events = if app.config.uda_tick_rate == 0 {
        Events::with_config(EventConfig {
            tick_rate: Duration::from_millis(100 * 60 * 60 * 24 * 30 * 12),
        })
    } else {
        Events::with_config(EventConfig {
            tick_rate: Duration::from_millis(app.config.uda_tick_rate),
        })
    };

    loop {
        app.render(&mut terminal).unwrap();
        // Handle input
        match events.next().await? {
            Event::Input(input) => {
                debug!("Received input = {:?}", input);
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
                {
                    Events::enter_tui_mode(&mut terminal);
                }
                if r.is_err() {
                    destruct_terminal();
                    return r;
                }
            }
            Event::Tick => {
                trace!("Tick event");
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
