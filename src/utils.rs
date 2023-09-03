use rustyline::line_buffer::{ChangeListener, DeleteListener, Direction};

/// Undo manager
#[derive(Default)]
pub struct Changeset {}

impl DeleteListener for Changeset {
  fn delete(&mut self, idx: usize, string: &str, _: Direction) {}
}

impl ChangeListener for Changeset {
  fn insert_char(&mut self, idx: usize, c: char) {}

  fn insert_str(&mut self, idx: usize, string: &str) {}

  fn replace(&mut self, idx: usize, old: &str, new: &str) {}
}

use std::path::PathBuf;

use color_eyre::eyre::{anyhow, Context, Result};
use directories::ProjectDirs;
use lazy_static::lazy_static;
use tracing::error;
use tracing_error::ErrorLayer;
use tracing_subscriber::{self, filter::EnvFilter, prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, Layer};

use crate::tui::Tui;

lazy_static! {
  pub static ref CRATE_NAME: String = env!("CARGO_CRATE_NAME").to_uppercase().to_string();
  pub static ref DATA_FOLDER: Option<PathBuf> = std::env::var(format!("{}_DATA", CRATE_NAME.clone())).ok().map(PathBuf::from);
  pub static ref CONFIG_FOLDER: Option<PathBuf> = std::env::var(format!("{}_CONFIG", CRATE_NAME.clone())).ok().map(PathBuf::from);
  pub static ref GIT_COMMIT_HASH: String = std::env::var(format!("{}_GIT_INFO", CRATE_NAME.clone())).unwrap_or_else(|_| String::from("Unknown"));
  pub static ref LOG_FILE: String = format!("{}.log", CRATE_NAME.to_lowercase());
}

fn project_directory() -> Option<ProjectDirs> {
  ProjectDirs::from("com", "kdheepak", CRATE_NAME.clone().to_lowercase().as_str())
}

pub fn initialize_panic_handler() -> Result<()> {
  let (panic_hook, eyre_hook) = color_eyre::config::HookBuilder::default().into_hooks();
  eyre_hook.install()?;
  std::panic::set_hook(Box::new(move |panic_info| {
    if let Ok(t) = Tui::new(0) {
      if let Err(r) = t.exit() {
        error!("Unable to exit Terminal: {:?}", r);
      }
    }
    let msg = format!("{}", panic_hook.panic_report(panic_info));
    tracing::error!("{}", strip_ansi_escapes::strip_str(&msg));
    use human_panic::{handle_dump, print_msg, Metadata};
    let meta = Metadata {
      version: env!("CARGO_PKG_VERSION").into(),
      name: env!("CARGO_PKG_NAME").into(),
      authors: env!("CARGO_PKG_AUTHORS").replace(':', ", ").into(),
      homepage: env!("CARGO_PKG_HOMEPAGE").into(),
    };
    let file_path = handle_dump(&meta, panic_info);
    print_msg(file_path, &meta).expect("human-panic: printing error message to console failed");
    eprintln!("{}", msg);
    std::process::exit(libc::EXIT_FAILURE);
  }));
  Ok(())
}

pub fn get_data_dir() -> PathBuf {
  let directory = if let Some(s) = DATA_FOLDER.clone() {
    s
  } else if let Some(proj_dirs) = project_directory() {
    proj_dirs.data_local_dir().to_path_buf()
  } else {
    PathBuf::from(".").join(".data")
  };
  directory
}

pub fn get_config_dir() -> PathBuf {
  let directory = if let Some(s) = CONFIG_FOLDER.clone() {
    s
  } else if let Some(proj_dirs) = project_directory() {
    proj_dirs.config_local_dir().to_path_buf()
  } else {
    PathBuf::from(".").join(".config")
  };
  directory
}

pub fn initialize_logging(directory: PathBuf) -> Result<()> {
  std::fs::create_dir_all(directory.clone())?;
  let log_path = directory.join(LOG_FILE.clone());
  let log_file = std::fs::File::create(log_path)?;
  let file_subscriber = tracing_subscriber::fmt::layer()
    .with_file(true)
    .with_line_number(true)
    .with_writer(log_file)
    .with_target(false)
    .with_ansi(false)
    .with_filter(EnvFilter::from_default_env());

  tracing_subscriber::registry()
    .with(file_subscriber)
    .with(tui_logger::tracing_subscriber_layer())
    .with(ErrorLayer::default())
    .init();
  let default_level = std::env::var("RUST_LOG").map_or(log::LevelFilter::Info, |val| match val.to_lowercase().as_str() {
    "off" => log::LevelFilter::Off,
    "error" => log::LevelFilter::Error,
    "warn" => log::LevelFilter::Warn,
    "info" => log::LevelFilter::Info,
    "debug" => log::LevelFilter::Debug,
    "trace" => log::LevelFilter::Trace,
    _ => log::LevelFilter::Info,
  });
  tui_logger::set_default_level(default_level);

  Ok(())
}

/// Similar to the `std::dbg!` macro, but generates `tracing` events rather
/// than printing to stdout.
///
/// By default, the verbosity level for the generated events is `DEBUG`, but
/// this can be customized.
#[macro_export]
macro_rules! trace_dbg {
    (target: $target:expr, level: $level:expr, $ex:expr) => {{
        match $ex {
            value => {
                tracing::event!(target: $target, $level, ?value, stringify!($ex));
                value
            }
        }
    }};
    (level: $level:expr, $ex:expr) => {
        trace_dbg!(target: module_path!(), level: $level, $ex)
    };
    (target: $target:expr, $ex:expr) => {
        trace_dbg!(target: $target, level: tracing::Level::DEBUG, $ex)
    };
    ($ex:expr) => {
        trace_dbg!(level: tracing::Level::DEBUG, $ex)
    };
}

pub fn version() -> String {
  let author = clap::crate_authors!();

  let commit_hash = GIT_COMMIT_HASH.clone();

  // let current_exe_path = PathBuf::from(clap::crate_name!()).display().to_string();
  let config_dir_path = get_config_dir().display().to_string();
  let data_dir_path = get_data_dir().display().to_string();

  format!(
    "\
{commit_hash}

Authors: {author}

Config directory: {config_dir_path}
Data directory: {data_dir_path}"
  )
}
