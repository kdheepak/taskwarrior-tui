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
    "context_menu" => kc.context_menu,
    "help" => kc.help,
    _ => KeyCode::Null,
  }
}

/// Generates the Help text from the template based on the current
/// key configuration. Every substring `{{token}}` in the template
/// is replaced with the character held by `kc.token`.
fn render_help(kc: &KeyConfig, tmpl: &str) -> String {
  // NOTE: assumes KeyCode is always a single char (which is currently the case)
  // this function MUST be updated if that ever changes
  let mut out = String::with_capacity(tmpl.len());
  let mut i = 0;
  while let Some(start) = tmpl[i..].find("{{") {
    let s = i + start;
    out.push_str(&tmpl[i..s]);
    if let Some(end) = tmpl[s + 2..].find("}}") {
      let e = s + 2 + end;
      let name = &tmpl[s + 2..e];
      if let KeyCode::Char(c) = keycode_for(name, kc) {
        out.push(c);
      }
      i = e + 2;
      continue;
    }
    break;
  }
  out.push_str(&tmpl[i..]);
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
