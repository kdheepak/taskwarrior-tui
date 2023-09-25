use std::{collections::HashMap, time::Duration};

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::*, widgets::*};
use serde_derive::{Deserialize, Serialize};
use task_hookrs::{import::import, task::Task};
use tokio::sync::mpsc::UnboundedSender;
use tui_input::backend::crossterm::EventHandler;
use uuid::Uuid;

use super::{Component, Frame};
use crate::{
  command::Command,
  config::{Config, KeyBindings},
};

#[derive(Default)]
pub struct TaskReport {
  pub config: Config,
  pub command_tx: Option<UnboundedSender<Command>>,
  pub last_export: Option<std::time::SystemTime>,
  pub report: String,
  pub filter: String,
  pub current_context_filter: String,
  pub tasks: Vec<Task>,
}

impl TaskReport {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn refresh(&mut self) -> Result<()> {
    self.last_export = Some(std::time::SystemTime::now());
    Ok(())
  }

  pub fn send_command(&self, command: Command) -> Result<()> {
    if let Some(ref tx) = self.command_tx {
      tx.send(command)?;
    }
    Ok(())
  }

  pub fn task_export(&mut self) -> Result<()> {
    let mut task = std::process::Command::new("task");

    task
      .arg("rc.json.array=on")
      .arg("rc.confirmation=off")
      .arg("rc.json.depends.array=on")
      .arg("rc.color=off")
      .arg("rc._forcecolor=off");
    // .arg("rc.verbose:override=false");

    if let Some(args) = shlex::split(format!(r#"rc.report.{}.filter='{}'"#, self.report, self.filter.trim()).trim()) {
      for arg in args {
        task.arg(arg);
      }
    }

    if !self.current_context_filter.trim().is_empty() {
      if let Some(args) = shlex::split(&self.current_context_filter) {
        for arg in args {
          task.arg(arg);
        }
      }
    }

    task.arg("export");

    task.arg(&self.report);

    log::info!("Running `{:?}`", task);
    let output = task.output()?;
    let data = String::from_utf8_lossy(&output.stdout);
    let error = String::from_utf8_lossy(&output.stderr);

    if output.status.success() {
      if let Ok(imported) = import(data.as_bytes()) {
        self.tasks = imported;
        log::info!("Imported {} tasks", self.tasks.len());
        self.send_command(Command::ShowTaskReport)?;
      }
    } else {
      self.send_command(Command::Error(format!("Unable to parse output of `{:?}`:\n`{:?}`", task, data)))?;
    }

    Ok(())
  }
}

impl Component for TaskReport {
  fn register_command_handler(&mut self, tx: UnboundedSender<Command>) -> Result<()> {
    self.command_tx = Some(tx);
    Ok(())
  }

  fn register_config_handler(&mut self, config: Config) -> Result<()> {
    self.config = config;
    Ok(())
  }

  fn update(&mut self, command: Command) -> Result<Option<Command>> {
    match command {
      _ => (),
    }
    Ok(None)
  }

  fn draw(&mut self, f: &mut Frame<'_>, rect: Rect) -> Result<()> {
    Ok(())
  }
}
