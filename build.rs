#![allow(dead_code)]
use std::process::{Command, Output};

use clap_complete::{
  generate_to,
  shells::{Bash, Fish, PowerShell, Zsh},
};

include!("src/cli.rs");

fn run_pandoc() -> Result<Output, std::io::Error> {
  let mut cmd = Command::new("pandoc");
  if let Some(args) = shlex::split("--standalone --to=man docs/taskwarrior-tui.1.md -o docs/taskwarrior-tui.1") {
    for arg in args {
      cmd.arg(arg);
    }
  }
  cmd.output()
}

fn get_commit_hash() {
  let git_output = std::process::Command::new("git").args(["rev-parse", "--git-dir"]).output().ok();
  let git_dir = git_output.as_ref().and_then(|output| {
    std::str::from_utf8(&output.stdout)
      .ok()
      .and_then(|s| s.strip_suffix('\n').or_else(|| s.strip_suffix("\r\n")))
  });

  // Tell cargo to rebuild if the head or any relevant refs change.
  if let Some(git_dir) = git_dir {
    let git_path = std::path::Path::new(git_dir);
    let refs_path = git_path.join("refs");
    if git_path.join("HEAD").exists() {
      println!("cargo:rerun-if-changed={}/HEAD", git_dir);
    }
    if git_path.join("packed-refs").exists() {
      println!("cargo:rerun-if-changed={}/packed-refs", git_dir);
    }
    if refs_path.join("heads").exists() {
      println!("cargo:rerun-if-changed={}/refs/heads", git_dir);
    }
    if refs_path.join("tags").exists() {
      println!("cargo:rerun-if-changed={}/refs/tags", git_dir);
    }
  }

  let git_output = std::process::Command::new("git")
    .args(["describe", "--always", "--tags", "--long", "--dirty"])
    .output()
    .ok();
  let git_info = git_output
    .as_ref()
    .and_then(|output| std::str::from_utf8(&output.stdout).ok().map(str::trim));
  let cargo_pkg_version = env!("CARGO_PKG_VERSION");

  // Default git_describe to cargo_pkg_version
  let mut git_describe = String::from(cargo_pkg_version);

  if let Some(git_info) = git_info {
    // If the `git_info` contains `CARGO_PKG_VERSION`, we simply use `git_info` as it is.
    // Otherwise, prepend `CARGO_PKG_VERSION` to `git_info`.
    if git_info.contains(cargo_pkg_version) {
      // Remove the 'g' before the commit sha
      let git_info = &git_info.replace('g', "");
      git_describe = git_info.to_string();
    } else {
      git_describe = format!("v{}-{}", cargo_pkg_version, git_info);
    }
  }

  println!("cargo:rustc-env=TASKWARRIOR_TUI_GIT_INFO={}", git_describe);
}

fn main() {
  get_commit_hash();
  let mut app = generate_cli_app();
  let name = app.get_name().to_string();
  let outdir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("completions/");
  dbg!(&outdir);
  generate_to(Bash, &mut app, &name, &outdir).unwrap();
  generate_to(Zsh, &mut app, &name, &outdir).unwrap();
  generate_to(Fish, &mut app, &name, &outdir).unwrap();
  generate_to(PowerShell, &mut app, &name, &outdir).unwrap();
  if run_pandoc().is_err() {
    dbg!("Unable to run pandoc to generate man page documentation");
  }
}
