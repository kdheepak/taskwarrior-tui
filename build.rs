#![allow(dead_code)]
use clap_generate::{
    generate_to,
    generators::{Bash, Fish, PowerShell, Zsh},
};

include!("src/cli.rs");

fn main() {
    let mut app = generate_cli_app();
    let name = app.get_name().to_string();
    let outdir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("completions/");
    dbg!(&outdir);
    generate_to(Bash, &mut app, &name, &outdir).unwrap();
    generate_to(Zsh, &mut app, &name, &outdir).unwrap();
    generate_to(Fish, &mut app, &name, &outdir).unwrap();
    generate_to(PowerShell, &mut app, &name, &outdir).unwrap();
}
