use clap::{App, Arg};

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const APP_NAME: &str = env!("CARGO_PKG_NAME");

pub fn generate_cli_app() -> App<'static> {
    let mut app = App::new(APP_NAME)
        .version(APP_VERSION)
        .author("Dheepak Krishnamurthy <@kdheepak>")
        .about("A taskwarrior terminal user interface")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .about("Sets a custom config file")
                .takes_value(true),
        )
        .arg(
            Arg::new("report")
                .short('r')
                .long("report")
                .value_name("STRING")
                .about("Sets default report")
                .takes_value(true),
        );

    app.set_bin_name(APP_NAME);
    app
}
