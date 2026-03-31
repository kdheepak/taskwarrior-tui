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
mod datetime;
mod event;
mod help;
mod history;
mod keyconfig;
mod pane;
mod scrollbar;
mod table;
mod task_report;
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

use anyhow::Result;
use app::{Mode, TaskwarriorTui};
use crossterm::{
  cursor,
  event::{DisableBracketedPaste, DisableMouseCapture, EnableMouseCapture, EventStream},
  execute,
  terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use futures::stream::{FuturesUnordered, StreamExt};
use log::{Level, LevelFilter, debug, error, info, log_enabled, trace, warn};
use log4rs::{
  append::file::FileAppender,
  config::{Appender, Config, Logger, Root},
  encode::pattern::PatternEncoder,
};
use path_clean::PathClean;
use ratatui::{Terminal, backend::CrosstermBackend};

use crate::{action::Action, event::Event, keyconfig::KeyConfig};

const LOG_PATTERN: &str = "{d(%Y-%m-%d %H:%M:%S)} | {l} | {f}:{L} | {m}{n}";

pub fn destruct_terminal() {
  disable_raw_mode().unwrap();
  execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture, DisableBracketedPaste).unwrap();
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

  let levelfilter = match std::env::var("TASKWARRIOR_TUI_LOG_LEVEL").unwrap_or_else(|_| "info".to_string()).as_str() {
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

fn set_env_path_if_unset(key: &str, value: &str, path_name: &str) {
  if env::var_os(key).is_none() {
    let absolute_path = absolute_path(PathBuf::from(value)).unwrap_or_else(|_| panic!("Unable to get path for {path_name}"));

    // SAFETY: this runs in `main` before the Tokio runtime is created and before
    // any threads are spawned, so there is no concurrent access to the process
    // environment while mutating it.
    unsafe {
      env::set_var(key, absolute_path);
    }
  } else {
    warn!("{key} environment variable cannot be set.")
  }
}

async fn tui_main(report: &str) -> Result<()> {
  panic::set_hook(Box::new(|panic_info| {
    destruct_terminal();
    better_panic::Settings::auto().create_panic_handler()(panic_info);
  }));

  let mut app = app::TaskwarriorTui::new(report, true).await?;

  let mut terminal = app.start_tui()?;

  let r = app.run(&mut terminal).await;

  app.pause_tui().await?;

  r
}

fn main() -> Result<()> {
  better_panic::install();

  let matches = cli::generate_cli_app().get_matches();

  let config = matches.get_one::<String>("config");
  let data = matches.get_one::<String>("data");
  let taskrc = matches.get_one::<String>("taskrc");
  let taskdata = matches.get_one::<String>("taskdata");
  let binding = String::from("next");
  let report = matches.get_one::<String>("report").unwrap_or(&binding);

  if let Some(e) = config {
    set_env_path_if_unset("TASKWARRIOR_TUI_CONFIG", e, "config");
  }

  if let Some(e) = data {
    set_env_path_if_unset("TASKWARRIOR_TUI_DATA", e, "data");
  }

  if let Some(e) = taskrc {
    set_env_path_if_unset("TASKRC", e, "taskrc");
  }

  if let Some(e) = taskdata {
    set_env_path_if_unset("TASKDATA", e, "taskdata");
  }

  initialize_logging();

  debug!("getting matches from clap...");
  debug!("report = {:?}", &report);
  debug!("config = {:?}", &config);

  let r = tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .build()?
    .block_on(async { tui_main(report).await });
  if let Err(err) = r {
    eprintln!(
      "\x1b[0;31m[taskwarrior-tui error]\x1b[0m: {}\n\nIf you need additional help, please report as a github issue on https://github.com/kdheepak/taskwarrior-tui",
      err
    );
    std::process::exit(1);
  }
  Ok(())
}
