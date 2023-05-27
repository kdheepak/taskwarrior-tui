use rustyline::line_buffer::ChangeListener;
use rustyline::line_buffer::DeleteListener;
use rustyline::line_buffer::Direction;

/// Undo manager
#[derive(Default)]
pub struct Changeset {}

impl DeleteListener for Changeset {
  fn delete(&mut self, idx: usize, string: &str, _: Direction) {}
}

impl ChangeListener for Changeset {
  fn insert_char(&mut self, idx: usize, c: char) {}

  fn insert_str(&mut self, idx: usize, string: &str) {}

  fn replace(&mut self, idx: usize, old: &str, new: &str) {}
}
