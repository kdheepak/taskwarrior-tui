#![allow(dead_code)]
use clap_generate::{generate_to, generators::*};

include!("src/cli.rs");

fn main() {
    let mut app = generate_cli_app();
    let name = app.get_name().to_string();
    let outdir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("completions/");
    dbg!(&outdir);
    generate_to::<Bash, _, _>(&mut app, &name, &outdir);
    generate_to::<Zsh, _, _>(&mut app, &name, &outdir);
    generate_to::<Fish, _, _>(&mut app, &name, &outdir);
    generate_to::<PowerShell, _, _>(&mut app, &name, &outdir);
}
