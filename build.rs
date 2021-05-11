#![allow(dead_code)]
use clap::{App, Arg};
use clap_generate::{generate_to, generators::*};

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const APP_NAME: &str = env!("CARGO_PKG_NAME");

fn main() {
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
    let name = app.get_name().to_string();
    let outdir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("completions/");
    dbg!(&outdir);
    generate_to::<Bash, _, _>(&mut app, &name, &outdir);
    generate_to::<Zsh, _, _>(&mut app, &name, &outdir);
    generate_to::<Fish, _, _>(&mut app, &name, &outdir);
    generate_to::<PowerShell, _, _>(&mut app, &name, &outdir);
}
