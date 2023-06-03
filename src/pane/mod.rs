use anyhow::Result;

use crate::action::Action;
use crate::app::{App, Mode};
use crossterm::event::KeyCode;
use std::ops::Index;

pub mod context;
pub mod project;

pub trait Pane {
  fn handle_input(app: &mut App, input: KeyCode) -> Result<()>;
  fn change_focus_to_left_pane(app: &mut App) {
    match app.mode {
      Mode::Tasks(_) => {
        if app.config.uda_change_focus_rotate {
          app.mode = Mode::Calendar;
        }
      }
      Mode::Projects => app.mode = Mode::Tasks(Action::Report),
      Mode::Calendar => {
        app.mode = Mode::Projects;
      }
    }
  }
  fn change_focus_to_right_pane(app: &mut App) {
    match app.mode {
      Mode::Tasks(_) => app.mode = Mode::Projects,
      Mode::Projects => app.mode = Mode::Calendar,
      Mode::Calendar => {
        if app.config.uda_change_focus_rotate {
          app.mode = Mode::Tasks(Action::Report);
        }
      }
    }
  }
}
