use clap::Arg;

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const APP_NAME: &str = env!("CARGO_PKG_NAME");

pub fn generate_cli_app() -> clap::Command {
  let mut app = clap::Command::new(APP_NAME)
    .version(APP_VERSION)
    .author("Dheepak Krishnamurthy <@kdheepak>")
    .about("A taskwarrior terminal user interface")
    .arg(
      Arg::new("data")
        .short('d')
        .long("data")
        .value_name("FOLDER")
        .help("Sets the data folder for taskwarrior-tui")
        .action(clap::ArgAction::Set),
    )
    .arg(
      Arg::new("config")
        .short('c')
        .long("config")
        .value_name("FOLDER")
        .help("Sets the config folder for taskwarrior-tui (currently not used)")
        .action(clap::ArgAction::Set),
    )
    .arg(
      Arg::new("taskdata")
        .long("taskdata")
        .value_name("FOLDER")
        .help("Sets the .task folder using the TASKDATA environment variable for taskwarrior")
        .action(clap::ArgAction::Set),
    )
    .arg(
      Arg::new("taskrc")
        .long("taskrc")
        .value_name("FILE")
        .help("Sets the .taskrc file using the TASKRC environment variable for taskwarrior")
        .action(clap::ArgAction::Set),
    )
    .arg(
      Arg::new("report")
        .short('r')
        .long("report")
        .value_name("STRING")
        .help("Sets default report")
        .action(clap::ArgAction::Set),
    );

  app.set_bin_name(APP_NAME);
  app
}
