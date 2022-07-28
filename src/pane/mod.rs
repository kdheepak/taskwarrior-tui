use anyhow::Result;

use crate::action::Action;
use crate::app::{Mode, TaskwarriorTui};
use crate::event::KeyCode;
use std::ops::Index;

pub mod context;
pub mod project;

pub trait Pane {
    fn handle_input(app: &mut TaskwarriorTui, input: KeyCode) -> Result<()>;
    fn change_focus_to_left_pane(app: &mut TaskwarriorTui) {
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
    fn change_focus_to_right_pane(app: &mut TaskwarriorTui) {
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
