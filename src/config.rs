use std::{collections::HashMap, error::Error, str};

use color_eyre::eyre::{eyre, Context, Result};
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
  pub uda_style_report_scrollbar: Style,
  pub uda_style_report_scrollbar_area: Style,
  pub uda_selection_bold: bool,
  pub uda_selection_italic: bool,
  pub uda_selection_dim: bool,
  pub uda_selection_blink: bool,
  pub uda_selection_reverse: bool,
  pub uda_calendar_months_per_row: usize,
  pub uda_style_context_active: Style,
  pub uda_style_report_selection: Style,
  pub uda_style_calendar_title: Style,
  pub uda_style_calendar_today: Style,
  pub uda_style_navbar: Style,
  pub uda_style_command: Style,
  pub uda_style_report_completion_pane: Style,
  pub uda_style_report_completion_pane_highlight: Style,
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
  pub uda: Vec<Uda>,
}

impl Config {
  pub fn new(data: &str, report: &str) -> Result<Self> {
    let bool_collection = Self::get_bool_collection();

    let enabled = true;
    let obfuscate = bool_collection.get("obfuscate").copied().unwrap_or(false);
    let print_empty_columns = bool_collection.get("print_empty_columns").copied().unwrap_or(false);

    let color = Self::get_color_collection(data);
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
    let uda_task_report_show_info = Self::get_uda_task_report_show_info(data);
    let uda_task_report_looping = Self::get_uda_task_report_looping(data);
    let uda_task_report_jump_to_task_on_add = Self::get_uda_task_report_jump_to_task_on_add(data);
    let uda_selection_indicator = Self::get_uda_selection_indicator(data);
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
    let uda_style_report_completion_pane = Self::get_uda_style("report.completion-pane", data);
    let uda_style_report_completion_pane_highlight = Self::get_uda_style("report.completion-pane-highlight", data);
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
    let uda_style_context_active = uda_style_context_active.unwrap_or_default();
    let uda_style_report_completion_pane =
      uda_style_report_completion_pane.unwrap_or_else(|| Style::default().fg(Color::Black).bg(Color::Rgb(223, 223, 223)));
    let uda_style_report_completion_pane_highlight = uda_style_report_completion_pane_highlight.unwrap_or(uda_style_report_completion_pane);
    let uda_quick_tag_name = Self::get_uda_quick_tag_name(data);
    let uda_task_report_prompt_on_undo = Self::get_uda_task_report_prompt_on_undo(data);
    let uda_task_report_prompt_on_delete = Self::get_uda_task_report_prompt_on_delete(data);
    let uda_task_report_prompt_on_done = Self::get_uda_task_report_prompt_on_done(data);
    let uda_context_menu_select_on_move = Self::get_uda_context_menu_select_on_move(data);
    let uda_task_report_date_time_vague_more_precise = Self::get_uda_task_report_date_time_vague_more_precise(data);

    Ok(Self {
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
      uda_change_focus_rotate,
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
      uda_selection_bold,
      uda_selection_italic,
      uda_selection_dim,
      uda_selection_blink,
      uda_selection_reverse,
      uda_calendar_months_per_row,
      uda_style_report_selection,
      uda_style_context_active,
      uda_style_calendar_title,
      uda_style_calendar_today,
      uda_style_navbar,
      uda_style_command,
      uda_style_report_completion_pane,
      uda_style_report_completion_pane_highlight,
      uda_style_report_scrollbar,
      uda_style_report_scrollbar_area,
      uda_shortcuts,
      uda_background_process,
      uda_background_process_period,
      uda_quick_tag_name,
      uda_task_report_prompt_on_undo,
      uda_task_report_prompt_on_delete,
      uda_task_report_prompt_on_done,
      uda_task_report_date_time_vague_more_precise,
      uda_context_menu_select_on_move,
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
      if line.starts_with("color.") {
        let mut i = line.split(' ');
        let attribute = i.next();
        let line = i.collect::<Vec<_>>().join(" ");
        let line = line.trim_start_matches(' ');
        let tcolor = Self::get_tcolor(line);
        if let Some(attr) = attribute {
          color_collection.insert(attr.to_string(), tcolor);
        };
      }
    }
    color_collection
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
    if let Some(fg) = Self::get_color_foreground(foreground.as_str()) {
      style = style.fg(fg);
    }
    if let Some(bg) = Self::get_color_background(background.as_str()) {
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
      .ok_or_else(|| eyre!("Unable to parse `task show rule.precedence.color`."))
      .unwrap();
    data.split(',').map(ToString::to_string).collect::<Vec<_>>()
  }

  fn get_uda_priority_values(data: &str) -> Vec<String> {
    let data = Self::get_config("uda.priority.values", data)
      .ok_or_else(|| eyre!("Unable to parse `task show uda.priority.values`."))
      .unwrap();
    data.split(',').map(ToString::to_string).collect::<Vec<_>>()
  }

  fn get_filter(data: &str, report: &str) -> Result<String> {
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
      .ok_or_else(|| eyre!("Unable to parse `task show data.location`."))
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
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_colors() {
    let c = Config::get_tcolor("red on blue");
    assert_eq!(c.fg.unwrap(), Color::Indexed(1));
    assert_eq!(c.bg.unwrap(), Color::Indexed(4));

    let c = Config::get_tcolor("bold red");
    assert_eq!(c.fg.unwrap(), Color::Indexed(9));
    assert!(c.bg.is_none());
    assert_eq!(c.add_modifier & Modifier::BOLD, Modifier::BOLD);

    let c = Config::get_tcolor("white on red");
    assert_eq!(c.fg.unwrap(), Color::Indexed(7));
    assert_eq!(c.bg.unwrap(), Color::Indexed(1));

    let c = Config::get_tcolor("blue");
    assert_eq!(c.fg.unwrap(), Color::Indexed(4));
    assert!(c.bg.is_none());

    let c = Config::get_tcolor("black on bright green");
    assert_eq!(c.fg.unwrap(), Color::Indexed(0));
    assert_eq!(c.bg.unwrap(), Color::Indexed(10));

    let c = Config::get_tcolor("magenta");
    assert_eq!(c.fg.unwrap(), Color::Indexed(5));
    assert!(c.bg.is_none());

    let c = Config::get_tcolor("white on green");
    assert_eq!(c.fg.unwrap(), Color::Indexed(7));
    assert_eq!(c.bg.unwrap(), Color::Indexed(2));

    let c = Config::get_tcolor("black on white");
    assert_eq!(c.fg.unwrap(), Color::Indexed(0));
    assert_eq!(c.bg.unwrap(), Color::Indexed(7));

    let c = Config::get_tcolor("black on bright white");
    assert_eq!(c.fg.unwrap(), Color::Indexed(0));
    assert_eq!(c.bg.unwrap(), Color::Indexed(15));

    let c = Config::get_tcolor("bold white");
    assert_eq!(c.fg.unwrap(), Color::Indexed(15));
    assert!(c.bg.is_none());

    let c = Config::get_tcolor("white");
    assert_eq!(c.fg.unwrap(), Color::Indexed(7));
    assert!(c.bg.is_none());

    let c = Config::get_tcolor("bold yellow");
    assert_eq!(c.fg.unwrap(), Color::Indexed(11));
    assert!(c.bg.is_none());

    let c = Config::get_tcolor("green");
    assert_eq!(c.fg.unwrap(), Color::Indexed(2));
    assert!(c.bg.is_none());

    let c = Config::get_tcolor("yellow");
    assert_eq!(c.fg.unwrap(), Color::Indexed(3));
    assert!(c.bg.is_none());

    let c = Config::get_tcolor("red");
    assert_eq!(c.fg.unwrap(), Color::Indexed(1));
    assert!(c.bg.is_none());

    let c = Config::get_tcolor("bold red");
    assert_eq!(c.fg.unwrap(), Color::Indexed(9));
    assert!(c.bg.is_none());

    let c = Config::get_tcolor("on green");
    assert!(c.fg.is_none());
    assert_eq!(c.bg.unwrap(), Color::Indexed(2));

    let c = Config::get_tcolor("on red");
    assert!(c.fg.is_none());
    assert_eq!(c.bg.unwrap(), Color::Indexed(1));

    let c = Config::get_tcolor("on yellow");
    assert!(c.fg.is_none());
    assert_eq!(c.bg.unwrap(), Color::Indexed(3));

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

    let c = Config::get_tcolor("black on green");
    assert_eq!(c.fg.unwrap(), Color::Indexed(0));
    assert_eq!(c.bg.unwrap(), Color::Indexed(2));

    let c = Config::get_tcolor("white on red");
    assert_eq!(c.fg.unwrap(), Color::Indexed(7));
    assert_eq!(c.bg.unwrap(), Color::Indexed(1));

    let c = Config::get_tcolor("bold white on red");
    assert_eq!(c.fg.unwrap(), Color::Indexed(15));
    assert_eq!(c.bg.unwrap(), Color::Indexed(1));

    let c = Config::get_tcolor("black on bright yellow");
    assert_eq!(c.fg.unwrap(), Color::Indexed(0));
    assert_eq!(c.bg.unwrap(), Color::Indexed(11));

    let c = Config::get_tcolor("black on bright red");
    assert_eq!(c.fg.unwrap(), Color::Indexed(0));
    assert_eq!(c.bg.unwrap(), Color::Indexed(9));

    let c = Config::get_tcolor("bold white on bright blue");
    assert_eq!(c.fg.unwrap(), Color::Indexed(15));
    assert_eq!(c.bg.unwrap(), Color::Indexed(12));

    let c = Config::get_tcolor("white on bright black");
    assert_eq!(c.fg.unwrap(), Color::Indexed(7));
    assert_eq!(c.bg.unwrap(), Color::Indexed(8));

    let c = Config::get_tcolor("bold blue");
    assert_eq!(c.fg.unwrap(), Color::Indexed(12));
    assert!(c.bg.is_none());
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
}
