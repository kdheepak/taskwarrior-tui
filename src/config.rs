use std::{collections::HashMap, error::Error, str};

use anyhow::{Context, Result};
use crossterm::terminal::size;
use ratatui::{
  style::{Color, Modifier, Style},
  symbols::{bar::FULL, line::DOUBLE_VERTICAL},
};

trait TaskWarriorBool {
  fn get_bool(&self) -> Option<bool>;
}

impl TaskWarriorBool for String {
  fn get_bool(&self) -> Option<bool> {
    if self == "true" || self == "1" || self == "y" || self == "yes" || self == "on" {
      Some(true)
    } else if self == "false" || self == "0" || self == "n" || self == "no" || self == "off" {
      Some(false)
    } else {
      None
    }
  }
}

impl TaskWarriorBool for str {
  fn get_bool(&self) -> Option<bool> {
    if self == "true" || self == "1" || self == "y" || self == "yes" || self == "on" {
      Some(true)
    } else if self == "false" || self == "0" || self == "n" || self == "no" || self == "off" {
      Some(false)
    } else {
      None
    }
  }
}

#[derive(Debug)]
pub struct Uda {
  label: String,
  kind: String,
  values: Option<Vec<String>>,
  default: Option<String>,
  urgency: Option<f64>,
}

#[derive(Debug)]
pub struct Config {
  pub enabled: bool,
  pub color: HashMap<String, Style>,
  pub color_keywords: Vec<(String, Style)>,
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
  pub uda_task_report_use_alternate_style: bool,
  pub uda_task_report_show_info: bool,
  pub uda_task_report_looping: bool,
  pub uda_task_report_jump_to_task_on_add: bool,
  pub uda_selection_indicator: String,
  pub uda_mark_highlight_indicator: String,
  pub uda_unmark_highlight_indicator: String,
  pub uda_mark_indicator: String,
  pub uda_unmark_indicator: String,
  pub uda_scrollbar_indicator: String,
  pub uda_scrollbar_area: String,
  pub uda_style_report_scrollbar: Style,
  pub uda_style_report_scrollbar_area: Style,
  pub uda_selection_bold: bool,
  pub uda_selection_italic: bool,
  pub uda_selection_dim: bool,
  pub uda_selection_blink: bool,
  pub uda_selection_reverse: bool,
  pub uda_calendar_months_per_row: usize,
  pub uda_style_context_active: Style,
  pub uda_style_report_menu_active: Style,
  pub uda_style_report_selection: Style,
  pub uda_style_calendar_title: Style,
  pub uda_style_calendar_today: Style,
  pub uda_style_navbar: Style,
  pub uda_style_command: Style,
  pub uda_style_report_completion_pane: Style,
  pub uda_style_report_completion_pane_highlight: Style,
  pub uda_style_title: Style,
  pub uda_style_title_border: Style,
  pub uda_style_help_gauge: Style,
  pub uda_style_command_error: Style,
  pub uda_shortcuts: Vec<String>,
  pub uda_change_focus_rotate: bool,
  pub uda_background_process: String,
  pub uda_background_process_period: usize,
  pub uda_quick_tag_name: String,
  pub uda_tasklist_vertical: bool,
  pub uda_task_report_prompt_on_undo: bool,
  pub uda_task_report_prompt_on_delete: bool,
  pub uda_task_report_prompt_on_done: bool,
  pub uda_task_report_date_time_vague_more_precise: bool,
  pub uda_context_menu_select_on_move: bool,
  pub uda_context_menu_close_on_select: bool,
  pub uda_report_menu_select_on_move: bool,
  pub uda_report_menu_close_on_select: bool,
  pub uda: Vec<Uda>,
}

impl Config {
  pub fn new(data: &str, report: &str) -> Result<Self> {
    let bool_collection = Self::get_bool_collection();

    let enabled = true;
    let obfuscate = bool_collection.get("obfuscate").copied().unwrap_or(false);
    let print_empty_columns = bool_collection.get("print_empty_columns").copied().unwrap_or(false);

    let color = Self::get_color_collection(data);
    let color_keywords: Vec<(String, Style)> = color
      .iter()
      .filter_map(|(key, style)| key.strip_prefix("color.keyword.").map(|kw| (kw.to_string(), *style)))
      .collect();
    let filter = Self::get_filter(data, report)?;
    let filter = if filter.trim_start().trim_end().is_empty() {
      filter
    } else {
      format!("{} ", filter)
    };
    let data_location = Self::get_data_location(data);
    let due = Self::get_due(data);
    let weekstart = Self::get_weekstart(data);
    let rule_precedence_color = Self::get_rule_precedence_color(data);
    let uda_priority_values = Self::get_uda_priority_values(data);
    let uda_tick_rate = Self::get_uda_tick_rate(data);
    let uda_change_focus_rotate = Self::get_uda_change_focus_rotate(data);
    let uda_auto_insert_double_quotes_on_add = Self::get_uda_auto_insert_double_quotes_on_add(data);
    let uda_auto_insert_double_quotes_on_annotate = Self::get_uda_auto_insert_double_quotes_on_annotate(data);
    let uda_auto_insert_double_quotes_on_log = Self::get_uda_auto_insert_double_quotes_on_log(data);
    let uda_prefill_task_metadata = Self::get_uda_prefill_task_metadata(data);
    let uda_reset_filter_on_esc = Self::get_uda_reset_filter_on_esc(data);
    let uda_task_detail_prefetch = Self::get_uda_task_detail_prefetch(data);
    let uda_task_report_use_all_tasks_for_completion = Self::get_uda_task_report_use_all_tasks_for_completion(data);
    let uda_task_report_use_alternate_style = Self::get_uda_task_report_use_alternate_style(data);
    let uda_task_report_show_info = Self::get_uda_task_report_show_info(data);
    let uda_task_report_looping = Self::get_uda_task_report_looping(data);
    let uda_task_report_jump_to_task_on_add = Self::get_uda_task_report_jump_to_task_on_add(data);
    let uda_selection_indicator = Self::get_uda_selection_indicator(data);
    let uda_mark_highlight_indicator = Self::get_uda_mark_highlight_indicator(data);
    let uda_unmark_highlight_indicator = Self::get_uda_unmark_highlight_indicator(data);
    let uda_mark_indicator = Self::get_uda_mark_indicator(data);
    let uda_unmark_indicator = Self::get_uda_unmark_indicator(data);
    let uda_scrollbar_indicator = Self::get_uda_scrollbar_indicator(data);
    let uda_scrollbar_area = Self::get_uda_scrollbar_area(data);
    let uda_selection_bold = Self::get_uda_selection_bold(data);
    let uda_selection_italic = Self::get_uda_selection_italic(data);
    let uda_selection_dim = Self::get_uda_selection_dim(data);
    let uda_selection_blink = Self::get_uda_selection_blink(data);
    let uda_selection_reverse = Self::get_uda_selection_reverse(data);
    let uda_calendar_months_per_row = Self::get_uda_months_per_row(data);
    let uda_style_report_selection = Self::get_uda_style("report.selection", data);
    let uda_style_report_scrollbar = Self::get_uda_style("report.scrollbar", data);
    let uda_style_report_scrollbar_area = Self::get_uda_style("report.scrollbar.area", data);
    let uda_style_calendar_title = Self::get_uda_style("calendar.title", data);
    let uda_style_calendar_today = Self::get_uda_style("calendar.today", data);
    let uda_style_navbar = Self::get_uda_style("navbar", data);
    let uda_style_command = Self::get_uda_style("command", data);
    let uda_style_context_active = Self::get_uda_style("context.active", data);
    let uda_style_report_menu_active = Self::get_uda_style("report-menu.active", data);
    let uda_style_report_completion_pane = Self::get_uda_style("report.completion-pane", data);
    let uda_style_report_completion_pane_highlight = Self::get_uda_style("report.completion-pane-highlight", data);
    let uda_style_title = Self::get_uda_style("title", data);
    let uda_style_title_border = Self::get_uda_style("title.border", data);
    let uda_style_help_gauge = Self::get_uda_style("help.gauge", data);
    let uda_style_command_error = Self::get_uda_style("command.error", data);
    let uda_shortcuts = Self::get_uda_shortcuts(data);
    let uda_background_process = Self::get_uda_background_process(data);
    let uda_background_process_period = Self::get_uda_background_process_period(data);
    let uda_style_report_selection = uda_style_report_selection.unwrap_or_default();
    let uda_style_report_scrollbar = uda_style_report_scrollbar.unwrap_or_else(|| Style::default().fg(Color::Black));
    let uda_style_report_scrollbar_area = uda_style_report_scrollbar_area.unwrap_or_default();
    let uda_style_calendar_title = uda_style_calendar_title.unwrap_or_default();
    let uda_style_calendar_today = uda_style_calendar_today.unwrap_or_else(|| Style::default().add_modifier(Modifier::BOLD));
    let uda_style_navbar = uda_style_navbar.unwrap_or_else(|| Style::default().add_modifier(Modifier::REVERSED));
    let uda_style_command = uda_style_command.unwrap_or_else(|| Style::default().add_modifier(Modifier::REVERSED));
    let uda_style_context_active = uda_style_context_active.unwrap_or_else(|| Style::default().add_modifier(Modifier::BOLD));
    let uda_style_report_menu_active = uda_style_report_menu_active.unwrap_or_else(|| Style::default().add_modifier(Modifier::BOLD));
    let uda_style_report_completion_pane =
      uda_style_report_completion_pane.unwrap_or_else(|| Style::default().fg(Color::Black).bg(Color::Rgb(223, 223, 223)));
    let uda_style_report_completion_pane_highlight = uda_style_report_completion_pane_highlight.unwrap_or(uda_style_report_completion_pane);
    let uda_style_title = uda_style_title.unwrap_or_else(|| Style::default().fg(Color::LightCyan));
    let uda_style_title_border = uda_style_title_border.unwrap_or_else(|| Style::default().fg(Color::White));
    let uda_style_help_gauge = uda_style_help_gauge.unwrap_or_else(|| Style::default().fg(Color::Gray));
    let uda_style_command_error = uda_style_command_error.unwrap_or_else(|| Style::default().fg(Color::Red));
    let uda_quick_tag_name = Self::get_uda_quick_tag_name(data);
    let uda_tasklist_vertical = Self::get_uda_tasklist_vertical(data);
    let uda_task_report_prompt_on_undo = Self::get_uda_task_report_prompt_on_undo(data);
    let uda_task_report_prompt_on_delete = Self::get_uda_task_report_prompt_on_delete(data);
    let uda_task_report_prompt_on_done = Self::get_uda_task_report_prompt_on_done(data);
    let uda_context_menu_select_on_move = Self::get_uda_context_menu_select_on_move(data);
    let uda_context_menu_close_on_select = Self::get_uda_context_menu_close_on_select(data);
    let uda_report_menu_select_on_move = Self::get_uda_report_menu_select_on_move(data);
    let uda_report_menu_close_on_select = Self::get_uda_report_menu_close_on_select(data);
    let uda_task_report_date_time_vague_more_precise = Self::get_uda_task_report_date_time_vague_more_precise(data);

    Ok(Self {
      enabled,
      color,
      color_keywords,
      filter,
      data_location,
      obfuscate,
      print_empty_columns,
      due,
      weekstart,
      rule_precedence_color,
      uda_priority_values,
      uda_tick_rate,
      uda_change_focus_rotate,
      uda_auto_insert_double_quotes_on_add,
      uda_auto_insert_double_quotes_on_annotate,
      uda_auto_insert_double_quotes_on_log,
      uda_prefill_task_metadata,
      uda_reset_filter_on_esc,
      uda_task_detail_prefetch,
      uda_task_report_use_all_tasks_for_completion,
      uda_task_report_use_alternate_style,
      uda_task_report_show_info,
      uda_task_report_looping,
      uda_task_report_jump_to_task_on_add,
      uda_selection_indicator,
      uda_mark_highlight_indicator,
      uda_unmark_highlight_indicator,
      uda_mark_indicator,
      uda_unmark_indicator,
      uda_scrollbar_indicator,
      uda_scrollbar_area,
      uda_selection_bold,
      uda_selection_italic,
      uda_selection_dim,
      uda_selection_blink,
      uda_selection_reverse,
      uda_calendar_months_per_row,
      uda_style_report_selection,
      uda_style_report_scrollbar,
      uda_style_report_scrollbar_area,
      uda_style_calendar_title,
      uda_style_calendar_today,
      uda_style_navbar,
      uda_style_command,
      uda_style_context_active,
      uda_style_report_menu_active,
      uda_style_report_completion_pane,
      uda_style_report_completion_pane_highlight,
      uda_style_title,
      uda_style_title_border,
      uda_style_help_gauge,
      uda_style_command_error,
      uda_shortcuts,
      uda_background_process,
      uda_background_process_period,
      uda_quick_tag_name,
      uda_tasklist_vertical,
      uda_task_report_prompt_on_undo,
      uda_task_report_prompt_on_delete,
      uda_task_report_prompt_on_done,
      uda_context_menu_select_on_move,
      uda_context_menu_close_on_select,
      uda_report_menu_select_on_move,
      uda_report_menu_close_on_select,
      uda_task_report_date_time_vague_more_precise,
      uda: vec![],
    })
  }

  fn get_bool_collection() -> HashMap<String, bool> {
    HashMap::new()
  }

  fn get_uda_background_process(data: &str) -> String {
    Self::get_config("uda.taskwarrior-tui.background_process", data).unwrap_or_default()
  }

  fn get_uda_background_process_period(data: &str) -> usize {
    Self::get_config("uda.taskwarrior-tui.background_process_period", data)
      .unwrap_or_default()
      .parse::<usize>()
      .unwrap_or(60)
  }

  fn get_uda_shortcuts(data: &str) -> Vec<String> {
    let mut v = vec![];
    for s in 0..=9 {
      let c = format!("uda.taskwarrior-tui.shortcuts.{}", s);
      let s = Self::get_config(&c, data).unwrap_or_default();
      v.push(s);
    }
    v
  }

  fn get_uda_style(config: &str, data: &str) -> Option<Style> {
    let c = format!("uda.taskwarrior-tui.style.{}", config);
    let s = Self::get_config(&c, data)?;
    Some(Self::get_tcolor(&s))
  }

  fn get_color_collection(data: &str) -> HashMap<String, Style> {
    let mut color_collection = HashMap::new();
    for line in data.split('\n') {
      if let Some((attribute, style)) = Self::parse_color_config(line) {
        color_collection.insert(attribute, style);
      }
    }
    color_collection
  }

  fn parse_color_config(line: &str) -> Option<(String, Style)> {
    if !line.starts_with("color.") {
      return None;
    }

    if let Some(delimiter) = Self::find_color_config_delimiter(line) {
      let attribute = line[..delimiter].trim_end().to_string();
      let style = Self::get_tcolor(line[delimiter..].trim_start());
      return Some((attribute, style));
    }

    Some((line.trim_end().to_string(), Style::default()))
  }

  fn find_color_config_delimiter(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
      if bytes[i] == b' ' {
        let start = i;
        while i < bytes.len() && bytes[i] == b' ' {
          i += 1;
        }
        // `task show` prints config entries in aligned columns, so the key/value
        // boundary is usually the first run of 2+ spaces. That lets us keep
        // single spaces inside color keys such as `color.uda.foo.In Review`.
        if i - start >= 2 {
          return Some(start);
        }
      } else {
        i += 1;
      }
    }

    for (delimiter, _) in line.match_indices(' ') {
      // Some inputs may not preserve the padded column gap. In that case, scan
      // each space and choose the first suffix that parses as a valid
      // Taskwarrior color expression, treating everything before it as the key.
      if Self::parse_tcolor(line[delimiter..].trim_start()).is_some() {
        return Some(delimiter);
      }
    }

    None
  }

  pub fn get_tcolor(line: &str) -> Style {
    Self::parse_tcolor(line).unwrap_or_default()
  }

  fn parse_tcolor(line: &str) -> Option<Style> {
    // Normalize spelling variants
    let line = line.replace("grey", "gray");
    let line = line.trim();

    // Split on " on " to separate foreground from background.
    // Also handle the background-only case where the string starts with "on ".
    // We search case-insensitively but keep the original casing for parsing.
    let lower = line.to_lowercase();
    let (fg_raw, bg_raw) = if lower.starts_with("on ") {
      // Background-only: "on blue", "on bright red", etc.
      ("", &line[3..])
    } else if let Some(pos) = lower.find(" on ") {
      (&line[..pos], &line[pos + 4..])
    } else {
      (line, "")
    };

    let mut modifiers = Modifier::empty();

    // Parse modifiers from the foreground portion (taskwarrior puts them before "on")
    if fg_raw.contains("bold") {
      modifiers |= Modifier::BOLD;
    }
    if fg_raw.contains("underline") {
      modifiers |= Modifier::UNDERLINED;
    }
    if fg_raw.contains("inverse") {
      modifiers |= Modifier::REVERSED;
    }
    if fg_raw.contains("italic") {
      modifiers |= Modifier::ITALIC;
    }
    if fg_raw.contains("strikethrough") {
      modifiers |= Modifier::CROSSED_OUT;
    }

    // Strip all modifier keywords from the foreground color name.
    // We split on whitespace and filter out recognized modifiers
    let modifier_words: &[&str] = &["bold", "underline", "inverse", "italic", "strikethrough"];
    let fg_color = fg_raw
      .split_whitespace()
      .filter(|word| !modifier_words.contains(&word.to_lowercase().as_str()))
      .collect::<Vec<_>>()
      .join(" ");
    let fg_color = fg_color.trim();

    // Strip leading "bright " from background (taskwarrior uses "on bright red")
    let bg_color = bg_raw.trim_start_matches("bright ").trim();
    let bg_is_bright = bg_raw.trim_start().starts_with("bright ");

    let mut style = Style::default();
    if !fg_color.is_empty() {
      style = style.fg(Self::parse_color(fg_color)?);
    }
    if !bg_color.is_empty() {
      style = style.bg(Self::parse_color_bg(bg_color, bg_is_bright)?);
    }
    style = style.add_modifier(modifiers);
    Some(style)
  }

  /// Parse a color name into a ratatui Color.
  ///
  /// Supports all Taskwarrior color formats:
  /// - Named 16-color: `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`
  /// - High-intensity variants: `bright <name>` or `bold <name>` -> indices 8-15
  /// - 256-color indexed: `color0` through `color255`
  /// - Grayscale ramp: `gray0` through `gray23` (also `grey` spelling)
  /// - RGB color cube: `rgb000` through `rgb555` (6x6x6)
  fn parse_color(s: &str) -> Option<Color> {
    let s = s.trim();
    if s.is_empty() {
      return None;
    }

    // High-intensity named colors: "bright <name>" or "bold <name>"
    let base = if let Some(rest) = s.strip_prefix("bright ") {
      let idx = Self::named_color_index(rest.trim())?;
      return Some(Color::Indexed(idx + 8));
    } else if let Some(rest) = s.strip_prefix("bold ") {
      let idx = Self::named_color_index(rest.trim())?;
      return Some(Color::Indexed(idx + 8));
    } else {
      s
    };

    // 256-color indexed: "colorN"
    if let Some(rest) = base.strip_prefix("color") {
      let n = rest.parse::<u8>().ok()?;
      return Some(Color::Indexed(n));
    }

    // Grayscale ramp: "grayN" (gray0-gray23 -> indexed 232-255)
    if let Some(rest) = base.strip_prefix("gray") {
      let n = rest.parse::<u8>().ok()?;
      return Some(Color::Indexed(232 + n));
    }

    // RGB color cube: "rgbRGB" where each digit is 0-5
    if let Some(rest) = base.strip_prefix("rgb") {
      let bytes = rest.as_bytes();
      if bytes.len() >= 3 {
        let r = (bytes[0] as char).to_digit(6)? as u8;
        let g = (bytes[1] as char).to_digit(6)? as u8;
        let b = (bytes[2] as char).to_digit(6)? as u8;
        return Some(Color::Indexed(16 + r * 36 + g * 6 + b));
      }
      return None;
    }

    // Plain named color
    Self::named_color_index(base).map(Color::Indexed)
  }

  /// Parse a background color. Handles the "bright" prefix for named colors
  /// (e.g. "bright red" -> high-intensity red, index 8-15).
  /// For indexed/gray/rgb colors the bright prefix is ignored (no bright variant exists).
  fn parse_color_bg(s: &str, is_bright: bool) -> Option<Color> {
    // For named colors with bright prefix, map to high-intensity variant (8-15)
    if is_bright && let Some(idx) = Self::named_color_index(s) {
      return Some(Color::Indexed(idx + 8));
      // For colorN/gray/rgb with "bright" prefix, fall through to normal parsing
      // (no bright variant for indexed colors — just use the index as-is)
    }
    Self::parse_color(s)
  }

  /// Map a plain color name to its 0-7 terminal index.
  fn named_color_index(s: &str) -> Option<u8> {
    match s.trim() {
      "black" => Some(0),
      "red" => Some(1),
      "green" => Some(2),
      "yellow" => Some(3),
      "blue" => Some(4),
      "magenta" => Some(5),
      "cyan" => Some(6),
      "white" => Some(7),
      _ => None,
    }
  }

  fn get_config(config: &str, data: &str) -> Option<String> {
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

  fn get_due(data: &str) -> usize {
    Self::get_config("due", data).unwrap_or_default().parse::<usize>().unwrap_or(7)
  }

  fn get_weekstart(data: &str) -> bool {
    let data = Self::get_config("weekstart", data).unwrap_or_default();
    data.eq_ignore_ascii_case("Monday")
  }

  fn get_rule_precedence_color(data: &str) -> Vec<String> {
    let data = Self::get_config("rule.precedence.color", data)
      .context("Unable to parse `task show rule.precedence.color`.")
      .unwrap();
    data.split(',').map(ToString::to_string).collect::<Vec<_>>()
  }

  fn get_uda_priority_values(data: &str) -> Vec<String> {
    let data = Self::get_config("uda.priority.values", data)
      .context("Unable to parse `task show uda.priority.values`.")
      .unwrap();
    data.split(',').map(ToString::to_string).collect::<Vec<_>>()
  }

  pub fn get_filter(data: &str, report: &str) -> Result<String> {
    if report == "all" {
      Ok("".into())
    } else if let Some(s) = Self::get_config(format!("uda.taskwarrior-tui.task-report.{}.filter", report).as_str(), data) {
      Ok(s)
    } else {
      Ok(Self::get_config(format!("report.{}.filter", report).as_str(), data).unwrap_or_default())
    }
  }

  fn get_data_location(data: &str) -> String {
    Self::get_config("data.location", data)
      .context("Unable to parse `task show data.location`.")
      .unwrap()
  }

  fn get_uda_auto_insert_double_quotes_on_add(data: &str) -> bool {
    Self::get_config("uda.taskwarrior-tui.task-report.auto-insert-double-quotes-on-add", data)
      .unwrap_or_default()
      .get_bool()
      .unwrap_or(true)
  }

  fn get_uda_auto_insert_double_quotes_on_annotate(data: &str) -> bool {
    Self::get_config("uda.taskwarrior-tui.task-report.auto-insert-double-quotes-on-annotate", data)
      .unwrap_or_default()
      .get_bool()
      .unwrap_or(true)
  }

  fn get_uda_auto_insert_double_quotes_on_log(data: &str) -> bool {
    Self::get_config("uda.taskwarrior-tui.task-report.auto-insert-double-quotes-on-log", data)
      .unwrap_or_default()
      .get_bool()
      .unwrap_or(true)
  }

  fn get_uda_prefill_task_metadata(data: &str) -> bool {
    Self::get_config("uda.taskwarrior-tui.task-report.pre-fill-task-meta-data", data)
      .unwrap_or_default()
      .get_bool()
      .unwrap_or(false)
  }

  fn get_uda_reset_filter_on_esc(data: &str) -> bool {
    Self::get_config("uda.taskwarrior-tui.task-report.reset-filter-on-esc", data)
      .unwrap_or_default()
      .get_bool()
      .unwrap_or(true)
  }

  fn get_uda_change_focus_rotate(data: &str) -> bool {
    Self::get_config("uda.taskwarrior-tui.tabs.change-focus-rotate", data)
      .unwrap_or_default()
      .get_bool()
      .unwrap_or(false)
  }

  fn get_uda_tick_rate(data: &str) -> u64 {
    Self::get_config("uda.taskwarrior-tui.tick-rate", data)
      .unwrap_or_default()
      .parse::<u64>()
      .unwrap_or(250)
  }

  fn get_uda_task_detail_prefetch(data: &str) -> usize {
    Self::get_config("uda.taskwarrior-tui.task-report.task-detail-prefetch", data)
      .unwrap_or_default()
      .parse::<usize>()
      .unwrap_or(10)
  }

  fn get_uda_task_report_use_all_tasks_for_completion(data: &str) -> bool {
    Self::get_config("uda.taskwarrior-tui.task-report.use-all-tasks-for-completion", data)
      .unwrap_or_default()
      .get_bool()
      .unwrap_or(false)
  }

  fn get_uda_task_report_use_alternate_style(data: &str) -> bool {
    Self::get_config("uda.taskwarrior-tui.task-report.use-alternate-style", data)
      .unwrap_or_default()
      .get_bool()
      .unwrap_or(true)
  }

  fn get_uda_task_report_show_info(data: &str) -> bool {
    Self::get_config("uda.taskwarrior-tui.task-report.show-info", data)
      .unwrap_or_default()
      .get_bool()
      .unwrap_or(true)
  }

  fn get_uda_task_report_jump_to_task_on_add(data: &str) -> bool {
    Self::get_config("uda.taskwarrior-tui.task-report.jump-to-task-on-add", data)
      .unwrap_or_default()
      .get_bool()
      .unwrap_or(true)
  }

  fn get_uda_task_report_looping(data: &str) -> bool {
    Self::get_config("uda.taskwarrior-tui.task-report.looping", data)
      .unwrap_or_default()
      .get_bool()
      .unwrap_or(true)
  }

  fn get_uda_task_report_date_time_vague_more_precise(data: &str) -> bool {
    Self::get_config("uda.taskwarrior-tui.task-report.date-time-vague-more-precise", data)
      .unwrap_or_default()
      .get_bool()
      .unwrap_or(false)
  }

  fn get_uda_task_report_prompt_on_done(data: &str) -> bool {
    Self::get_config("uda.taskwarrior-tui.task-report.prompt-on-done", data)
      .unwrap_or_default()
      .get_bool()
      .unwrap_or(false)
  }

  fn get_uda_context_menu_select_on_move(data: &str) -> bool {
    Self::get_config("uda.taskwarrior-tui.context-menu.select-on-move", data)
      .unwrap_or_default()
      .get_bool()
      .unwrap_or(false)
  }

  fn get_uda_context_menu_close_on_select(data: &str) -> bool {
    Self::get_config("uda.taskwarrior-tui.context-menu.close-on-select", data)
      .unwrap_or_default()
      .get_bool()
      .unwrap_or(true)
  }

  fn get_uda_report_menu_select_on_move(data: &str) -> bool {
    Self::get_config("uda.taskwarrior-tui.report-menu.select-on-move", data)
      .unwrap_or_default()
      .get_bool()
      .unwrap_or(false)
  }

  fn get_uda_report_menu_close_on_select(data: &str) -> bool {
    Self::get_config("uda.taskwarrior-tui.report-menu.close-on-select", data)
      .unwrap_or_default()
      .get_bool()
      .unwrap_or(true)
  }

  fn get_uda_task_report_prompt_on_undo(data: &str) -> bool {
    Self::get_config("uda.taskwarrior-tui.task-report.prompt-on-undo", data)
      .unwrap_or_default()
      .get_bool()
      .unwrap_or(false)
  }

  fn get_uda_task_report_prompt_on_delete(data: &str) -> bool {
    Self::get_config("uda.taskwarrior-tui.task-report.prompt-on-delete", data)
      .unwrap_or_default()
      .get_bool()
      .unwrap_or(false)
  }

  fn get_uda_selection_indicator(data: &str) -> String {
    let indicator = Self::get_config("uda.taskwarrior-tui.selection.indicator", data);
    match indicator {
      None => "\u{2022} ".to_string(),
      Some(indicator) => format!("{} ", indicator),
    }
  }

  fn get_uda_mark_indicator(data: &str) -> String {
    let indicator = Self::get_config("uda.taskwarrior-tui.mark.indicator", data);
    match indicator {
      None => "\u{2714} ".to_string(),
      Some(indicator) => format!("{} ", indicator),
    }
  }

  fn get_uda_unmark_indicator(data: &str) -> String {
    let indicator = Self::get_config("uda.taskwarrior-tui.unmark.indicator", data);
    match indicator {
      None => "  ".to_string(),
      Some(indicator) => format!("{} ", indicator),
    }
  }

  fn get_uda_scrollbar_indicator(data: &str) -> String {
    let indicator = Self::get_config("uda.taskwarrior-tui.scrollbar.indicator", data);
    match indicator {
      None => FULL.to_string(),
      Some(indicator) => format!("{}", indicator.chars().next().unwrap_or_else(|| FULL.to_string().chars().next().unwrap())),
    }
  }

  fn get_uda_scrollbar_area(data: &str) -> String {
    let area = Self::get_config("uda.taskwarrior-tui.scrollbar.area", data);
    match area {
      None => DOUBLE_VERTICAL.to_string(),
      Some(area) => format!(
        "{}",
        area.chars().next().unwrap_or_else(|| DOUBLE_VERTICAL.to_string().chars().next().unwrap())
      ),
    }
  }

  fn get_uda_mark_highlight_indicator(data: &str) -> String {
    let indicator = Self::get_config("uda.taskwarrior-tui.mark-selection.indicator", data);
    match indicator {
      None => "\u{29bf} ".to_string(),
      Some(indicator) => format!("{} ", indicator),
    }
  }

  fn get_uda_unmark_highlight_indicator(data: &str) -> String {
    let indicator = Self::get_config("uda.taskwarrior-tui.unmark-selection.indicator", data);
    match indicator {
      None => "\u{29be} ".to_string(),
      Some(indicator) => format!("{} ", indicator),
    }
  }

  fn get_uda_selection_bold(data: &str) -> bool {
    Self::get_config("uda.taskwarrior-tui.selection.bold", data)
      .unwrap_or_default()
      .get_bool()
      .unwrap_or(true)
  }

  fn get_uda_selection_italic(data: &str) -> bool {
    Self::get_config("uda.taskwarrior-tui.selection.italic", data)
      .unwrap_or_default()
      .get_bool()
      .unwrap_or(false)
  }

  fn get_uda_selection_dim(data: &str) -> bool {
    Self::get_config("uda.taskwarrior-tui.selection.dim", data)
      .unwrap_or_default()
      .get_bool()
      .unwrap_or(false)
  }

  fn get_uda_selection_blink(data: &str) -> bool {
    Self::get_config("uda.taskwarrior-tui.selection.blink", data)
      .unwrap_or_default()
      .get_bool()
      .unwrap_or(false)
  }

  fn get_uda_selection_reverse(data: &str) -> bool {
    Self::get_config("uda.taskwarrior-tui.selection.reverse", data)
      .unwrap_or_default()
      .get_bool()
      .unwrap_or(false)
  }

  fn get_uda_months_per_row(data: &str) -> usize {
    Self::get_config("uda.taskwarrior-tui.calendar.months-per-row", data)
      .unwrap_or_default()
      .parse::<usize>()
      .unwrap_or(4)
  }

  fn get_uda_quick_tag_name(data: &str) -> String {
    let tag_name = Self::get_config("uda.taskwarrior-tui.quick-tag.name", data);
    match tag_name {
      None => "next".to_string(),
      Some(tag_name) => tag_name,
    }
  }

  fn get_uda_tasklist_vertical(data: &str) -> bool {
    Self::get_config("uda.taskwarrior-tui.tasklist.vertical", data)
      .unwrap_or_default()
      .get_bool()
      // Vertical mode is disabled by default if the option is not set and the terminal is not wide
      // enough.
      .unwrap_or(size().unwrap_or((50, 15)).0 <= 160)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_config_collects_keyword_colors() {
    let data = [
      "data.location /tmp/taskwarrior-tui-tests",
      "rule.precedence.color keyword.,tag.,project.",
      "uda.priority.values H,M,L,",
      "report.next.filter status:pending",
      "color.keyword.fixme red on blue",
      "color.keyword.todo underline yellow",
      "color.active green",
    ]
    .join("\n");

    let config = Config::new(&data, "next").unwrap();

    assert_eq!(config.color_keywords.len(), 2);
    assert!(config.color_keywords.contains(&("fixme".to_string(), Config::get_tcolor("red on blue"))));
    assert!(
      config
        .color_keywords
        .contains(&("todo".to_string(), Config::get_tcolor("underline yellow")))
    );
  }

  #[test]
  fn test_config_collects_uda_colors_with_spaces_in_key() {
    let data = [
      "data.location /tmp/taskwarrior-tui-tests",
      "rule.precedence.color uda.,tag.,project.",
      "uda.priority.values H,M,L,",
      "report.next.filter status:pending",
      "color.uda.jirastatus.In Review  black on bright cyan",
      "color.uda.jirastatus.To Do  bright white",
    ]
    .join("\n");

    let config = Config::new(&data, "next").unwrap();

    assert_eq!(
      config.color.get("color.uda.jirastatus.In Review"),
      Some(&Config::get_tcolor("black on bright cyan"))
    );
    assert_eq!(config.color.get("color.uda.jirastatus.To Do"), Some(&Config::get_tcolor("bright white")));
  }

  #[test]
  fn test_named_colors_and_backgrounds() {
    // --- Basic named colors ---
    let c = Config::get_tcolor("red");
    assert_eq!(c.fg.unwrap(), Color::Indexed(1));
    assert!(c.bg.is_none());

    let c = Config::get_tcolor("blue");
    assert_eq!(c.fg.unwrap(), Color::Indexed(4));
    assert!(c.bg.is_none());

    let c = Config::get_tcolor("white");
    assert_eq!(c.fg.unwrap(), Color::Indexed(7));
    assert!(c.bg.is_none());

    let c = Config::get_tcolor("green");
    assert_eq!(c.fg.unwrap(), Color::Indexed(2));
    assert!(c.bg.is_none());

    let c = Config::get_tcolor("yellow");
    assert_eq!(c.fg.unwrap(), Color::Indexed(3));
    assert!(c.bg.is_none());

    let c = Config::get_tcolor("magenta");
    assert_eq!(c.fg.unwrap(), Color::Indexed(5));
    assert!(c.bg.is_none());

    let c = Config::get_tcolor("cyan");
    assert_eq!(c.fg.unwrap(), Color::Indexed(6));
    assert!(c.bg.is_none());

    let c = Config::get_tcolor("black");
    assert_eq!(c.fg.unwrap(), Color::Indexed(0));
    assert!(c.bg.is_none());

    // --- Foreground + background (named) ---
    let c = Config::get_tcolor("red on blue");
    assert_eq!(c.fg.unwrap(), Color::Indexed(1));
    assert_eq!(c.bg.unwrap(), Color::Indexed(4));

    let c = Config::get_tcolor("white on red");
    assert_eq!(c.fg.unwrap(), Color::Indexed(7));
    assert_eq!(c.bg.unwrap(), Color::Indexed(1));

    let c = Config::get_tcolor("white on green");
    assert_eq!(c.fg.unwrap(), Color::Indexed(7));
    assert_eq!(c.bg.unwrap(), Color::Indexed(2));

    let c = Config::get_tcolor("black on white");
    assert_eq!(c.fg.unwrap(), Color::Indexed(0));
    assert_eq!(c.bg.unwrap(), Color::Indexed(7));

    let c = Config::get_tcolor("black on red");
    assert_eq!(c.fg.unwrap(), Color::Indexed(0));
    assert_eq!(c.bg.unwrap(), Color::Indexed(1));

    let c = Config::get_tcolor("black on yellow");
    assert_eq!(c.fg.unwrap(), Color::Indexed(0));
    assert_eq!(c.bg.unwrap(), Color::Indexed(3));

    let c = Config::get_tcolor("black on green");
    assert_eq!(c.fg.unwrap(), Color::Indexed(0));
    assert_eq!(c.bg.unwrap(), Color::Indexed(2));

    let c = Config::get_tcolor("white on black");
    assert_eq!(c.fg.unwrap(), Color::Indexed(7));
    assert_eq!(c.bg.unwrap(), Color::Indexed(0));

    // --- Background-only ---
    let c = Config::get_tcolor("on green");
    assert!(c.fg.is_none());
    assert_eq!(c.bg.unwrap(), Color::Indexed(2));

    let c = Config::get_tcolor("on red");
    assert!(c.fg.is_none());
    assert_eq!(c.bg.unwrap(), Color::Indexed(1));

    let c = Config::get_tcolor("on yellow");
    assert!(c.fg.is_none());
    assert_eq!(c.bg.unwrap(), Color::Indexed(3));

    // --- bright backgrounds (high-intensity, indices 8-15) ---
    let c = Config::get_tcolor("black on bright green");
    assert_eq!(c.fg.unwrap(), Color::Indexed(0));
    assert_eq!(c.bg.unwrap(), Color::Indexed(10));

    let c = Config::get_tcolor("black on bright white");
    assert_eq!(c.fg.unwrap(), Color::Indexed(0));
    assert_eq!(c.bg.unwrap(), Color::Indexed(15));

    let c = Config::get_tcolor("black on bright yellow");
    assert_eq!(c.fg.unwrap(), Color::Indexed(0));
    assert_eq!(c.bg.unwrap(), Color::Indexed(11));

    let c = Config::get_tcolor("black on bright red");
    assert_eq!(c.fg.unwrap(), Color::Indexed(0));
    assert_eq!(c.bg.unwrap(), Color::Indexed(9));

    let c = Config::get_tcolor("white on bright black");
    assert_eq!(c.fg.unwrap(), Color::Indexed(7));
    assert_eq!(c.bg.unwrap(), Color::Indexed(8));
  }

  #[test]
  fn test_modifiers() {
    // --- underline modifier ---
    let c = Config::get_tcolor("underline red");
    assert_eq!(c.fg.unwrap(), Color::Indexed(1));
    assert_eq!(c.add_modifier & Modifier::UNDERLINED, Modifier::UNDERLINED);

    let c = Config::get_tcolor("underline bold red on blue");
    assert_eq!(c.fg.unwrap(), Color::Indexed(1));
    assert_eq!(c.bg.unwrap(), Color::Indexed(4));
    assert_eq!(c.add_modifier & Modifier::UNDERLINED, Modifier::UNDERLINED);
    assert_eq!(c.add_modifier & Modifier::BOLD, Modifier::BOLD);

    // --- inverse modifier ---
    let c = Config::get_tcolor("inverse red");
    assert_eq!(c.fg.unwrap(), Color::Indexed(1));
    assert_eq!(c.add_modifier & Modifier::REVERSED, Modifier::REVERSED);

    // --- italic modifier ---
    let c = Config::get_tcolor("italic red");
    assert_eq!(c.fg.unwrap(), Color::Indexed(1));
    assert_eq!(c.add_modifier & Modifier::ITALIC, Modifier::ITALIC);

    let c = Config::get_tcolor("italic red on blue");
    assert_eq!(c.fg.unwrap(), Color::Indexed(1));
    assert_eq!(c.bg.unwrap(), Color::Indexed(4));
    assert_eq!(c.add_modifier & Modifier::ITALIC, Modifier::ITALIC);

    // --- strikethrough modifier ---
    let c = Config::get_tcolor("strikethrough red");
    assert_eq!(c.fg.unwrap(), Color::Indexed(1));
    assert_eq!(c.add_modifier & Modifier::CROSSED_OUT, Modifier::CROSSED_OUT);

    // --- combined modifiers ---
    let c = Config::get_tcolor("bold underline italic red on blue");
    assert_eq!(c.fg.unwrap(), Color::Indexed(1));
    assert_eq!(c.bg.unwrap(), Color::Indexed(4));
    assert_eq!(c.add_modifier & Modifier::BOLD, Modifier::BOLD);
    assert_eq!(c.add_modifier & Modifier::UNDERLINED, Modifier::UNDERLINED);
    assert_eq!(c.add_modifier & Modifier::ITALIC, Modifier::ITALIC);
  }

  #[test]
  fn test_256_grayscale_rgb() {
    // --- 256-color indexed ---
    let c = Config::get_tcolor("color0");
    assert_eq!(c.fg.unwrap(), Color::Indexed(0));

    let c = Config::get_tcolor("color255");
    assert_eq!(c.fg.unwrap(), Color::Indexed(255));

    let c = Config::get_tcolor("color100");
    assert_eq!(c.fg.unwrap(), Color::Indexed(100));

    // colorN background (previously broken by wrapping_shl bug)
    let c = Config::get_tcolor("white on color196");
    assert_eq!(c.fg.unwrap(), Color::Indexed(7));
    assert_eq!(c.bg.unwrap(), Color::Indexed(196));

    // --- grayscale ramp (gray0-gray23 -> indexed 232-255) ---
    let c = Config::get_tcolor("gray0");
    assert_eq!(c.fg.unwrap(), Color::Indexed(232));

    let c = Config::get_tcolor("gray23");
    assert_eq!(c.fg.unwrap(), Color::Indexed(255));

    let c = Config::get_tcolor("gray5");
    assert_eq!(c.fg.unwrap(), Color::Indexed(237));

    // grey spelling variant
    let c = Config::get_tcolor("grey5");
    assert_eq!(c.fg.unwrap(), Color::Indexed(237));

    // gray background (previously broken by wrapping_shl bug)
    let c = Config::get_tcolor("white on gray5");
    assert_eq!(c.fg.unwrap(), Color::Indexed(7));
    assert_eq!(c.bg.unwrap(), Color::Indexed(237));

    let c = Config::get_tcolor("on gray23");
    assert!(c.fg.is_none());
    assert_eq!(c.bg.unwrap(), Color::Indexed(255));

    // --- RGB color cube (rgb000-rgb555 -> indexed 16-231) ---
    // rgb500 = 16 + 5*36 + 0*6 + 0 = 196
    let c = Config::get_tcolor("rgb500");
    assert_eq!(c.fg.unwrap(), Color::Indexed(196));

    // rgb050 = 16 + 0*36 + 5*6 + 0 = 46
    let c = Config::get_tcolor("rgb050");
    assert_eq!(c.fg.unwrap(), Color::Indexed(46));

    // rgb005 = 16 + 0 + 0 + 5 = 21
    let c = Config::get_tcolor("rgb005");
    assert_eq!(c.fg.unwrap(), Color::Indexed(21));

    // rgb550 = 16 + 5*36 + 5*6 + 0 = 226
    let c = Config::get_tcolor("rgb550");
    assert_eq!(c.fg.unwrap(), Color::Indexed(226));

    // rgb444 = 16 + 4*36 + 4*6 + 4 = 16 + 144 + 24 + 4 = 188
    let c = Config::get_tcolor("rgb444");
    assert_eq!(c.fg.unwrap(), Color::Indexed(188));

    // rgb background (previously broken by wrapping_shl bug)
    let c = Config::get_tcolor("white on rgb500");
    assert_eq!(c.fg.unwrap(), Color::Indexed(7));
    assert_eq!(c.bg.unwrap(), Color::Indexed(196));

    let c = Config::get_tcolor("on rgb444");
    assert!(c.fg.is_none());
    assert_eq!(c.bg.unwrap(), Color::Indexed(188));

    // --- TokyoNight Moon representative colors ---
    // color111 on color60
    let c = Config::get_tcolor("color111 on color60");
    assert_eq!(c.fg.unwrap(), Color::Indexed(111));
    assert_eq!(c.bg.unwrap(), Color::Indexed(60));

    // color203 (overdue/error red)
    let c = Config::get_tcolor("color203");
    assert_eq!(c.fg.unwrap(), Color::Indexed(203));
  }

  /// Regression test: "bold red" must produce Indexed(1) + BOLD, not Indexed(9).
  #[test]
  fn test_bold_not_bright_regression() {
    let c = Config::get_tcolor("bold red");
    assert_eq!(c.fg.unwrap(), Color::Indexed(1));
    assert!(c.bg.is_none());
    assert_eq!(c.add_modifier & Modifier::BOLD, Modifier::BOLD);

    let c = Config::get_tcolor("bold white");
    assert_eq!(c.fg.unwrap(), Color::Indexed(7));
    assert!(c.bg.is_none());
    assert_eq!(c.add_modifier & Modifier::BOLD, Modifier::BOLD);

    let c = Config::get_tcolor("bold yellow");
    assert_eq!(c.fg.unwrap(), Color::Indexed(3));
    assert!(c.bg.is_none());
    assert_eq!(c.add_modifier & Modifier::BOLD, Modifier::BOLD);

    let c = Config::get_tcolor("bold blue");
    assert_eq!(c.fg.unwrap(), Color::Indexed(4));
    assert!(c.bg.is_none());
    assert_eq!(c.add_modifier & Modifier::BOLD, Modifier::BOLD);

    let c = Config::get_tcolor("bold white on red");
    assert_eq!(c.fg.unwrap(), Color::Indexed(7));
    assert_eq!(c.bg.unwrap(), Color::Indexed(1));
    assert_eq!(c.add_modifier & Modifier::BOLD, Modifier::BOLD);

    let c = Config::get_tcolor("bold white on bright blue");
    assert_eq!(c.fg.unwrap(), Color::Indexed(7));
    assert_eq!(c.bg.unwrap(), Color::Indexed(12));
    assert_eq!(c.add_modifier & Modifier::BOLD, Modifier::BOLD);
  }

  /// Regression test: trailing modifiers must not
  /// cause the foreground color to be silently dropped.
  #[test]
  fn test_trailing_modifier_regression() {
    // Single trailing modifier
    let c = Config::get_tcolor("color209 bold");
    assert_eq!(c.fg.unwrap(), Color::Indexed(209));
    assert_eq!(c.add_modifier & Modifier::BOLD, Modifier::BOLD);

    let c = Config::get_tcolor("color117 bold");
    assert_eq!(c.fg.unwrap(), Color::Indexed(117));
    assert_eq!(c.add_modifier & Modifier::BOLD, Modifier::BOLD);

    let c = Config::get_tcolor("color111 bold");
    assert_eq!(c.fg.unwrap(), Color::Indexed(111));
    assert_eq!(c.add_modifier & Modifier::BOLD, Modifier::BOLD);

    let c = Config::get_tcolor("color61 underline");
    assert_eq!(c.fg.unwrap(), Color::Indexed(61));
    assert_eq!(c.add_modifier & Modifier::UNDERLINED, Modifier::UNDERLINED);

    let c = Config::get_tcolor("color210 underline");
    assert_eq!(c.fg.unwrap(), Color::Indexed(210));
    assert_eq!(c.add_modifier & Modifier::UNDERLINED, Modifier::UNDERLINED);

    let c = Config::get_tcolor("red italic");
    assert_eq!(c.fg.unwrap(), Color::Indexed(1));
    assert_eq!(c.add_modifier & Modifier::ITALIC, Modifier::ITALIC);

    let c = Config::get_tcolor("blue inverse");
    assert_eq!(c.fg.unwrap(), Color::Indexed(4));
    assert_eq!(c.add_modifier & Modifier::REVERSED, Modifier::REVERSED);

    let c = Config::get_tcolor("green strikethrough");
    assert_eq!(c.fg.unwrap(), Color::Indexed(2));
    assert_eq!(c.add_modifier & Modifier::CROSSED_OUT, Modifier::CROSSED_OUT);

    // Two trailing modifiers
    let c = Config::get_tcolor("color150 bold underline");
    assert_eq!(c.fg.unwrap(), Color::Indexed(150));
    assert_eq!(c.add_modifier & Modifier::BOLD, Modifier::BOLD);
    assert_eq!(c.add_modifier & Modifier::UNDERLINED, Modifier::UNDERLINED);

    let c = Config::get_tcolor("color210 bold underline");
    assert_eq!(c.fg.unwrap(), Color::Indexed(210));
    assert_eq!(c.add_modifier & Modifier::BOLD, Modifier::BOLD);
    assert_eq!(c.add_modifier & Modifier::UNDERLINED, Modifier::UNDERLINED);

    // Trailing modifier with background
    let c = Config::get_tcolor("color150 bold on color236");
    assert_eq!(c.fg.unwrap(), Color::Indexed(150));
    assert_eq!(c.bg.unwrap(), Color::Indexed(236));
    assert_eq!(c.add_modifier & Modifier::BOLD, Modifier::BOLD);

    let c = Config::get_tcolor("color210 underline on color234");
    assert_eq!(c.fg.unwrap(), Color::Indexed(210));
    assert_eq!(c.bg.unwrap(), Color::Indexed(234));
    assert_eq!(c.add_modifier & Modifier::UNDERLINED, Modifier::UNDERLINED);

    // Mixed: leading + trailing modifiers
    let c = Config::get_tcolor("bold color150 underline");
    assert_eq!(c.fg.unwrap(), Color::Indexed(150));
    assert_eq!(c.add_modifier & Modifier::BOLD, Modifier::BOLD);
    assert_eq!(c.add_modifier & Modifier::UNDERLINED, Modifier::UNDERLINED);
  }

  #[test]
  fn test_get_config_long_value() {
    let config = Config::get_config(
      "report.test.filter",
      "report.test.description test\nreport.test.filter filter and\n                   test\nreport.test.columns=id",
    );
    assert_eq!(config.unwrap(), "filter and test");
  }

  #[test]
  fn test_get_config_long_value_followed_by_default_value() {
    let config = Config::get_config(
      "report.test.filter",
      "report.test.description test\nreport.test.filter filter and\n                   test\n  Default value test",
    );
    assert_eq!(config.unwrap(), "filter and test");
  }

  #[test]
  fn test_get_config_last_long_value() {
    let config = Config::get_config(
      "report.test.filter",
      "report.test.description test\nreport.test.filter filter and\n                   test",
    );
    assert_eq!(config.unwrap(), "filter and test");
  }

  #[test]
  fn test_get_uda_task_report_use_alternate_style_defaults_to_true() {
    assert!(Config::get_uda_task_report_use_alternate_style(""));
  }

  #[test]
  fn test_get_uda_task_report_use_alternate_style_can_be_disabled() {
    let data = "uda.taskwarrior-tui.task-report.use-alternate-style false";
    assert!(!Config::get_uda_task_report_use_alternate_style(data));
  }

  #[test]
  fn test_get_uda_report_menu_close_on_select_defaults_to_true() {
    assert!(Config::get_uda_report_menu_close_on_select(""));
  }

  #[test]
  fn test_get_uda_report_menu_close_on_select_can_be_disabled() {
    let data = "uda.taskwarrior-tui.report-menu.close-on-select false";
    assert!(!Config::get_uda_report_menu_close_on_select(data));
  }

  #[test]
  fn test_get_uda_context_menu_close_on_select_defaults_to_true() {
    assert!(Config::get_uda_context_menu_close_on_select(""));
  }

  #[test]
  fn test_get_uda_context_menu_close_on_select_can_be_disabled() {
    let data = "uda.taskwarrior-tui.context-menu.close-on-select false";
    assert!(!Config::get_uda_context_menu_close_on_select(data));
  }
}
