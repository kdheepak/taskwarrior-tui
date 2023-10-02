use color_eyre::eyre::Result;
use ratatui::{prelude::*, widgets::*};
use task_hookrs::task::Task;
use tokio::sync::mpsc::UnboundedSender;

use super::{Component, Frame};
use crate::{action::Action, config::Config};

#[derive(Default)]
pub struct TaskDetails {
  pub command_tx: Option<UnboundedSender<Action>>,
  pub config: Config,
  pub uuid: Option<uuid::Uuid>,
  pub task: Option<Task>,
}

impl TaskDetails {
  pub fn update(&mut self) {
    if let Some(uuid) = self.uuid {
      let tx = self.command_tx.clone().unwrap();
      tokio::spawn(async move {
        let output = tokio::process::Command::new("task")
          .arg("rc.json.array=on")
          .arg("rc.confirmation=off")
          .arg("rc.json.depends.array=on")
          .arg("rc.color=off")
          .arg("rc._forcecolor=off")
          .arg(format!("{}", uuid))
          .arg("export")
          .output()
          .await;
        if let Ok(output) = output {
          let data = String::from_utf8_lossy(&output.stdout).to_string();
        }
      });
    }
  }
}

impl Component for TaskDetails {
  fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
    self.command_tx = Some(tx);
    Ok(())
  }

  fn update(&mut self, action: Action) -> Result<Option<Action>> {
    match action {
      Action::TaskDetailsUpdateUuid(uuid) => {
        self.uuid = Some(uuid);
        self.update();
      },
      Action::TaskDetailsUpdateData((uuid, data)) => {
        let imported = task_hookrs::import::import(data.as_bytes());
        if let Ok(tasks) = imported {
          self.task = Some(tasks[0].clone())
        }
      },
      _ => {},
    }
    Ok(None)
  }

  fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
    Ok(())
  }
}
