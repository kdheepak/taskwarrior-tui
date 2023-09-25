#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

pub mod app;
pub mod cli;
pub mod command;
pub mod components;
pub mod config;
pub mod tui;
pub mod utils;

use clap::Parser;
use cli::Cli;
use color_eyre::eyre::Result;

use crate::{
  app::App,
  utils::{initialize_logging, initialize_panic_handler, version},
};

async fn tokio_main() -> Result<()> {
  initialize_logging()?;

  initialize_panic_handler()?;

  let args = Cli::parse();
  let mut runner = App::new(args.tick_rate, args.frame_rate)?;
  runner.run().await?;

  Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
  tokio_main().await.unwrap();
  Ok(())
}
