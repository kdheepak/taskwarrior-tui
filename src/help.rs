use std::cmp;

use ratatui::{
  buffer::Buffer,
  layout::{Alignment, Rect},
  style::{Modifier, Style},
  text::{Line, Span, Text},
  widgets::{Block, BorderType, Borders, Clear, Paragraph, Widget},
};

const TEMPLATE: &str = include_str!("help.tmpl");

use crate::{event::KeyCode, keyconfig::KeyConfig};

pub struct Help {
  pub title: String,
  pub scroll: u16,
  pub text_height: usize,
  /// Dynamically generated contents of the Help screen
  pub text: String,
}

/// Returns the configured KeyCode for a given name string
fn keycode_for(name: &str, kc: &KeyConfig) -> KeyCode {
  match name {
    "quit" => kc.quit,
    "refresh" => kc.refresh,
    "next_tab" => kc.next_tab,
    "previous_tab" => kc.previous_tab,
    "filter" => kc.filter,
    "add" => kc.add,
    "done" => kc.done,
    "edit" => kc.edit,
    "duplicate" => kc.duplicate,
    "down" => kc.down,
    "up" => kc.up,
    "page_down" => kc.page_down,
    "page_up" => kc.page_up,
    "go_to_top" => kc.go_to_top,
    "go_to_bottom" => kc.go_to_bottom,
    "log" => kc.log,
    "modify" => kc.modify,
    "start_stop" => kc.start_stop,
    "quick_tag" => kc.quick_tag,
    "undo" => kc.undo,
    "select" => kc.select,
    "select_all" => kc.select_all,
    "delete" => kc.delete,
    "zoom" => kc.zoom,
    "annotate" => kc.annotate,
    "shell" => kc.shell,
    "shortcut0" => kc.shortcut0,
    "shortcut1" => kc.shortcut1,
    "shortcut2" => kc.shortcut2,
    "shortcut3" => kc.shortcut3,
    "shortcut4" => kc.shortcut4,
    "shortcut5" => kc.shortcut5,
    "shortcut6" => kc.shortcut6,
    "shortcut7" => kc.shortcut7,
    "shortcut8" => kc.shortcut8,
    "shortcut9" => kc.shortcut9,
    "context_menu" => kc.context_menu,
    "help" => kc.help,
    "scroll_down" => kc.scroll_down,
    "scroll_up" => kc.scroll_up,
    "jump" => kc.jump,
    "reset_filter" => kc.reset_filter,
    "priority_h" => kc.priority_h,
    "priority_m" => kc.priority_m,
    "priority_l" => kc.priority_l,
    "priority_n" => kc.priority_n,
    "priority_up" => kc.priority_up,
    "priority_down" => kc.priority_down,
    _ => KeyCode::Null,
  }
}

/// Formats a KeyCode into a human-readable display string for the help screen.
/// Uses angle-bracket notation consistent with the config file format.
fn keycode_to_string(kc: KeyCode) -> String {
  match kc {
    KeyCode::Char('\n') => "<Enter>".to_string(),
    KeyCode::Char(' ') => "<Space>".to_string(),
    KeyCode::Char(c) => c.to_string(),
    KeyCode::Ctrl(c) => format!("<C-{}>", c),
    KeyCode::Alt(c) => format!("<A-{}>", c),
    KeyCode::Esc => "<Esc>".to_string(),
    KeyCode::Tab => "<Tab>".to_string(),
    KeyCode::BackTab => "<S-Tab>".to_string(),
    KeyCode::Backspace => "<BS>".to_string(),
    KeyCode::Delete => "<Del>".to_string(),
    KeyCode::Insert => "<Ins>".to_string(),
    KeyCode::Up => "<Up>".to_string(),
    KeyCode::Down => "<Down>".to_string(),
    KeyCode::Left => "<Left>".to_string(),
    KeyCode::Right => "<Right>".to_string(),
    KeyCode::PageUp => "<PageUp>".to_string(),
    KeyCode::PageDown => "<PageDown>".to_string(),
    KeyCode::Home => "<Home>".to_string(),
    KeyCode::End => "<End>".to_string(),
    KeyCode::F(n) => format!("<F{}>", n),
    KeyCode::CtrlBackspace => "<C-BS>".to_string(),
    KeyCode::CtrlDelete => "<C-Del>".to_string(),
    KeyCode::AltBackspace => "<A-BS>".to_string(),
    KeyCode::AltDelete => "<A-Del>".to_string(),
    KeyCode::Null => "<Null>".to_string(),
    KeyCode::Nop => "<Nop>".to_string(),
  }
}

/// Generates the Help text from the template based on the current
/// key configuration. Every substring `{{token}}` in the template
/// is replaced with the display string for the configured key.
/// Lines containing a `|` delimiter are then aligned into two columns:
/// the key column (left of `|`) is padded to a uniform width so the
/// description column (right of `|`) lines up.
fn render_help(kc: &KeyConfig, tmpl: &str) -> String {
  // Pass 1: substitute {{token}} placeholders
  let mut substituted = String::with_capacity(tmpl.len());
  let mut i = 0;
  while let Some(start) = tmpl[i..].find("{{") {
    let s = i + start;
    substituted.push_str(&tmpl[i..s]);
    if let Some(end) = tmpl[s + 2..].find("}}") {
      let e = s + 2 + end;
      let name = &tmpl[s + 2..e];
      let key = keycode_for(name, kc);
      substituted.push_str(&keycode_to_string(key));
      i = e + 2;
      continue;
    }
    break;
  }
  substituted.push_str(&tmpl[i..]);

  // Pass 2: find maximum key column width across all `|`-delimited lines
  let lines: Vec<&str> = substituted.lines().collect();
  let max_key_width = lines
    .iter()
    .filter_map(|line| {
      let pipe = line.find('|')?;
      Some(line[..pipe].len())
    })
    .max()
    .unwrap_or(0);

  // Pass 3: rebuild output with aligned columns
  let mut out = String::with_capacity(substituted.len() + lines.len() * 4);
  for (idx, line) in lines.iter().enumerate() {
    if let Some(pipe) = line.find('|') {
      let key_part = &line[..pipe];
      let desc_part = &line[pipe + 1..];
      out.push_str(&format!("{:<width$}  {}", key_part, desc_part, width = max_key_width));
    } else {
      out.push_str(line);
    }
    if idx < lines.len() - 1 {
      out.push('\n');
    }
  }
  out
}

impl Help {
  pub fn new(keyconfig: &KeyConfig) -> Self {
    let text = render_help(keyconfig, TEMPLATE);
    let text_height = text.lines().count();

    Self {
      title: "Help".to_string(),
      scroll: 0,
      text_height,
      text,
    }
  }
}

impl Default for Help {
  fn default() -> Self {
    Self::new(&KeyConfig::default())
  }
}

impl Widget for &Help {
  fn render(self, area: Rect, buf: &mut Buffer) {
    let text: Vec<Line> = self.text.lines().map(|l| Line::from(l.to_owned())).collect();
    Paragraph::new(text)
      .block(
        Block::default()
          .title(Span::styled(&self.title, Style::default().add_modifier(Modifier::BOLD)))
          .borders(Borders::ALL)
          .border_type(BorderType::Rounded),
      )
      .alignment(Alignment::Left)
      .scroll((self.scroll, 0))
      .render(area, buf);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_keycode_to_string_nop() {
    assert_eq!(keycode_to_string(KeyCode::Nop), "<Nop>");
  }

  #[test]
  fn test_render_help_shows_nop_binding() {
    let mut kc = KeyConfig::default();
    kc.shell = KeyCode::Nop;
    let text = render_help(&kc, TEMPLATE);
    assert!(text.contains("<Nop>"), "Help text should show <Nop> for disabled binding");
  }
}
