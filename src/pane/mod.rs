use std::ops::Index;

use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;

use crate::{
  action::Action,
  app::{Mode, TaskwarriorTui},
  tui::Event,
};

pub mod context;
pub mod project;

pub trait Pane {
  fn handle_input(app: &mut TaskwarriorTui, input: KeyEvent) -> Result<()>;
  fn change_focus_to_left_pane(app: &mut TaskwarriorTui) {
    match app.mode {
      Mode::Projects => app.mode = Mode::TaskReport,
      Mode::Calendar => {
        app.mode = Mode::Projects;
      }
      _ => {
        if app.config.uda_change_focus_rotate {
          app.mode = Mode::Calendar;
        }
      }
    }
  }
  fn change_focus_to_right_pane(app: &mut TaskwarriorTui) {
    match app.mode {
      Mode::Projects => app.mode = Mode::Calendar,
      Mode::Calendar => {
        if app.config.uda_change_focus_rotate {
          app.mode = Mode::TaskReport;
        }
      }
      _ => app.mode = Mode::Projects,
    }
  }
}
