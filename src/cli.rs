use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Cli {
  #[arg(short, long, value_name = "FOLDER", help = "Sets the data folder for taskwarrior-tui")]
  pub data: Option<String>,

  #[arg(short, long, value_name = "FOLDER", help = "Sets the config folder for taskwarrior-tui")]
  pub config: Option<String>,

  #[arg(
    long,
    value_name = "FOLDER",
    help = "Sets the .task folder using the TASKDATA environment variable for taskwarrior"
  )]
  pub taskdata: Option<PathBuf>,

  #[arg(
    long,
    value_name = "FILE",
    help = "Sets the .taskrc file using the TASKRC environment variable for taskwarrior"
  )]
  pub taskrc: Option<PathBuf>,

  #[arg(value_name = "FLOAT", help = "Tick rate, i.e. number of ticks per second", default_value_t = 1.0)]
  pub tick_rate: f64,

  #[arg(value_name = "FLOAT", help = "Frame rate, i.e. number of frames per second", default_value_t = 60.0)]
  pub frame_rate: f64,

  #[arg(short, long, value_name = "STRING", help = "Sets default report")]
  pub report: Option<String>,
}
