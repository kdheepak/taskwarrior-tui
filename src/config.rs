use figment::{
  providers::{Env, Format, Serialized, Toml},
  Figment,
};
use ratatui::{
  style::{Color, Modifier, Style},
  symbols::line::DOUBLE_VERTICAL,
};
use std::{collections::HashMap, error::Error, path::PathBuf, str};

use color_eyre::eyre::{eyre, Context, Result};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::de::{self, Deserialize, Deserializer, MapAccess, Visitor};
use serde::ser::{self, Serialize, Serializer};
use serde_derive::{Deserialize, Serialize};

use crate::{action::Action, keyevent::parse_key_sequence, keymap::KeyMap, utils::get_config_dir};

#[derive(Default, Clone, Debug)]
pub struct SerdeStyle(Style);

impl<'de> Deserialize<'de> for SerdeStyle {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    struct StyleVisitor;

    impl<'de> Visitor<'de> for StyleVisitor {
      type Value = SerdeStyle;

      fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string representation of tui::style::Style")
      }

      fn visit_str<E: de::Error>(self, v: &str) -> Result<SerdeStyle, E> {
        Ok(SerdeStyle(get_tcolor(v)))
      }
    }

    deserializer.deserialize_str(StyleVisitor)
  }
}

pub fn get_tcolor(line: &str) -> Style {
  let (foreground, background) = line.split_at(line.to_lowercase().find("on ").unwrap_or(line.len()));
  let (mut foreground, mut background) = (String::from(foreground), String::from(background));
  background = background.replace("on ", "");
  let mut modifiers = Modifier::empty();
  if foreground.contains("bright") {
    foreground = foreground.replace("bright ", "");
    background = background.replace("bright ", "");
    background.insert_str(0, "bright ");
  }
  foreground = foreground.replace("grey", "gray");
  background = background.replace("grey", "gray");
  if foreground.contains("underline") {
    modifiers |= Modifier::UNDERLINED;
  }
  let foreground = foreground.replace("underline ", "");
  // TODO: use bold, bright boolean flags
  if foreground.contains("bold") {
    modifiers |= Modifier::BOLD;
  }
  // let foreground = foreground.replace("bold ", "");
  if foreground.contains("inverse") {
    modifiers |= Modifier::REVERSED;
  }
  let foreground = foreground.replace("inverse ", "");
  let mut style = Style::default();
  if let Some(fg) = get_color_foreground(foreground.as_str()) {
    style = style.fg(fg);
  }
  if let Some(bg) = get_color_background(background.as_str()) {
    style = style.bg(bg);
  }
  style = style.add_modifier(modifiers);
  style
}

fn get_color_foreground(s: &str) -> Option<Color> {
  let s = s.trim_start();
  let s = s.trim_end();
  if s.contains("color") {
    let c = s.trim_start_matches("color").parse::<u8>().unwrap_or_default();
    Some(Color::Indexed(c))
  } else if s.contains("gray") {
    let c = 232 + s.trim_start_matches("gray").parse::<u8>().unwrap_or_default();
    Some(Color::Indexed(c))
  } else if s.contains("rgb") {
    let red = (s.as_bytes()[3] as char).to_digit(10).unwrap_or_default() as u8;
    let green = (s.as_bytes()[4] as char).to_digit(10).unwrap_or_default() as u8;
    let blue = (s.as_bytes()[5] as char).to_digit(10).unwrap_or_default() as u8;
    let c = 16 + red * 36 + green * 6 + blue;
    Some(Color::Indexed(c))
  } else if s == "bold black" {
    Some(Color::Indexed(8))
  } else if s == "bold red" {
    Some(Color::Indexed(9))
  } else if s == "bold green" {
    Some(Color::Indexed(10))
  } else if s == "bold yellow" {
    Some(Color::Indexed(11))
  } else if s == "bold blue" {
    Some(Color::Indexed(12))
  } else if s == "bold magenta" {
    Some(Color::Indexed(13))
  } else if s == "bold cyan" {
    Some(Color::Indexed(14))
  } else if s == "bold white" {
    Some(Color::Indexed(15))
  } else if s == "black" {
    Some(Color::Indexed(0))
  } else if s == "red" {
    Some(Color::Indexed(1))
  } else if s == "green" {
    Some(Color::Indexed(2))
  } else if s == "yellow" {
    Some(Color::Indexed(3))
  } else if s == "blue" {
    Some(Color::Indexed(4))
  } else if s == "magenta" {
    Some(Color::Indexed(5))
  } else if s == "cyan" {
    Some(Color::Indexed(6))
  } else if s == "white" {
    Some(Color::Indexed(7))
  } else {
    None
  }
}

fn get_color_background(s: &str) -> Option<Color> {
  let s = s.trim_start();
  let s = s.trim_end();
  if s.contains("bright color") {
    let s = s.trim_start_matches("bright ");
    let c = s.trim_start_matches("color").parse::<u8>().unwrap_or_default();
    Some(Color::Indexed(c.wrapping_shl(8)))
  } else if s.contains("color") {
    let c = s.trim_start_matches("color").parse::<u8>().unwrap_or_default();
    Some(Color::Indexed(c))
  } else if s.contains("gray") {
    let s = s.trim_start_matches("bright ");
    let c = 232 + s.trim_start_matches("gray").parse::<u8>().unwrap_or_default();
    Some(Color::Indexed(c.wrapping_shl(8)))
  } else if s.contains("rgb") {
    let s = s.trim_start_matches("bright ");
    let red = (s.as_bytes()[3] as char).to_digit(10).unwrap_or_default() as u8;
    let green = (s.as_bytes()[4] as char).to_digit(10).unwrap_or_default() as u8;
    let blue = (s.as_bytes()[5] as char).to_digit(10).unwrap_or_default() as u8;
    let c = 16 + red * 36 + green * 6 + blue;
    Some(Color::Indexed(c.wrapping_shl(8)))
  } else if s == "bright black" {
    Some(Color::Indexed(8))
  } else if s == "bright red" {
    Some(Color::Indexed(9))
  } else if s == "bright green" {
    Some(Color::Indexed(10))
  } else if s == "bright yellow" {
    Some(Color::Indexed(11))
  } else if s == "bright blue" {
    Some(Color::Indexed(12))
  } else if s == "bright magenta" {
    Some(Color::Indexed(13))
  } else if s == "bright cyan" {
    Some(Color::Indexed(14))
  } else if s == "bright white" {
    Some(Color::Indexed(15))
  } else if s == "black" {
    Some(Color::Indexed(0))
  } else if s == "red" {
    Some(Color::Indexed(1))
  } else if s == "green" {
    Some(Color::Indexed(2))
  } else if s == "yellow" {
    Some(Color::Indexed(3))
  } else if s == "blue" {
    Some(Color::Indexed(4))
  } else if s == "magenta" {
    Some(Color::Indexed(5))
  } else if s == "cyan" {
    Some(Color::Indexed(6))
  } else if s == "white" {
    Some(Color::Indexed(7))
  } else {
    None
  }
}

impl Serialize for SerdeStyle {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    // Getting the foreground color string
    let fg_str = color_to_string(self.0.fg.unwrap());

    // Getting the background color string
    let mut bg_str = color_to_string(self.0.bg.unwrap());

    // If the background is not default, prepend with "on "
    if bg_str != "" {
      bg_str.insert_str(0, "on ");
    }

    // Building the modifier string
    let mut mod_str = String::new();
    let mod_val = self.0.add_modifier;
    if mod_val.contains(Modifier::BOLD) {
      mod_str.push_str("bold ");
    }
    if mod_val.contains(Modifier::UNDERLINED) {
      mod_str.push_str("underline ");
    }
    if mod_val.contains(Modifier::REVERSED) {
      mod_str.push_str("inverse ");
    }

    // Constructing the final style string
    let style_str = format!("{}{} {}", mod_str, fg_str, bg_str).trim().to_string();

    serializer.serialize_str(&style_str)
  }
}

fn color_to_string(color: Color) -> String {
  match color {
    Color::Black => "black".to_string(),
    Color::Red => "red".to_string(),
    Color::Green => "green".to_string(),
    // ... handle all other colors ...
    _ => "".to_string(), // Default case, adjust as needed
  }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
  pub tick_rate: usize,
  pub keymap: HashMap<String, KeyMap>,
  pub enabled: bool,
  pub color: HashMap<String, SerdeStyle>,
  pub filter: String,
  pub data_location: String,
  pub obfuscate: bool,
  pub print_empty_columns: bool,
  pub due: usize,
  pub weekstart: bool,
  pub rule_precedence_color: Vec<String>,
  pub uda_priority_values: Vec<String>,
  pub uda_tick_rate: u64,
  pub uda_auto_insert_double_quotes_on_add: bool,
  pub uda_auto_insert_double_quotes_on_annotate: bool,
  pub uda_auto_insert_double_quotes_on_log: bool,
  pub uda_prefill_task_metadata: bool,
  pub uda_reset_filter_on_esc: bool,
  pub uda_task_detail_prefetch: usize,
  pub uda_task_report_use_all_tasks_for_completion: bool,
  pub uda_task_report_show_info: bool,
  pub uda_task_report_looping: bool,
  pub uda_task_report_jump_to_task_on_add: bool,
  pub uda_selection_indicator: String,
  pub uda_mark_indicator: String,
  pub uda_unmark_indicator: String,
  pub uda_scrollbar_indicator: String,
  pub uda_scrollbar_area: String,
  pub uda_style_report_scrollbar: SerdeStyle,
  pub uda_style_report_scrollbar_area: SerdeStyle,
  pub uda_selection_bold: bool,
  pub uda_selection_italic: bool,
  pub uda_selection_dim: bool,
  pub uda_selection_blink: bool,
  pub uda_selection_reverse: bool,
  pub uda_calendar_months_per_row: usize,
  pub uda_style_context_active: SerdeStyle,
  pub uda_style_report_selection: SerdeStyle,
  pub uda_style_calendar_title: SerdeStyle,
  pub uda_style_calendar_today: SerdeStyle,
  pub uda_style_navbar: SerdeStyle,
  pub uda_style_command: SerdeStyle,
  pub uda_style_report_completion_pane: SerdeStyle,
  pub uda_style_report_completion_pane_highlight: SerdeStyle,
  pub uda_shortcuts: Vec<String>,
  pub uda_change_focus_rotate: bool,
  pub uda_background_process: String,
  pub uda_background_process_period: usize,
  pub uda_quick_tag_name: String,
  pub uda_task_report_prompt_on_undo: bool,
  pub uda_task_report_prompt_on_delete: bool,
  pub uda_task_report_prompt_on_done: bool,
  pub uda_task_report_date_time_vague_more_precise: bool,
  pub uda_context_menu_select_on_move: bool,
}

impl Default for Config {
  fn default() -> Self {
    let tick_rate = 250;

    let mut task_report_keymap: KeyMap = Default::default();
    task_report_keymap.insert(parse_key_sequence("q").unwrap(), Action::Quit);
    task_report_keymap.insert(parse_key_sequence("r").unwrap(), Action::Refresh);
    task_report_keymap.insert(parse_key_sequence("G").unwrap(), Action::GotoBottom);
    task_report_keymap.insert(parse_key_sequence("<g><g>").unwrap(), Action::GotoTop);
    task_report_keymap.insert(parse_key_sequence("<g><j>").unwrap(), Action::GotoPageBottom);
    task_report_keymap.insert(parse_key_sequence("<g><k>").unwrap(), Action::GotoPageTop);
    task_report_keymap.insert(parse_key_sequence("<g><G>").unwrap(), Action::GotoBottom);
    task_report_keymap.insert(parse_key_sequence("j").unwrap(), Action::Down);
    task_report_keymap.insert(parse_key_sequence("k").unwrap(), Action::Up);
    task_report_keymap.insert(parse_key_sequence("J").unwrap(), Action::PageDown);
    task_report_keymap.insert(parse_key_sequence("K").unwrap(), Action::PageUp);
    task_report_keymap.insert(parse_key_sequence("<d><d>").unwrap(), Action::Delete);
    task_report_keymap.insert(parse_key_sequence("<x><x>").unwrap(), Action::Done);
    task_report_keymap.insert(parse_key_sequence("s").unwrap(), Action::ToggleStartStop);
    task_report_keymap.insert(parse_key_sequence("t").unwrap(), Action::QuickTag);
    task_report_keymap.insert(parse_key_sequence("v").unwrap(), Action::Select);
    task_report_keymap.insert(parse_key_sequence("V").unwrap(), Action::SelectAll);
    task_report_keymap.insert(parse_key_sequence("u").unwrap(), Action::Undo);
    task_report_keymap.insert(parse_key_sequence("e").unwrap(), Action::Edit);
    task_report_keymap.insert(parse_key_sequence("m").unwrap(), Action::Modify);
    task_report_keymap.insert(parse_key_sequence("!").unwrap(), Action::Shell);
    task_report_keymap.insert(parse_key_sequence("l").unwrap(), Action::Log);
    task_report_keymap.insert(parse_key_sequence("a").unwrap(), Action::Add);
    task_report_keymap.insert(parse_key_sequence("A").unwrap(), Action::Annotate);
    task_report_keymap.insert(parse_key_sequence("?").unwrap(), Action::Help);
    task_report_keymap.insert(parse_key_sequence("/").unwrap(), Action::Filter);
    task_report_keymap.insert(parse_key_sequence("z").unwrap(), Action::ToggleZoom);
    task_report_keymap.insert(parse_key_sequence("c").unwrap(), Action::Context);
    task_report_keymap.insert(parse_key_sequence("]").unwrap(), Action::Next);
    task_report_keymap.insert(parse_key_sequence("[").unwrap(), Action::Previous);
    task_report_keymap.insert(parse_key_sequence("1").unwrap(), Action::Shortcut(1));
    task_report_keymap.insert(parse_key_sequence("2").unwrap(), Action::Shortcut(2));
    task_report_keymap.insert(parse_key_sequence("3").unwrap(), Action::Shortcut(3));
    task_report_keymap.insert(parse_key_sequence("4").unwrap(), Action::Shortcut(4));
    task_report_keymap.insert(parse_key_sequence("5").unwrap(), Action::Shortcut(5));
    task_report_keymap.insert(parse_key_sequence("6").unwrap(), Action::Shortcut(6));
    task_report_keymap.insert(parse_key_sequence("7").unwrap(), Action::Shortcut(7));
    task_report_keymap.insert(parse_key_sequence("8").unwrap(), Action::Shortcut(8));
    task_report_keymap.insert(parse_key_sequence("9").unwrap(), Action::Shortcut(9));

    let mut keymap: HashMap<String, KeyMap> = Default::default();
    keymap.insert("task-report".into(), task_report_keymap);

    let enabled = true;
    let color = Default::default();
    let filter = Default::default();
    let data_location = Default::default();
    let obfuscate = false;
    let print_empty_columns = false;
    let due = 7; // due 7 days
    let weekstart = true; // starts on monday
    let rule_precedence_color = Default::default();
    let uda_priority_values = Default::default();
    let uda_tick_rate = 250;
    let uda_auto_insert_double_quotes_on_add = true;
    let uda_auto_insert_double_quotes_on_annotate = true;
    let uda_auto_insert_double_quotes_on_log = true;
    let uda_prefill_task_metadata = Default::default();
    let uda_reset_filter_on_esc = true;
    let uda_task_detail_prefetch = 10;
    let uda_task_report_use_all_tasks_for_completion = Default::default();
    let uda_task_report_show_info = true;
    let uda_task_report_looping = true;
    let uda_task_report_jump_to_task_on_add = true;
    let uda_selection_indicator = "\u{2022} ".to_string();
    let uda_mark_indicator = "\u{2714} ".to_string();
    let uda_unmark_indicator = "  ".to_string();
    let uda_scrollbar_indicator = "█".to_string();
    let uda_scrollbar_area = "║".to_string();
    let uda_style_report_scrollbar = Default::default();
    let uda_style_report_scrollbar_area = Default::default();
    let uda_selection_bold = true;
    let uda_selection_italic = Default::default();
    let uda_selection_dim = Default::default();
    let uda_selection_blink = Default::default();
    let uda_selection_reverse = Default::default();
    let uda_calendar_months_per_row = 4;
    let uda_style_context_active = Default::default();
    let uda_style_report_selection = Default::default();
    let uda_style_calendar_title = Default::default();
    let uda_style_calendar_today = Default::default();
    let uda_style_navbar = Default::default();
    let uda_style_command = Default::default();
    let uda_style_report_completion_pane = Default::default();
    let uda_style_report_completion_pane_highlight = Default::default();
    let uda_shortcuts = Default::default();
    let uda_change_focus_rotate = Default::default();
    let uda_background_process = Default::default();
    let uda_background_process_period = Default::default();
    let uda_quick_tag_name = Default::default();
    let uda_task_report_prompt_on_undo = Default::default();
    let uda_task_report_prompt_on_delete = Default::default();
    let uda_task_report_prompt_on_done = Default::default();
    let uda_task_report_date_time_vague_more_precise = Default::default();
    let uda_context_menu_select_on_move = Default::default();

    Self {
      tick_rate,
      keymap,
      enabled,
      color,
      filter,
      data_location,
      obfuscate,
      print_empty_columns,
      due,
      weekstart,
      rule_precedence_color,
      uda_priority_values,
      uda_tick_rate,
      uda_auto_insert_double_quotes_on_add,
      uda_auto_insert_double_quotes_on_annotate,
      uda_auto_insert_double_quotes_on_log,
      uda_prefill_task_metadata,
      uda_reset_filter_on_esc,
      uda_task_detail_prefetch,
      uda_task_report_use_all_tasks_for_completion,
      uda_task_report_show_info,
      uda_task_report_looping,
      uda_task_report_jump_to_task_on_add,
      uda_selection_indicator,
      uda_mark_indicator,
      uda_unmark_indicator,
      uda_scrollbar_indicator,
      uda_scrollbar_area,
      uda_style_report_scrollbar,
      uda_style_report_scrollbar_area,
      uda_selection_bold,
      uda_selection_italic,
      uda_selection_dim,
      uda_selection_blink,
      uda_selection_reverse,
      uda_calendar_months_per_row,
      uda_style_context_active,
      uda_style_report_selection,
      uda_style_calendar_title,
      uda_style_calendar_today,
      uda_style_navbar,
      uda_style_command,
      uda_style_report_completion_pane,
      uda_style_report_completion_pane_highlight,
      uda_shortcuts,
      uda_change_focus_rotate,
      uda_background_process,
      uda_background_process_period,
      uda_quick_tag_name,
      uda_task_report_prompt_on_undo,
      uda_task_report_prompt_on_delete,
      uda_task_report_prompt_on_done,
      uda_task_report_date_time_vague_more_precise,
      uda_context_menu_select_on_move,
    }
  }
}

impl Config {
  pub fn new() -> Result<Self> {
    let config: Self = Figment::from(Serialized::defaults(Config::default()))
      .merge(Toml::file(get_config_dir().join("config.toml")))
      .merge(Env::prefixed("TASKWARRIOR_TUI_"))
      .extract()?;
    Ok(config)
  }

  pub fn write(&self, path: PathBuf) -> Result<()> {
    let content = toml::to_string(&self)?;
    std::fs::write(&path, content)?;
    std::fs::File::open(&path)?.sync_data()?;
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_read_config() {
    let config = Config::new().unwrap();
    dbg!(config);
  }

  // #[test]
  // fn test_write_config() {
  //   let config: Config = Default::default();
  //   config.write("tests/data/test.toml".into()).unwrap();
  // }
}
