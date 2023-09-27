use std::{collections::HashMap, fmt, path::PathBuf};

use color_eyre::eyre::Result;
use config::Value;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use derive_deref::{Deref, DerefMut};
use ratatui::style::{Color, Modifier, Style};
use serde::de::{self, Deserialize, Deserializer, MapAccess, Visitor};
use serde_derive::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::{action::Action, app::Mode};

const CONFIG: &str = include_str!("../.config/config.json5");

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct TaskwarriorConfig {
  #[serde(default)]
  pub rule_precedence_color: Vec<String>,
  #[serde(default)]
  pub uda_priority_values: Vec<String>,
  #[serde(default)]
  pub weekstart: bool,
  #[serde(default)]
  pub due: usize,
  #[serde(default)]
  pub color: HashMap<String, Style>,
  #[serde(default)]
  pub data_location: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct TaskReportConfig {
  #[serde(default)]
  pub looping: bool,
  #[serde(default)]
  pub selection_indicator: String,
}

impl Into<Value> for TaskReportConfig {
  fn into(self) -> Value {
    let json_value = serde_json::to_value(self).unwrap();
    _convert_json_to_config(json_value)
  }
}

fn _convert_json_to_config(json_value: serde_json::Value) -> config::Value {
  match json_value {
    JsonValue::Null => config::Value::new(None, config::ValueKind::Nil),
    JsonValue::Bool(b) => config::Value::from(b),
    JsonValue::Number(n) => {
      if let Some(i) = n.as_i64() {
        config::Value::from(i)
      } else if let Some(f) = n.as_f64() {
        config::Value::from(f)
      } else {
        unreachable!()
      }
    },
    JsonValue::String(s) => config::Value::from(s),
    JsonValue::Array(arr) => {
      let cv_arr: Vec<_> = arr.into_iter().map(_convert_json_to_config).collect();
      config::Value::new(None, config::ValueKind::Array(cv_arr))
    },
    JsonValue::Object(map) => {
      let cv_map: HashMap<_, _> = map.into_iter().map(|(k, v)| (k, _convert_json_to_config(v))).collect();
      config::Value::new(None, config::ValueKind::Table(cv_map))
    },
  }
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct AppConfig {
  #[serde(default)]
  pub _data_dir: PathBuf,
  #[serde(default)]
  pub _config_dir: PathBuf,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Config {
  #[serde(default)]
  pub taskwarrior: TaskwarriorConfig,
  #[serde(default)]
  pub task_report: TaskReportConfig,
  #[serde(default, flatten)]
  pub config: AppConfig,
  #[serde(default)]
  pub keybindings: KeyBindings,
  #[serde(default)]
  pub styles: Styles,
}

impl Config {
  pub fn new() -> Result<Self, config::ConfigError> {
    let default_config: Config = json5::from_str(CONFIG).unwrap();
    let data_dir = crate::utils::get_data_dir();
    let config_dir = crate::utils::get_config_dir();
    let mut builder = config::Config::builder()
      .set_default("task_report", default_config.task_report)?
      .set_default("_data_dir", data_dir.to_str().unwrap())?
      .set_default("_config_dir", config_dir.to_str().unwrap())?;

    let config_files = [
      ("config.json5", config::FileFormat::Json5),
      ("config.json", config::FileFormat::Json),
      ("config.yaml", config::FileFormat::Yaml),
      ("config.toml", config::FileFormat::Toml),
      ("config.ini", config::FileFormat::Ini),
    ];
    for (file, format) in &config_files {
      if config_dir.join(file).exists() {
        builder = builder.add_source(config::File::from(config_dir.join(file)).format(*format).required(false));
      }
    }

    let mut cfg: Self = builder.build()?.try_deserialize()?;

    for (mode, default_bindings) in default_config.keybindings.iter() {
      let user_bindings = cfg.keybindings.entry(*mode).or_default();
      for (key, cmd) in default_bindings.iter() {
        user_bindings.entry(key.clone()).or_insert_with(|| cmd.clone());
      }
    }
    for (mode, default_styles) in default_config.styles.iter() {
      let user_styles = cfg.styles.entry(*mode).or_default();
      for (style_key, style) in default_styles.iter() {
        user_styles.entry(style_key.clone()).or_insert_with(|| style.clone());
      }
    }

    Ok(cfg)
  }

  pub fn taskwarrior_config(&mut self) -> Result<()> {
    let output = std::process::Command::new("task")
      .arg("rc.color=off")
      .arg("rc._forcecolor=off")
      .arg("rc.defaultwidth=0")
      .arg("show")
      .output()?;

    if !output.status.success() {
      let output = std::process::Command::new("task").arg("diagnostics").output()?;
      return Err(color_eyre::eyre::eyre!(
        "Unable to run `task show`.\n{}\n{}\nPlease check your configuration or open a issue on github.",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
      ));
    }

    let data = String::from_utf8_lossy(&output.stdout);

    self.rule_precedence_color(&data);
    self.uda_priority_values(&data);
    self.weekstart(&data);
    self.due(&data);
    self.data_location(&data);
    self.color(&data);
    Ok(())
  }

  fn color(&mut self, data: &str) {
    let mut color = HashMap::new();
    for line in data.split('\n') {
      if line.starts_with("color.") {
        let mut i = line.split(' ');
        let attribute = i.next();
        let line = i.collect::<Vec<_>>().join(" ");
        let line = line.trim_start_matches(' ');
        let style = parse_style(line);
        if let Some(attr) = attribute {
          color.insert(attr.to_string(), style);
        };
      }
    }
    log::info!("{color:?}");
    self.taskwarrior.color = color;
  }

  fn data_location(&mut self, data: &str) {
    self.taskwarrior.data_location = get_config("data.location", data).unwrap();
  }

  fn rule_precedence_color(&mut self, data: &str) {
    let data = get_config("rule.precedence.color", data).unwrap();
    self.taskwarrior.rule_precedence_color = data.split(',').map(ToString::to_string).collect::<Vec<_>>();
  }

  fn weekstart(&mut self, data: &str) {
    let data = try_get_config("weekstart", data).unwrap_or_default();
    self.taskwarrior.weekstart = data.eq_ignore_ascii_case("Monday");
  }

  fn due(&mut self, data: &str) {
    self.taskwarrior.due = try_get_config("due", data).unwrap_or_default().parse::<usize>().unwrap_or(7)
  }

  fn uda_priority_values(&mut self, data: &str) {
    let data = get_config("uda.priority.values", data).unwrap();
    self.taskwarrior.uda_priority_values = data.split(',').map(ToString::to_string).collect::<Vec<_>>();
  }
}

fn get_config(config: &str, data: &str) -> Result<String> {
  try_get_config(config, data).ok_or(color_eyre::eyre::eyre!("Unable to parse `task show {config}`"))
}

fn try_get_config(config: &str, data: &str) -> Option<String> {
  let mut config_lines = Vec::new();

  for line in data.split('\n') {
    if config_lines.is_empty() {
      if line.starts_with(config) {
        config_lines.push(line.trim_start_matches(config).trim_start().trim_end().to_string());
      } else {
        let config = &config.replace('-', "_");
        if line.starts_with(config) {
          config_lines.push(line.trim_start_matches(config).trim_start().trim_end().to_string());
        }
      }
    } else {
      if !line.starts_with("   ") {
        return Some(config_lines.join(" "));
      }

      config_lines.push(line.trim_start().trim_end().to_string());
    }
  }

  if !config_lines.is_empty() {
    return Some(config_lines.join(" "));
  }

  None
}

#[derive(Clone, Debug, Default, Deref, DerefMut)]
pub struct KeyBindings(pub HashMap<Mode, HashMap<Vec<KeyEvent>, Action>>);

impl<'de> Deserialize<'de> for KeyBindings {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let parsed_map = HashMap::<Mode, HashMap<String, Action>>::deserialize(deserializer)?;

    let keybindings = parsed_map
      .into_iter()
      .map(|(mode, inner_map)| {
        let converted_inner_map =
          inner_map.into_iter().map(|(key_str, cmd)| (parse_key_sequence(&key_str).unwrap(), cmd)).collect();
        (mode, converted_inner_map)
      })
      .collect();

    Ok(KeyBindings(keybindings))
  }
}

fn parse_key_event(raw: &str) -> Result<KeyEvent, String> {
  let raw_lower = raw.to_ascii_lowercase();
  let (remaining, modifiers) = extract_modifiers(&raw_lower);
  parse_key_code_with_modifiers(remaining, modifiers)
}

fn extract_modifiers(raw: &str) -> (&str, KeyModifiers) {
  let mut modifiers = KeyModifiers::empty();
  let mut current = raw;

  loop {
    match current {
      rest if rest.starts_with("ctrl-") => {
        modifiers.insert(KeyModifiers::CONTROL);
        current = &rest[5..];
      },
      rest if rest.starts_with("alt-") => {
        modifiers.insert(KeyModifiers::ALT);
        current = &rest[4..];
      },
      rest if rest.starts_with("shift-") => {
        modifiers.insert(KeyModifiers::SHIFT);
        current = &rest[6..];
      },
      _ => break, // break out of the loop if no known prefix is detected
    };
  }

  (current, modifiers)
}

fn parse_key_code_with_modifiers(raw: &str, mut modifiers: KeyModifiers) -> Result<KeyEvent, String> {
  let c = match raw {
    "esc" => KeyCode::Esc,
    "enter" => KeyCode::Enter,
    "left" => KeyCode::Left,
    "right" => KeyCode::Right,
    "up" => KeyCode::Up,
    "down" => KeyCode::Down,
    "home" => KeyCode::Home,
    "end" => KeyCode::End,
    "pageup" => KeyCode::PageUp,
    "pagedown" => KeyCode::PageDown,
    "backtab" => {
      modifiers.insert(KeyModifiers::SHIFT);
      KeyCode::BackTab
    },
    "backspace" => KeyCode::Backspace,
    "delete" => KeyCode::Delete,
    "insert" => KeyCode::Insert,
    "f1" => KeyCode::F(1),
    "f2" => KeyCode::F(2),
    "f3" => KeyCode::F(3),
    "f4" => KeyCode::F(4),
    "f5" => KeyCode::F(5),
    "f6" => KeyCode::F(6),
    "f7" => KeyCode::F(7),
    "f8" => KeyCode::F(8),
    "f9" => KeyCode::F(9),
    "f10" => KeyCode::F(10),
    "f11" => KeyCode::F(11),
    "f12" => KeyCode::F(12),
    "space" => KeyCode::Char(' '),
    "hyphen" => KeyCode::Char('-'),
    "minus" => KeyCode::Char('-'),
    "tab" => KeyCode::Tab,
    c if c.len() == 1 => {
      let mut c = c.chars().next().unwrap();
      if modifiers.contains(KeyModifiers::SHIFT) {
        c = c.to_ascii_uppercase();
      }
      KeyCode::Char(c)
    },
    _ => return Err(format!("Unable to parse {raw}")),
  };
  Ok(KeyEvent::new(c, modifiers))
}

pub fn key_event_to_string(key_event: &KeyEvent) -> String {
  let char;
  let key_code = match key_event.code {
    KeyCode::Backspace => "backspace",
    KeyCode::Enter => "enter",
    KeyCode::Left => "left",
    KeyCode::Right => "right",
    KeyCode::Up => "up",
    KeyCode::Down => "down",
    KeyCode::Home => "home",
    KeyCode::End => "end",
    KeyCode::PageUp => "pageup",
    KeyCode::PageDown => "pagedown",
    KeyCode::Tab => "tab",
    KeyCode::BackTab => "backtab",
    KeyCode::Delete => "delete",
    KeyCode::Insert => "insert",
    KeyCode::F(c) => {
      char = format!("f({c})");
      &char
    },
    KeyCode::Char(c) if c == ' ' => "space",
    KeyCode::Char(c) => {
      char = c.to_string();
      &char
    },
    KeyCode::Esc => "esc",
    KeyCode::Null => "",
    KeyCode::CapsLock => "",
    KeyCode::Menu => "",
    KeyCode::ScrollLock => "",
    KeyCode::Media(_) => "",
    KeyCode::NumLock => "",
    KeyCode::PrintScreen => "",
    KeyCode::Pause => "",
    KeyCode::KeypadBegin => "",
    KeyCode::Modifier(_) => "",
  };

  let mut modifiers = Vec::with_capacity(3);

  if key_event.modifiers.intersects(KeyModifiers::CONTROL) {
    modifiers.push("ctrl");
  }

  if key_event.modifiers.intersects(KeyModifiers::SHIFT) {
    modifiers.push("shift");
  }

  if key_event.modifiers.intersects(KeyModifiers::ALT) {
    modifiers.push("alt");
  }

  let mut key = modifiers.join("-");

  if !key.is_empty() {
    key.push('-');
  }
  key.push_str(key_code);

  key
}

pub fn parse_key_sequence(raw: &str) -> Result<Vec<KeyEvent>, String> {
  if raw.chars().filter(|c| *c == '>').count() != raw.chars().filter(|c| *c == '<').count() {
    return Err(format!("Unable to parse `{}`", raw));
  }
  let raw = if !raw.contains("><") {
    let raw = raw.strip_prefix('<').unwrap_or(raw);
    let raw = raw.strip_prefix('>').unwrap_or(raw);
    raw
  } else {
    raw
  };
  let sequences = raw
    .split("><")
    .map(|seq| {
      if let Some(s) = seq.strip_prefix('<') {
        s
      } else if let Some(s) = seq.strip_suffix('>') {
        s
      } else {
        seq
      }
    })
    .collect::<Vec<_>>();

  sequences.into_iter().map(parse_key_event).collect()
}

#[derive(Clone, Debug, Default, Deref, DerefMut)]
pub struct Styles(pub HashMap<Mode, HashMap<String, Style>>);

impl<'de> Deserialize<'de> for Styles {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let parsed_map = HashMap::<Mode, HashMap<String, String>>::deserialize(deserializer)?;

    let styles = parsed_map
      .into_iter()
      .map(|(mode, inner_map)| {
        let converted_inner_map = inner_map.into_iter().map(|(str, style)| (str, parse_style(&style))).collect();
        (mode, converted_inner_map)
      })
      .collect();

    Ok(Styles(styles))
  }
}

pub fn parse_style(line: &str) -> Style {
  let (foreground, background) = line.split_at(line.to_lowercase().find("on ").unwrap_or(line.len()));
  let foreground = process_color_string(foreground);
  let background = process_color_string(&background.replace("on ", ""));

  let mut style = Style::default();
  if let Some(fg) = parse_color(&foreground.0) {
    style = style.fg(fg);
  }
  if let Some(bg) = parse_color(&background.0) {
    style = style.bg(bg);
  }
  style = style.add_modifier(foreground.1 | background.1);
  style
}

fn process_color_string(color_str: &str) -> (String, Modifier) {
  let color = color_str
    .replace("grey", "gray")
    .replace("bright ", "")
    .replace("bold ", "")
    .replace("underline ", "")
    .replace("inverse ", "");

  let mut modifiers = Modifier::empty();
  if color_str.contains("underline") {
    modifiers |= Modifier::UNDERLINED;
  }
  if color_str.contains("bold") {
    modifiers |= Modifier::BOLD;
  }
  if color_str.contains("inverse") {
    modifiers |= Modifier::REVERSED;
  }

  (color, modifiers)
}

fn parse_color(s: &str) -> Option<Color> {
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

#[cfg(test)]
mod tests {
  use pretty_assertions::assert_eq;

  use super::*;

  #[test]
  fn test_parse_style_default() {
    let style = parse_style("");
    assert_eq!(style, Style::default());
  }

  #[test]
  fn test_parse_style_foreground() {
    let style = parse_style("red");
    assert_eq!(style.fg, Some(Color::Indexed(1)));
  }

  #[test]
  fn test_parse_style_background() {
    let style = parse_style("on blue");
    assert_eq!(style.bg, Some(Color::Indexed(4)));
  }

  #[test]
  fn test_parse_style_modifiers() {
    let style = parse_style("underline red on blue");
    assert_eq!(style.fg, Some(Color::Indexed(1)));
    assert_eq!(style.bg, Some(Color::Indexed(4)));
  }

  #[test]
  fn test_process_color_string() {
    let (color, modifiers) = process_color_string("underline bold inverse gray");
    assert_eq!(color, "gray");
    assert!(modifiers.contains(Modifier::UNDERLINED));
    assert!(modifiers.contains(Modifier::BOLD));
    assert!(modifiers.contains(Modifier::REVERSED));
  }

  #[test]
  fn test_parse_color_rgb() {
    let color = parse_color("rgb123");
    let expected = 16 + 1 * 36 + 2 * 6 + 3;
    assert_eq!(color, Some(Color::Indexed(expected)));
  }

  #[test]
  fn test_parse_color_unknown() {
    let color = parse_color("unknown");
    assert_eq!(color, None);
  }

  #[test]
  fn test_config() -> Result<()> {
    let c = Config::new()?;
    assert_eq!(
      c.keybindings.get(&Mode::TaskReport).unwrap().get(&parse_key_sequence("<q>").unwrap_or_default()).unwrap(),
      &Action::Quit
    );
    Ok(())
  }

  #[test]
  fn test_simple_keys() {
    assert_eq!(parse_key_event("a").unwrap(), KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty()));

    assert_eq!(parse_key_event("enter").unwrap(), KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()));

    assert_eq!(parse_key_event("esc").unwrap(), KeyEvent::new(KeyCode::Esc, KeyModifiers::empty()));
  }

  #[test]
  fn test_with_modifiers() {
    assert_eq!(parse_key_event("ctrl-a").unwrap(), KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL));

    assert_eq!(parse_key_event("alt-enter").unwrap(), KeyEvent::new(KeyCode::Enter, KeyModifiers::ALT));

    assert_eq!(parse_key_event("shift-esc").unwrap(), KeyEvent::new(KeyCode::Esc, KeyModifiers::SHIFT));
  }

  #[test]
  fn test_multiple_modifiers() {
    assert_eq!(
      parse_key_event("ctrl-alt-a").unwrap(),
      KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL | KeyModifiers::ALT)
    );

    assert_eq!(
      parse_key_event("ctrl-shift-enter").unwrap(),
      KeyEvent::new(KeyCode::Enter, KeyModifiers::CONTROL | KeyModifiers::SHIFT)
    );
  }

  #[test]
  fn test_reverse_multiple_modifiers() {
    assert_eq!(
      key_event_to_string(&KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL | KeyModifiers::ALT)),
      "ctrl-alt-a".to_string()
    );
  }

  #[test]
  fn test_invalid_keys() {
    assert!(parse_key_event("invalid-key").is_err());
    assert!(parse_key_event("ctrl-invalid-key").is_err());
  }

  #[test]
  fn test_case_insensitivity() {
    assert_eq!(parse_key_event("CTRL-a").unwrap(), KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL));

    assert_eq!(parse_key_event("AlT-eNtEr").unwrap(), KeyEvent::new(KeyCode::Enter, KeyModifiers::ALT));
  }
}
