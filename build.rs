#![allow(dead_code)]
use std::process::Command;

use clap_generate::{
    generate_to,
    generators::{Bash, Fish, PowerShell, Zsh},
};

include!("src/cli.rs");

fn run_pandoc() -> Result<(), &'static str> {
    let mut cmd = Command::new("pandoc");
    if let Some(args) = shlex::split("--standalone --to=man docs/taskwarrior-tui.1.md -o docs/taskwarrior-tui.1") {
        for arg in args {
            cmd.arg(arg);
        }
    }
    let output = cmd.output();
    match output {
        Ok(_) => Ok(()),
        Err(_) => Err("Cannot run `pandoc` to generate man page documentation."),
    }
}

fn main() {
    let mut app = generate_cli_app();
    let name = app.get_name().to_string();
    let outdir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("completions/");
    dbg!(&outdir);
    generate_to(Bash, &mut app, &name, &outdir).unwrap();
    generate_to(Zsh, &mut app, &name, &outdir).unwrap();
    generate_to(Fish, &mut app, &name, &outdir).unwrap();
    generate_to(PowerShell, &mut app, &name, &outdir).unwrap();
    run_pandoc().unwrap();
}
