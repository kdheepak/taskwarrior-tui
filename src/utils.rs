use rustyline::line_buffer::{ChangeListener, DeleteListener, Direction};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

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

pub fn display_control_chars(text: &str) -> String {
  let mut rendered = String::with_capacity(text.len());

  for c in text.chars() {
    match c {
      '\u{7f}' => rendered.push_str("^?"),
      c if c.is_ascii_control() => {
        rendered.push('^');
        rendered.push(char::from((c as u8).saturating_add(64)));
      }
      _ => rendered.push(c),
    }
  }

  rendered
}

pub fn display_width(text: &str) -> usize {
  display_control_chars(text).graphemes(true).map(|g| g.width()).sum()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_display_control_chars() {
    assert_eq!("hello^Jworld^M", display_control_chars("hello\nworld\r"));
    assert_eq!(14, display_width("hello\nworld\r"));
  }
}
