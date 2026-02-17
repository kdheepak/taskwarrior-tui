use std::{collections::HashSet, error::Error, hash::Hash};

use anyhow::{anyhow, Result};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};

use crate::event::KeyCode;

#[derive(Serialize, Deserialize, Debug)]
pub struct KeyConfig {
  pub quit: KeyCode,
  pub refresh: KeyCode,
  pub go_to_bottom: KeyCode,
  pub go_to_top: KeyCode,
  pub down: KeyCode,
  pub up: KeyCode,
  pub page_down: KeyCode,
  pub page_up: KeyCode,
  pub delete: KeyCode,
  pub done: KeyCode,
  pub start_stop: KeyCode,
  pub quick_tag: KeyCode,
  pub select: KeyCode,
  pub select_all: KeyCode,
  pub undo: KeyCode,
  pub edit: KeyCode,
  pub duplicate: KeyCode,
  pub modify: KeyCode,
  pub shell: KeyCode,
  pub log: KeyCode,
  pub add: KeyCode,
  pub annotate: KeyCode,
  pub help: KeyCode,
  pub filter: KeyCode,
  pub zoom: KeyCode,
  pub context_menu: KeyCode,
  pub next_tab: KeyCode,
  pub previous_tab: KeyCode,
  pub scroll_down: KeyCode,
  pub scroll_up: KeyCode,
  pub jump: KeyCode,
  pub reset_filter: KeyCode,
  pub priority_h: KeyCode,
  pub priority_m: KeyCode,
  pub priority_l: KeyCode,
  pub priority_n: KeyCode,
  pub priority_up: KeyCode,
  pub priority_down: KeyCode,
  pub shortcut0: KeyCode,
  pub shortcut1: KeyCode,
  pub shortcut2: KeyCode,
  pub shortcut3: KeyCode,
  pub shortcut4: KeyCode,
  pub shortcut5: KeyCode,
  pub shortcut6: KeyCode,
  pub shortcut7: KeyCode,
  pub shortcut8: KeyCode,
  pub shortcut9: KeyCode,
}

impl Default for KeyConfig {
  fn default() -> Self {
    Self {
      quit: KeyCode::Char('q'),
      refresh: KeyCode::Char('r'),
      go_to_bottom: KeyCode::Char('G'),
      go_to_top: KeyCode::Char('g'),
      down: KeyCode::Char('j'),
      up: KeyCode::Char('k'),
      page_down: KeyCode::Char('J'),
      page_up: KeyCode::Char('K'),
      delete: KeyCode::Char('x'),
      done: KeyCode::Char('d'),
      start_stop: KeyCode::Char('s'),
      quick_tag: KeyCode::Char('t'),
      select: KeyCode::Char('v'),
      select_all: KeyCode::Char('V'),
      undo: KeyCode::Char('u'),
      edit: KeyCode::Char('e'),
      duplicate: KeyCode::Char('y'),
      modify: KeyCode::Char('m'),
      shell: KeyCode::Char('!'),
      log: KeyCode::Char('l'),
      add: KeyCode::Char('a'),
      annotate: KeyCode::Char('A'),
      help: KeyCode::Char('?'),
      filter: KeyCode::Char('/'),
      zoom: KeyCode::Char('z'),
      context_menu: KeyCode::Char('c'),
      next_tab: KeyCode::Char(']'),
      previous_tab: KeyCode::Char('['),
      scroll_down: KeyCode::Ctrl('e'),
      scroll_up: KeyCode::Ctrl('y'),
      jump: KeyCode::Char(':'),
      reset_filter: KeyCode::Ctrl('r'),
      priority_h: KeyCode::Char('H'),
      priority_m: KeyCode::Char('M'),
      priority_l: KeyCode::Char('L'),
      priority_n: KeyCode::Char('N'),
      priority_up: KeyCode::Char('+'),
      priority_down: KeyCode::Char('-'),
      shortcut0: KeyCode::Char('0'),
      shortcut1: KeyCode::Char('1'),
      shortcut2: KeyCode::Char('2'),
      shortcut3: KeyCode::Char('3'),
      shortcut4: KeyCode::Char('4'),
      shortcut5: KeyCode::Char('5'),
      shortcut6: KeyCode::Char('6'),
      shortcut7: KeyCode::Char('7'),
      shortcut8: KeyCode::Char('8'),
      shortcut9: KeyCode::Char('9'),
    }
  }
}

impl KeyConfig {
  pub fn new(data: &str) -> Result<Self> {
    let mut kc = Self::default();
    kc.update(data)?;
    Ok(kc)
  }

  pub fn update(&mut self, data: &str) -> Result<()> {
    let quit = Self::get_config("uda.taskwarrior-tui.keyconfig.quit", data);
    let refresh = Self::get_config("uda.taskwarrior-tui.keyconfig.refresh", data);
    let go_to_bottom = Self::get_config("uda.taskwarrior-tui.keyconfig.go-to-bottom", data);
    let go_to_top = Self::get_config("uda.taskwarrior-tui.keyconfig.go-to-top", data);
    let down = Self::get_config("uda.taskwarrior-tui.keyconfig.down", data);
    let up = Self::get_config("uda.taskwarrior-tui.keyconfig.up", data);
    let page_down = Self::get_config("uda.taskwarrior-tui.keyconfig.page-down", data);
    let page_up = Self::get_config("uda.taskwarrior-tui.keyconfig.page-up", data);
    let delete = Self::get_config("uda.taskwarrior-tui.keyconfig.delete", data);
    let done = Self::get_config("uda.taskwarrior-tui.keyconfig.done", data);
    let start_stop = Self::get_config("uda.taskwarrior-tui.keyconfig.start-stop", data);
    let quick_tag = Self::get_config("uda.taskwarrior-tui.keyconfig.quick-tag", data);
    let select = Self::get_config("uda.taskwarrior-tui.keyconfig.select", data);
    let select_all = Self::get_config("uda.taskwarrior-tui.keyconfig.select-all", data);
    let undo = Self::get_config("uda.taskwarrior-tui.keyconfig.undo", data);
    let edit = Self::get_config("uda.taskwarrior-tui.keyconfig.edit", data);
    let duplicate = Self::get_config("uda.taskwarrior-tui.keyconfig.duplicate", data);
    let modify = Self::get_config("uda.taskwarrior-tui.keyconfig.modify", data);
    let shell = Self::get_config("uda.taskwarrior-tui.keyconfig.shell", data);
    let log = Self::get_config("uda.taskwarrior-tui.keyconfig.log", data);
    let add = Self::get_config("uda.taskwarrior-tui.keyconfig.add", data);
    let annotate = Self::get_config("uda.taskwarrior-tui.keyconfig.annotate", data);
    let filter = Self::get_config("uda.taskwarrior-tui.keyconfig.filter", data);
    let zoom = Self::get_config("uda.taskwarrior-tui.keyconfig.zoom", data);
    let context_menu = Self::get_config("uda.taskwarrior-tui.keyconfig.context-menu", data);
    let next_tab = Self::get_config("uda.taskwarrior-tui.keyconfig.next-tab", data);
    let previous_tab = Self::get_config("uda.taskwarrior-tui.keyconfig.previous-tab", data);
    let scroll_down = Self::get_config("uda.taskwarrior-tui.keyconfig.scroll-down", data);
    let scroll_up = Self::get_config("uda.taskwarrior-tui.keyconfig.scroll-up", data);
    let jump = Self::get_config("uda.taskwarrior-tui.keyconfig.jump", data);
    let reset_filter = Self::get_config("uda.taskwarrior-tui.keyconfig.reset-filter", data);
    let help = Self::get_config("uda.taskwarrior-tui.keyconfig.help", data);
    let priority_h = Self::get_config("uda.taskwarrior-tui.keyconfig.priority-h", data);
    let priority_m = Self::get_config("uda.taskwarrior-tui.keyconfig.priority-m", data);
    let priority_l = Self::get_config("uda.taskwarrior-tui.keyconfig.priority-l", data);
    let priority_n = Self::get_config("uda.taskwarrior-tui.keyconfig.priority-n", data);
    let priority_up = Self::get_config("uda.taskwarrior-tui.keyconfig.priority-up", data);
    let priority_down = Self::get_config("uda.taskwarrior-tui.keyconfig.priority-down", data);
    let shortcut0 = Self::get_config("uda.taskwarrior-tui.keyconfig.shortcut0", data);
    let shortcut1 = Self::get_config("uda.taskwarrior-tui.keyconfig.shortcut1", data);
    let shortcut2 = Self::get_config("uda.taskwarrior-tui.keyconfig.shortcut2", data);
    let shortcut3 = Self::get_config("uda.taskwarrior-tui.keyconfig.shortcut3", data);
    let shortcut4 = Self::get_config("uda.taskwarrior-tui.keyconfig.shortcut4", data);
    let shortcut5 = Self::get_config("uda.taskwarrior-tui.keyconfig.shortcut5", data);
    let shortcut6 = Self::get_config("uda.taskwarrior-tui.keyconfig.shortcut6", data);
    let shortcut7 = Self::get_config("uda.taskwarrior-tui.keyconfig.shortcut7", data);
    let shortcut8 = Self::get_config("uda.taskwarrior-tui.keyconfig.shortcut8", data);
    let shortcut9 = Self::get_config("uda.taskwarrior-tui.keyconfig.shortcut9", data);

    self.quit = quit.unwrap_or(self.quit);
    self.refresh = refresh.unwrap_or(self.refresh);
    self.go_to_bottom = go_to_bottom.unwrap_or(self.go_to_bottom);
    self.go_to_top = go_to_top.unwrap_or(self.go_to_top);
    self.down = down.unwrap_or(self.down);
    self.up = up.unwrap_or(self.up);
    self.page_down = page_down.unwrap_or(self.page_down);
    self.page_up = page_up.unwrap_or(self.page_up);
    self.delete = delete.unwrap_or(self.delete);
    self.done = done.unwrap_or(self.done);
    self.start_stop = start_stop.unwrap_or(self.start_stop);
    self.quick_tag = quick_tag.unwrap_or(self.quick_tag);
    self.select = select.unwrap_or(self.select);
    self.select_all = select_all.unwrap_or(self.select_all);
    self.undo = undo.unwrap_or(self.undo);
    self.edit = edit.unwrap_or(self.edit);
    self.duplicate = duplicate.unwrap_or(self.duplicate);
    self.modify = modify.unwrap_or(self.modify);
    self.shell = shell.unwrap_or(self.shell);
    self.log = log.unwrap_or(self.log);
    self.add = add.unwrap_or(self.add);
    self.annotate = annotate.unwrap_or(self.annotate);
    self.filter = filter.unwrap_or(self.filter);
    self.zoom = zoom.unwrap_or(self.zoom);
    self.context_menu = context_menu.unwrap_or(self.context_menu);
    self.next_tab = next_tab.unwrap_or(self.next_tab);
    self.previous_tab = previous_tab.unwrap_or(self.previous_tab);
    self.scroll_down = scroll_down.unwrap_or(self.scroll_down);
    self.scroll_up = scroll_up.unwrap_or(self.scroll_up);
    self.jump = jump.unwrap_or(self.jump);
    self.reset_filter = reset_filter.unwrap_or(self.reset_filter);
    self.help = help.unwrap_or(self.help);
    self.priority_h = priority_h.unwrap_or(self.priority_h);
    self.priority_m = priority_m.unwrap_or(self.priority_m);
    self.priority_l = priority_l.unwrap_or(self.priority_l);
    self.priority_n = priority_n.unwrap_or(self.priority_n);
    self.priority_up = priority_up.unwrap_or(self.priority_up);
    self.priority_down = priority_down.unwrap_or(self.priority_down);
    self.shortcut0 = shortcut0.unwrap_or(self.shortcut0);
    self.shortcut1 = shortcut1.unwrap_or(self.shortcut1);
    self.shortcut2 = shortcut2.unwrap_or(self.shortcut2);
    self.shortcut3 = shortcut3.unwrap_or(self.shortcut3);
    self.shortcut4 = shortcut4.unwrap_or(self.shortcut4);
    self.shortcut5 = shortcut5.unwrap_or(self.shortcut5);
    self.shortcut6 = shortcut6.unwrap_or(self.shortcut6);
    self.shortcut7 = shortcut7.unwrap_or(self.shortcut7);
    self.shortcut8 = shortcut8.unwrap_or(self.shortcut8);
    self.shortcut9 = shortcut9.unwrap_or(self.shortcut9);

    self.check()
  }

  pub fn check(&self) -> Result<()> {
    let elements = vec![
      ("quit", &self.quit),
      ("refresh", &self.refresh),
      ("go_to_bottom", &self.go_to_bottom),
      ("go_to_top", &self.go_to_top),
      ("down", &self.down),
      ("up", &self.up),
      ("page_down", &self.page_down),
      ("page_up", &self.page_up),
      ("delete", &self.delete),
      ("done", &self.done),
      ("select", &self.select),
      ("select_all", &self.select_all),
      ("start_stop", &self.start_stop),
      ("quick_tag", &self.quick_tag),
      ("undo", &self.undo),
      ("edit", &self.edit),
      ("duplicate", &self.duplicate),
      ("modify", &self.modify),
      ("shell", &self.shell),
      ("log", &self.log),
      ("add", &self.add),
      ("annotate", &self.annotate),
      ("help", &self.help),
      ("filter", &self.filter),
      ("zoom", &self.zoom),
      ("context_menu", &self.context_menu),
      ("next_tab", &self.next_tab),
      ("previous_tab", &self.previous_tab),
      ("scroll_down", &self.scroll_down),
      ("scroll_up", &self.scroll_up),
      ("jump", &self.jump),
      ("reset_filter", &self.reset_filter),
      ("priority-h", &self.priority_h),
      ("priority-m", &self.priority_m),
      ("priority-l", &self.priority_l),
      ("priority-n", &self.priority_n),
      ("priority-up", &self.priority_up),
      ("priority-down", &self.priority_down),
      ("shortcut0", &self.shortcut0),
      ("shortcut1", &self.shortcut1),
      ("shortcut2", &self.shortcut2),
      ("shortcut3", &self.shortcut3),
      ("shortcut4", &self.shortcut4),
      ("shortcut5", &self.shortcut5),
      ("shortcut6", &self.shortcut6),
      ("shortcut7", &self.shortcut7),
      ("shortcut8", &self.shortcut8),
      ("shortcut9", &self.shortcut9),
    ];
    let mut seen = std::collections::HashMap::new();
    for (name, key) in &elements {
      // Skip Nop entries — multiple bindings can all be disabled without conflict
      if **key == KeyCode::Nop {
        continue;
      }
      if let Some(existing) = seen.insert(key, name) {
        return Err(anyhow!(
          "Duplicate key binding: '{}' and '{}' are both mapped to the same key",
          existing,
          name
        ));
      }
    }
    Ok(())
  }

  fn get_config(config: &str, data: &str) -> Option<KeyCode> {
    for line in data.split('\n') {
      if line.starts_with(config) {
        let value = line.trim_start_matches(config).trim_start().trim_end();
        match parse_key_value(value) {
          Some(kc) => return Some(kc),
          None => error!("Invalid key value '{}' for {}", value, config),
        }
      } else if line.starts_with(&config.replace('-', "_")) {
        let value = line.trim_start_matches(&config.replace('-', "_")).trim_start().trim_end();
        match parse_key_value(value) {
          Some(kc) => return Some(kc),
          None => error!("Invalid key value '{}' for {}", value, config),
        }
      }
    }
    None
  }
}

/// Parse a key value string into a KeyCode.
///
/// Supports:
///   - Single character: `"q"` → `KeyCode::Char('q')`
///   - Angle bracket notation: `"<C-e>"` → `KeyCode::Ctrl('e')`
///
/// Returns `None` for empty strings, multi-character bare strings,
/// or unrecognized angle bracket tokens.
fn parse_key_value(s: &str) -> Option<KeyCode> {
  let s = s.trim();
  if s.starts_with('<') && s.ends_with('>') {
    parse_angle_bracket(&s[1..s.len() - 1])
  } else if has_just_one_char(s) {
    Some(KeyCode::Char(s.chars().next().unwrap()))
  } else {
    None
  }
}

/// Parse the inner content of an angle bracket key notation.
///
/// Handles modifier+key combinations (`C-e`, `A-d`, `M-f`, `S-Tab`)
/// and standalone named keys (`Esc`, `Enter`, `F1`, `Up`, etc.).
/// All matching is case-insensitive.
fn parse_angle_bracket(inner: &str) -> Option<KeyCode> {
  let inner_lower = inner.to_lowercase();

  // Try modifier+key: split on first '-' only (to handle e.g. "C-Backspace")
  if let Some(pos) = inner_lower.find('-') {
    let modifier = &inner_lower[..pos];
    let key = &inner_lower[pos + 1..];

    match modifier {
      "c" => {
        // Ctrl + single char
        if key.len() == 1 {
          return Some(KeyCode::Ctrl(key.chars().next().unwrap()));
        }
        // Ctrl + named special key
        return match key {
          "backspace" | "bs" => Some(KeyCode::CtrlBackspace),
          "delete" | "del" => Some(KeyCode::CtrlDelete),
          _ => None,
        };
      }
      "a" | "m" => {
        // Alt + single char
        if key.len() == 1 {
          return Some(KeyCode::Alt(key.chars().next().unwrap()));
        }
        // Alt + named special key
        return match key {
          "backspace" | "bs" => Some(KeyCode::AltBackspace),
          "delete" | "del" => Some(KeyCode::AltDelete),
          _ => None,
        };
      }
      "s" => {
        // Shift modifier — only supported for Tab
        return match key {
          "tab" => Some(KeyCode::BackTab),
          _ => None,
        };
      }
      _ => {}
    }
  }

  // No modifier matched — try standalone named key (using the lowercased full inner)
  match inner_lower.as_str() {
    "esc" | "escape" => Some(KeyCode::Esc),
    "enter" | "cr" | "return" => Some(KeyCode::Char('\n')),
    "tab" => Some(KeyCode::Tab),
    "backtab" => Some(KeyCode::BackTab),
    "bs" | "backspace" => Some(KeyCode::Backspace),
    "del" | "delete" => Some(KeyCode::Delete),
    "ins" | "insert" => Some(KeyCode::Insert),
    "space" => Some(KeyCode::Char(' ')),
    "up" => Some(KeyCode::Up),
    "down" => Some(KeyCode::Down),
    "left" => Some(KeyCode::Left),
    "right" => Some(KeyCode::Right),
    "pageup" | "pgup" => Some(KeyCode::PageUp),
    "pagedown" | "pgdn" => Some(KeyCode::PageDown),
    "home" => Some(KeyCode::Home),
    "end" => Some(KeyCode::End),
    "null" => Some(KeyCode::Null),
    "nop" => Some(KeyCode::Nop),
    _ => {
      // Function keys: f1 through f12
      if let Some(num_str) = inner_lower.strip_prefix('f') {
        if let Ok(n) = num_str.parse::<u8>() {
          if (1..=12).contains(&n) {
            return Some(KeyCode::F(n));
          }
        }
      }
      None
    }
  }
}

fn has_just_one_char(s: &str) -> bool {
  let mut chars = s.chars();
  chars.next().is_some() && chars.next().is_none()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_defaults_when_config_absent() {
    let data = "";
    let kc = KeyConfig::new(data).unwrap();
    assert_eq!(kc.help, KeyCode::Char('?'));
    assert_eq!(kc.priority_h, KeyCode::Char('H'));
    assert_eq!(kc.priority_m, KeyCode::Char('M'));
    assert_eq!(kc.priority_l, KeyCode::Char('L'));
    assert_eq!(kc.priority_n, KeyCode::Char('N'));
    assert_eq!(kc.priority_up, KeyCode::Char('+'));
    assert_eq!(kc.priority_down, KeyCode::Char('-'));
  }

  #[test]
  fn test_help_key_configurable() {
    let data = "uda.taskwarrior-tui.keyconfig.help h\n";
    let kc = KeyConfig::new(data).unwrap();
    assert_eq!(kc.help, KeyCode::Char('h'));
  }

  #[test]
  fn test_priority_keys_configurable() {
    let data = "uda.taskwarrior-tui.keyconfig.priority-h P\n\
                uda.taskwarrior-tui.keyconfig.priority-m Q\n\
                uda.taskwarrior-tui.keyconfig.priority-l R\n\
                uda.taskwarrior-tui.keyconfig.priority-n S\n";
    let kc = KeyConfig::new(data).unwrap();
    assert_eq!(kc.priority_h, KeyCode::Char('P'));
    assert_eq!(kc.priority_m, KeyCode::Char('Q'));
    assert_eq!(kc.priority_l, KeyCode::Char('R'));
    assert_eq!(kc.priority_n, KeyCode::Char('S'));
  }

  #[test]
  fn test_priority_up_down_configurable() {
    let data = "uda.taskwarrior-tui.keyconfig.priority-up <C-p>\n\
                uda.taskwarrior-tui.keyconfig.priority-down <C-n>\n";
    let kc = KeyConfig::new(data).unwrap();
    assert_eq!(kc.priority_up, KeyCode::Ctrl('p'));
    assert_eq!(kc.priority_down, KeyCode::Ctrl('n'));
  }

  #[test]
  fn test_existing_keys_still_work() {
    let data = "uda.taskwarrior-tui.keyconfig.quit Q\n\
                uda.taskwarrior-tui.keyconfig.add n\n";
    let kc = KeyConfig::new(data).unwrap();
    assert_eq!(kc.quit, KeyCode::Char('Q'));
    assert_eq!(kc.add, KeyCode::Char('n'));
    // unset keys keep defaults
    assert_eq!(kc.refresh, KeyCode::Char('r'));
    assert_eq!(kc.down, KeyCode::Char('j'));
  }

  // ── Single character (backward compat) ──────────────────────────────

  #[test]
  fn test_parse_single_char() {
    assert_eq!(parse_key_value("q"), Some(KeyCode::Char('q')));
    assert_eq!(parse_key_value("Q"), Some(KeyCode::Char('Q')));
    assert_eq!(parse_key_value("/"), Some(KeyCode::Char('/')));
    assert_eq!(parse_key_value("?"), Some(KeyCode::Char('?')));
    assert_eq!(parse_key_value("!"), Some(KeyCode::Char('!')));
    assert_eq!(parse_key_value("1"), Some(KeyCode::Char('1')));
  }

  #[test]
  fn test_parse_single_char_with_whitespace() {
    assert_eq!(parse_key_value("  q  "), Some(KeyCode::Char('q')));
    assert_eq!(parse_key_value(" / "), Some(KeyCode::Char('/')));
  }

  #[test]
  fn test_parse_empty_and_multi_char_bare() {
    assert_eq!(parse_key_value(""), None);
    assert_eq!(parse_key_value("ab"), None);
    assert_eq!(parse_key_value("quit"), None);
  }

  // ── Ctrl modifier ──────────────────────────────────────────────────

  #[test]
  fn test_parse_ctrl_char() {
    assert_eq!(parse_key_value("<C-e>"), Some(KeyCode::Ctrl('e')));
    assert_eq!(parse_key_value("<C-r>"), Some(KeyCode::Ctrl('r')));
    assert_eq!(parse_key_value("<C-y>"), Some(KeyCode::Ctrl('y')));
    assert_eq!(parse_key_value("<C-a>"), Some(KeyCode::Ctrl('a')));
  }

  #[test]
  fn test_parse_ctrl_case_insensitive() {
    // Modifier prefix case
    assert_eq!(parse_key_value("<c-e>"), Some(KeyCode::Ctrl('e')));
    assert_eq!(parse_key_value("<C-e>"), Some(KeyCode::Ctrl('e')));
    // Key char case — always lowercased
    assert_eq!(parse_key_value("<C-E>"), Some(KeyCode::Ctrl('e')));
    assert_eq!(parse_key_value("<c-E>"), Some(KeyCode::Ctrl('e')));
  }

  #[test]
  fn test_parse_ctrl_special() {
    assert_eq!(parse_key_value("<C-Backspace>"), Some(KeyCode::CtrlBackspace));
    assert_eq!(parse_key_value("<C-BS>"), Some(KeyCode::CtrlBackspace));
    assert_eq!(parse_key_value("<c-backspace>"), Some(KeyCode::CtrlBackspace));
    assert_eq!(parse_key_value("<C-Delete>"), Some(KeyCode::CtrlDelete));
    assert_eq!(parse_key_value("<C-Del>"), Some(KeyCode::CtrlDelete));
    assert_eq!(parse_key_value("<c-del>"), Some(KeyCode::CtrlDelete));
  }

  // ── Alt modifier ──────────────────────────────────────────────────

  #[test]
  fn test_parse_alt_char() {
    assert_eq!(parse_key_value("<A-d>"), Some(KeyCode::Alt('d')));
    assert_eq!(parse_key_value("<A-f>"), Some(KeyCode::Alt('f')));
    assert_eq!(parse_key_value("<A-b>"), Some(KeyCode::Alt('b')));
  }

  #[test]
  fn test_parse_meta_alias() {
    assert_eq!(parse_key_value("<M-f>"), Some(KeyCode::Alt('f')));
    assert_eq!(parse_key_value("<M-d>"), Some(KeyCode::Alt('d')));
    assert_eq!(parse_key_value("<m-f>"), Some(KeyCode::Alt('f')));
  }

  #[test]
  fn test_parse_alt_case_insensitive() {
    assert_eq!(parse_key_value("<a-d>"), Some(KeyCode::Alt('d')));
    assert_eq!(parse_key_value("<A-D>"), Some(KeyCode::Alt('d')));
  }

  #[test]
  fn test_parse_alt_special() {
    assert_eq!(parse_key_value("<A-Backspace>"), Some(KeyCode::AltBackspace));
    assert_eq!(parse_key_value("<A-BS>"), Some(KeyCode::AltBackspace));
    assert_eq!(parse_key_value("<M-Backspace>"), Some(KeyCode::AltBackspace));
    assert_eq!(parse_key_value("<A-Delete>"), Some(KeyCode::AltDelete));
    assert_eq!(parse_key_value("<A-Del>"), Some(KeyCode::AltDelete));
    assert_eq!(parse_key_value("<M-Del>"), Some(KeyCode::AltDelete));
  }

  // ── Shift modifier ────────────────────────────────────────────────

  #[test]
  fn test_parse_shift_tab() {
    assert_eq!(parse_key_value("<S-Tab>"), Some(KeyCode::BackTab));
    assert_eq!(parse_key_value("<s-tab>"), Some(KeyCode::BackTab));
  }

  #[test]
  fn test_parse_shift_unsupported() {
    // Shift is only supported for Tab
    assert_eq!(parse_key_value("<S-a>"), None);
    assert_eq!(parse_key_value("<S-Enter>"), None);
  }

  // ── Named keys (no modifier) ──────────────────────────────────────

  #[test]
  fn test_parse_esc() {
    assert_eq!(parse_key_value("<Esc>"), Some(KeyCode::Esc));
    assert_eq!(parse_key_value("<Escape>"), Some(KeyCode::Esc));
    assert_eq!(parse_key_value("<esc>"), Some(KeyCode::Esc));
    assert_eq!(parse_key_value("<ESC>"), Some(KeyCode::Esc));
    assert_eq!(parse_key_value("<ESCAPE>"), Some(KeyCode::Esc));
  }

  #[test]
  fn test_parse_enter() {
    assert_eq!(parse_key_value("<Enter>"), Some(KeyCode::Char('\n')));
    assert_eq!(parse_key_value("<CR>"), Some(KeyCode::Char('\n')));
    assert_eq!(parse_key_value("<Return>"), Some(KeyCode::Char('\n')));
    assert_eq!(parse_key_value("<enter>"), Some(KeyCode::Char('\n')));
    assert_eq!(parse_key_value("<cr>"), Some(KeyCode::Char('\n')));
  }

  #[test]
  fn test_parse_tab() {
    assert_eq!(parse_key_value("<Tab>"), Some(KeyCode::Tab));
    assert_eq!(parse_key_value("<tab>"), Some(KeyCode::Tab));
  }

  #[test]
  fn test_parse_backtab() {
    assert_eq!(parse_key_value("<BackTab>"), Some(KeyCode::BackTab));
    assert_eq!(parse_key_value("<backtab>"), Some(KeyCode::BackTab));
  }

  #[test]
  fn test_parse_backspace() {
    assert_eq!(parse_key_value("<BS>"), Some(KeyCode::Backspace));
    assert_eq!(parse_key_value("<Backspace>"), Some(KeyCode::Backspace));
    assert_eq!(parse_key_value("<bs>"), Some(KeyCode::Backspace));
  }

  #[test]
  fn test_parse_delete() {
    assert_eq!(parse_key_value("<Del>"), Some(KeyCode::Delete));
    assert_eq!(parse_key_value("<Delete>"), Some(KeyCode::Delete));
    assert_eq!(parse_key_value("<del>"), Some(KeyCode::Delete));
  }

  #[test]
  fn test_parse_insert() {
    assert_eq!(parse_key_value("<Ins>"), Some(KeyCode::Insert));
    assert_eq!(parse_key_value("<Insert>"), Some(KeyCode::Insert));
    assert_eq!(parse_key_value("<ins>"), Some(KeyCode::Insert));
  }

  #[test]
  fn test_parse_space() {
    assert_eq!(parse_key_value("<Space>"), Some(KeyCode::Char(' ')));
    assert_eq!(parse_key_value("<space>"), Some(KeyCode::Char(' ')));
  }

  // ── Arrow keys ────────────────────────────────────────────────────

  #[test]
  fn test_parse_arrows() {
    assert_eq!(parse_key_value("<Up>"), Some(KeyCode::Up));
    assert_eq!(parse_key_value("<Down>"), Some(KeyCode::Down));
    assert_eq!(parse_key_value("<Left>"), Some(KeyCode::Left));
    assert_eq!(parse_key_value("<Right>"), Some(KeyCode::Right));
    assert_eq!(parse_key_value("<up>"), Some(KeyCode::Up));
    assert_eq!(parse_key_value("<DOWN>"), Some(KeyCode::Down));
  }

  // ── Navigation keys ───────────────────────────────────────────────

  #[test]
  fn test_parse_page_keys() {
    assert_eq!(parse_key_value("<PageUp>"), Some(KeyCode::PageUp));
    assert_eq!(parse_key_value("<PgUp>"), Some(KeyCode::PageUp));
    assert_eq!(parse_key_value("<pageup>"), Some(KeyCode::PageUp));
    assert_eq!(parse_key_value("<PageDown>"), Some(KeyCode::PageDown));
    assert_eq!(parse_key_value("<PgDn>"), Some(KeyCode::PageDown));
    assert_eq!(parse_key_value("<pagedown>"), Some(KeyCode::PageDown));
    assert_eq!(parse_key_value("<pgdn>"), Some(KeyCode::PageDown));
  }

  #[test]
  fn test_parse_home_end() {
    assert_eq!(parse_key_value("<Home>"), Some(KeyCode::Home));
    assert_eq!(parse_key_value("<End>"), Some(KeyCode::End));
    assert_eq!(parse_key_value("<home>"), Some(KeyCode::Home));
    assert_eq!(parse_key_value("<end>"), Some(KeyCode::End));
  }

  // ── Function keys ─────────────────────────────────────────────────

  #[test]
  fn test_parse_function_keys() {
    assert_eq!(parse_key_value("<F1>"), Some(KeyCode::F(1)));
    assert_eq!(parse_key_value("<F2>"), Some(KeyCode::F(2)));
    assert_eq!(parse_key_value("<F10>"), Some(KeyCode::F(10)));
    assert_eq!(parse_key_value("<F12>"), Some(KeyCode::F(12)));
    assert_eq!(parse_key_value("<f1>"), Some(KeyCode::F(1)));
    assert_eq!(parse_key_value("<f12>"), Some(KeyCode::F(12)));
  }

  #[test]
  fn test_parse_function_keys_out_of_range() {
    assert_eq!(parse_key_value("<F0>"), None);
    assert_eq!(parse_key_value("<F13>"), None);
    assert_eq!(parse_key_value("<F99>"), None);
  }

  // ── Null ──────────────────────────────────────────────────────────

  #[test]
  fn test_parse_null() {
    assert_eq!(parse_key_value("<Null>"), Some(KeyCode::Null));
    assert_eq!(parse_key_value("<null>"), Some(KeyCode::Null));
  }

  // ── Invalid input ─────────────────────────────────────────────────

  #[test]
  fn test_parse_invalid_angle_bracket() {
    assert_eq!(parse_key_value("<>"), None);
    assert_eq!(parse_key_value("<FooBar>"), None);
    assert_eq!(parse_key_value("<C->"), None);
    assert_eq!(parse_key_value("<C-ab>"), None); // multi-char after Ctrl
    assert_eq!(parse_key_value("<X-a>"), None); // unknown modifier
  }

  #[test]
  fn test_parse_malformed_brackets() {
    assert_eq!(parse_key_value("<C-e"), None); // missing closing >
    assert_eq!(parse_key_value("C-e>"), None); // missing opening <
    assert_eq!(parse_key_value("Esc"), None); // bare multi-char without brackets
  }

  // ── Integration: get_config with angle bracket values ─────────────

  #[test]
  fn test_get_config_angle_bracket_ctrl() {
    let data = "uda.taskwarrior-tui.keyconfig.quit <C-q>\n";
    let result = KeyConfig::get_config("uda.taskwarrior-tui.keyconfig.quit", data);
    assert_eq!(result, Some(KeyCode::Ctrl('q')));
  }

  #[test]
  fn test_get_config_angle_bracket_esc() {
    let data = "uda.taskwarrior-tui.keyconfig.quit <Esc>\n";
    let result = KeyConfig::get_config("uda.taskwarrior-tui.keyconfig.quit", data);
    assert_eq!(result, Some(KeyCode::Esc));
  }

  #[test]
  fn test_get_config_angle_bracket_f_key() {
    let data = "uda.taskwarrior-tui.keyconfig.help <F1>\n";
    let result = KeyConfig::get_config("uda.taskwarrior-tui.keyconfig.help", data);
    assert_eq!(result, Some(KeyCode::F(1)));
  }

  #[test]
  fn test_get_config_angle_bracket_enter() {
    let data = "uda.taskwarrior-tui.keyconfig.done <Enter>\n";
    let result = KeyConfig::get_config("uda.taskwarrior-tui.keyconfig.done", data);
    assert_eq!(result, Some(KeyCode::Char('\n')));
  }

  #[test]
  fn test_get_config_single_char_backward_compat() {
    let data = "uda.taskwarrior-tui.keyconfig.quit Q\n";
    let result = KeyConfig::get_config("uda.taskwarrior-tui.keyconfig.quit", data);
    assert_eq!(result, Some(KeyCode::Char('Q')));
  }

  #[test]
  fn test_get_config_underscore_fallback_with_angle_bracket() {
    let data = "uda.taskwarrior_tui.keyconfig.page_down <PageDown>\n";
    let result = KeyConfig::get_config("uda.taskwarrior-tui.keyconfig.page-down", data);
    assert_eq!(result, Some(KeyCode::PageDown));
  }

  #[test]
  fn test_get_config_absent() {
    let data = "uda.taskwarrior-tui.keyconfig.quit q\n";
    let result = KeyConfig::get_config("uda.taskwarrior-tui.keyconfig.help", data);
    assert_eq!(result, None);
  }

  // ── KeyConfig::new integration ────────────────────────────────────

  #[test]
  fn test_keyconfig_new_with_angle_brackets() {
    let data = "uda.taskwarrior-tui.keyconfig.quit <Esc>\n\
                uda.taskwarrior-tui.keyconfig.filter <C-f>\n\
                uda.taskwarrior-tui.keyconfig.add <F2>\n";
    let kc = KeyConfig::new(data).unwrap();
    assert_eq!(kc.quit, KeyCode::Esc);
    assert_eq!(kc.filter, KeyCode::Ctrl('f'));
    assert_eq!(kc.add, KeyCode::F(2));
    // Unset keys keep defaults
    assert_eq!(kc.down, KeyCode::Char('j'));
    assert_eq!(kc.refresh, KeyCode::Char('r'));
  }

  #[test]
  fn test_keyconfig_new_defaults_unchanged() {
    let data = "";
    let kc = KeyConfig::new(data).unwrap();
    assert_eq!(kc.quit, KeyCode::Char('q'));
    assert_eq!(kc.help, KeyCode::Char('?'));
    assert_eq!(kc.filter, KeyCode::Char('/'));
    assert_eq!(kc.down, KeyCode::Char('j'));
    assert_eq!(kc.up, KeyCode::Char('k'));
    assert_eq!(kc.scroll_down, KeyCode::Ctrl('e'));
    assert_eq!(kc.scroll_up, KeyCode::Ctrl('y'));
    assert_eq!(kc.jump, KeyCode::Char(':'));
    assert_eq!(kc.reset_filter, KeyCode::Ctrl('r'));
  }

  #[test]
  fn test_keyconfig_mixed_single_and_angle() {
    let data = "uda.taskwarrior-tui.keyconfig.quit Q\n\
                uda.taskwarrior-tui.keyconfig.add <C-a>\n";
    let kc = KeyConfig::new(data).unwrap();
    assert_eq!(kc.quit, KeyCode::Char('Q'));
    assert_eq!(kc.add, KeyCode::Ctrl('a'));
  }

  #[test]
  fn test_keyconfig_scroll_down_configurable() {
    let data = "uda.taskwarrior-tui.keyconfig.scroll-down <C-d>\n";
    let kc = KeyConfig::new(data).unwrap();
    assert_eq!(kc.scroll_down, KeyCode::Ctrl('d'));
  }

  #[test]
  fn test_keyconfig_scroll_up_configurable() {
    let data = "uda.taskwarrior-tui.keyconfig.scroll-up <C-u>\n";
    let kc = KeyConfig::new(data).unwrap();
    assert_eq!(kc.scroll_up, KeyCode::Ctrl('u'));
  }

  #[test]
  fn test_keyconfig_jump_configurable() {
    let data = "uda.taskwarrior-tui.keyconfig.jump <C-j>\n";
    let kc = KeyConfig::new(data).unwrap();
    assert_eq!(kc.jump, KeyCode::Ctrl('j'));
  }

  #[test]
  fn test_keyconfig_reset_filter_configurable() {
    let data = "uda.taskwarrior-tui.keyconfig.reset-filter <C-x>\n";
    let kc = KeyConfig::new(data).unwrap();
    assert_eq!(kc.reset_filter, KeyCode::Ctrl('x'));
  }

  #[test]
  fn test_check_defaults_no_duplicates() {
    let kc = KeyConfig::new("").unwrap();
    assert!(kc.check().is_ok());
  }

  #[test]
  fn test_check_detects_priority_shortcut_conflict() {
    // priority_h defaults to 'H', shortcut0 defaults to '0'
    // Setting priority-h to '0' should conflict with shortcut0
    let data = "uda.taskwarrior-tui.keyconfig.priority-h 0\n";
    let result = KeyConfig::new(data);
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("priority-h"), "Error should mention priority-h: {}", msg);
    assert!(msg.contains("shortcut0"), "Error should mention shortcut0: {}", msg);
  }

  #[test]
  fn test_check_detects_nonadjacent_duplicates() {
    // Set shortcut9 (normally '9') to 'q' which conflicts with quit (normally 'q')
    // These are far apart in the check() vector, testing non-adjacent detection
    let data = "uda.taskwarrior-tui.keyconfig.shortcut9 q\n";
    let result = KeyConfig::new(data);
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("quit"), "Error should mention quit: {}", msg);
    assert!(msg.contains("shortcut9"), "Error should mention shortcut9: {}", msg);
  }

  // ── <Nop> support tests ──────────────────────────────────────────────

  #[test]
  fn test_parse_nop_angle_bracket() {
    assert_eq!(parse_key_value("<Nop>"), Some(KeyCode::Nop));
  }

  #[test]
  fn test_parse_nop_case_insensitive() {
    assert_eq!(parse_key_value("<nop>"), Some(KeyCode::Nop));
    assert_eq!(parse_key_value("<NOP>"), Some(KeyCode::Nop));
    assert_eq!(parse_key_value("<NoP>"), Some(KeyCode::Nop));
  }

  #[test]
  fn test_nop_config_parsing() {
    let data = "uda.taskwarrior-tui.keyconfig.shell <Nop>\n";
    let kc = KeyConfig::new(data).unwrap();
    assert_eq!(kc.shell, KeyCode::Nop);
  }

  #[test]
  fn test_nop_multiple_no_duplicate_conflict() {
    // Setting multiple bindings to <Nop> should not trigger duplicate detection
    let data = "\
      uda.taskwarrior-tui.keyconfig.shell <Nop>\n\
      uda.taskwarrior-tui.keyconfig.log <Nop>\n\
      uda.taskwarrior-tui.keyconfig.annotate <Nop>\n";
    let kc = KeyConfig::new(data).unwrap();
    assert_eq!(kc.shell, KeyCode::Nop);
    assert_eq!(kc.log, KeyCode::Nop);
    assert_eq!(kc.annotate, KeyCode::Nop);
    assert!(kc.check().is_ok());
  }

  #[test]
  fn test_nop_does_not_match_real_input() {
    // Nop should not equal any key that can come from actual keyboard input
    assert_ne!(KeyCode::Nop, KeyCode::Null);
    assert_ne!(KeyCode::Nop, KeyCode::Char(' '));
    assert_ne!(KeyCode::Nop, KeyCode::Esc);
    assert_ne!(KeyCode::Nop, KeyCode::Tab);
  }
}
