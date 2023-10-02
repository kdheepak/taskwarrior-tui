use color_eyre::eyre::Result;
use futures::channel::mpsc::UnboundedSender;
use ratatui::{prelude::*, widgets::*};
use task_hookrs::task::Task;

use super::{Component, Frame};
use crate::{action::Action, config::Config};

#[derive(Default)]
pub struct TaskDetails {
  pub command_tx: Option<UnboundedSender<Action>>,
  pub config: Config,
}

impl Component for TaskDetails {
  fn update(&mut self, action: Action) -> Result<Option<Action>> {
    match action {
      Action::TaskDetailsUpdateUuid(uuid) => {},
      _ => {},
    }
  }

  fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
    Ok(())
  }
}
