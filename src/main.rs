#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(clippy::too_many_arguments)]

mod action;
mod app;
mod calendar;
mod cli;
mod completion;
mod config;
mod help;
mod history;
mod keyconfig;
mod keyevent;
mod keymap;
mod pane;
mod scrollbar;
mod table;
mod task_report;
mod traits;
mod tui;
mod ui;
mod utils;

use std::{
  env,
  error::Error,
  io::{self, Write},
  panic,
  path::{Path, PathBuf},
  time::Duration,
};

// use app::{Mode, TaskwarriorTui};
use color_eyre::eyre::Result;
use utils::{absolute_path, get_config_dir, get_data_dir, initialize_logging, initialize_panic_handler};
// use crossterm::{
//   cursor,
//   event::{DisableMouseCapture, EnableMouseCapture, EventStream},
//   execute,
//   terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
// };
// use futures::stream::{FuturesUnordered, StreamExt};
// use log::{debug, error, info, log_enabled, trace, warn, Level, LevelFilter};
// use ratatui::{backend::CrosstermBackend, Terminal};
// use utils::{get_config_dir, get_data_dir};

// use crate::{
//   action::Action,
//   keyconfig::KeyConfig,
//   utils::{initialize_logging, initialize_panic_handler},
// };
//
// const LOG_PATTERN: &str = "{d(%Y-%m-%d %H:%M:%S)} | {l} | {f}:{L} | {m}{n}";

#[tokio::main]
async fn main() -> Result<()> {
  let matches = cli::generate_cli_app().get_matches();

  let config = matches.get_one::<String>("config");
  let data = matches.get_one::<String>("data");
  let taskrc = matches.get_one::<String>("taskrc");
  let taskdata = matches.get_one::<String>("taskdata");
  let binding = String::from("next");
  let report = matches.get_one::<String>("report").unwrap_or(&binding);

  let config_dir = config.map(PathBuf::from).unwrap_or_else(get_config_dir);
  let data_dir = data.map(PathBuf::from).unwrap_or_else(get_data_dir);

  if let Some(e) = taskrc {
    if env::var("TASKRC").is_err() {
      // if environment variable is not set, this env::var returns an error
      env::set_var(
        "TASKRC",
        absolute_path(PathBuf::from(e)).expect("Unable to get path for taskrc"),
      )
    } else {
      log::warn!("TASKRC environment variable cannot be set.")
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
      log::warn!("TASKDATA environment variable cannot be set.")
    }
  }

  initialize_logging()?;
  initialize_panic_handler()?;

  log::info!("getting matches from clap...");
  log::debug!("report = {:?}", &report);
  log::debug!("config = {:?}", &config);

  let mut app = app::TaskwarriorTui::new(report)?;

  let r = app.run().await;

  if let Err(err) = r {
    eprintln!("\x1b[0;31m[taskwarrior-tui error]\x1b[0m: {}\n\nIf you need additional help, please report as a github issue on https://github.com/kdheepak/taskwarrior-tui", err);
    std::process::exit(1);
  }
  Ok(())
}
