use std::{
  borrow::Borrow,
  cmp::Ordering,
  collections::{HashMap, HashSet},
  convert::TryInto,
  fs, io,
  io::{Read, Write},
  path::{Path, PathBuf},
  sync::{mpsc, Arc, Mutex},
  time::{Duration, Instant, SystemTime},
};

use chrono::{DateTime, Datelike, FixedOffset, Local, NaiveDate, NaiveDateTime, TimeZone, Timelike};
use color_eyre::eyre::{anyhow, Context, Result};
use crossterm::{
  event::{DisableMouseCapture, EnableMouseCapture, KeyCode, KeyEvent, KeyModifiers},
  execute,
  style::style,
  terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::SinkExt;
use lazy_static::lazy_static;
use log::{debug, error, info, log_enabled, trace, warn, Level, LevelFilter};
use ratatui::{
  backend::{Backend, CrosstermBackend},
  layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
  style::{Color, Modifier, Style},
  symbols::bar::FULL,
  terminal::Frame,
  text::{Line, Span, Text},
  widgets::{Block, BorderType, Borders, Clear, Gauge, LineGauge, List, ListItem, Paragraph, Tabs, Wrap},
  Terminal,
};
use regex::Regex;
use rustyline::{history::SearchDirection as HistoryDirection, At, Editor, Word};
use task_hookrs::{date::Date, import::import, project::Project, status::TaskStatus, task::Task};
use tui_input::{backend::crossterm::EventHandler, Input};
use unicode_segmentation::{Graphemes, UnicodeSegmentation};
use unicode_width::UnicodeWidthStr;
use uuid::Uuid;
use versions::Versioning;

use crate::{
  action::Action,
  calendar::Calendar,
  completion::{get_start_word_under_cursor, CompletionList},
  config,
  config::Config,
  help::Help,
  history::HistoryContext,
  keyconfig::KeyConfig,
  pane::{
    context::{ContextDetails, ContextsState},
    project::ProjectsState,
    Pane,
  },
  scrollbar::Scrollbar,
  table::{Row, Table, TableMode, TableState},
  task_report::TaskReportTable,
  trace_dbg,
  traits::TaskwarriorTuiTask,
  tui::{self, Event},
  ui,
  utils::{self, get_data_dir},
};

const MAX_LINE: usize = 4096;

lazy_static! {
  static ref START_TIME: Instant = Instant::now();
  static ref TASKWARRIOR_VERSION_SUPPORTED: Versioning = Versioning::new("2.6.0").unwrap();
}

#[derive(Debug)]
pub enum DateState {
  BeforeToday,
  EarlierToday,
  LaterToday,
  AfterToday,
  NotDue,
}

pub fn get_date_state(reference: &Date, due: usize) -> DateState {
  let now = Local::now();
  let reference = TimeZone::from_utc_datetime(now.offset(), reference);
  let now = TimeZone::from_utc_datetime(now.offset(), &now.naive_utc());

  if reference.date_naive() < now.date_naive() {
    return DateState::BeforeToday;
  }

  if reference.date_naive() == now.date_naive() {
    return if reference.time() < now.time() {
      DateState::EarlierToday
    } else {
      DateState::LaterToday
    };
  }

  if reference <= now + chrono::Duration::days(7) {
    DateState::AfterToday
  } else {
    DateState::NotDue
  }
}

fn get_offset_hour_minute() -> (&'static str, i32, i32) {
  let off = Local::now().offset().local_minus_utc();
  let sym = if off >= 0 { "+" } else { "-" };
  let off = off.abs();
  let h = if off > 60 * 60 { off / 60 / 60 } else { 0 };
  let m = if (off - ((off / 60 / 60) * 60 * 60)) > 60 {
    (off - ((off / 60 / 60) * 60 * 60)) / 60
  } else {
    0
  };
  (sym, h, m)
}

fn get_formatted_datetime(date: &Date) -> String {
  let now = Local::now();
  let date = TimeZone::from_utc_datetime(now.offset(), date);
  let (sym, h, m) = get_offset_hour_minute();
  format!(
    "'{:04}-{:02}-{:02}T{:02}:{:02}:{:02}{}{:02}:{:02}'",
    date.year(),
    date.month(),
    date.day(),
    date.hour(),
    date.minute(),
    date.second(),
    sym,
    h,
    m,
  )
}

fn centered_rect(r: Rect, percent_x: u16, percent_y: u16) -> Rect {
  let popup_layout = Layout::default()
    .direction(Direction::Vertical)
    .constraints(
      [
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
      ]
      .as_ref(),
    )
    .split(r);

  Layout::default()
    .direction(Direction::Horizontal)
    .constraints(
      [
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
      ]
      .as_ref(),
    )
    .split(popup_layout[1])[1]
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
  TaskReport,
  TaskFilter,
  TaskAdd,
  TaskAnnotate,
  TaskSubprocess,
  TaskLog,
  TaskModify,
  TaskHelpPopup,
  TaskContextMenu,
  TaskJump,
  TaskDeletePrompt,
  TaskUndoPrompt,
  TaskDonePrompt,
  TaskError,
  Projects,
  Calendar,
}

pub struct TaskwarriorTui {
  pub tick_rate: u64,
  pub should_quit: bool,
  pub dirty: bool,
  pub task_table_state: TableState,
  pub current_context_filter: String,
  pub current_context: String,
  pub command: Input,
  pub filter: Input,
  pub modify: Input,
  pub tasks: Vec<Task>,
  pub all_tasks: Vec<Task>,
  pub task_details: HashMap<Uuid, String>,
  pub marked: HashSet<Uuid>,
  pub current_selection: usize,
  pub current_selection_uuid: Option<Uuid>,
  pub current_selection_id: Option<u64>,
  pub task_report_table: TaskReportTable,
  pub calendar_year: i32,
  pub mode: Mode,
  pub previous_mode: Option<Mode>,
  pub config: Config,
  pub task_report_show_info: bool,
  pub task_report_height: u16,
  pub task_details_scroll: u16,
  pub help_popup: Help,
  pub last_export: Option<SystemTime>,
  pub keyconfig: KeyConfig,
  pub terminal_width: u16,
  pub terminal_height: u16,
  pub filter_history: HistoryContext,
  pub command_history: HistoryContext,
  pub history_status: Option<String>,
  pub completion_list: CompletionList,
  pub show_completion_pane: bool,
  pub report: String,
  pub projects: ProjectsState,
  pub contexts: ContextsState,
  pub task_version: Versioning,
  pub error: Option<String>,
  pub requires_redraw: bool,
  pub changes: utils::Changeset,
}

impl TaskwarriorTui {
  pub fn new(report: &str) -> Result<Self> {
    let output = std::process::Command::new("task")
      .arg("rc.color=off")
      .arg("rc._forcecolor=off")
      .arg("rc.defaultwidth=0")
      .arg("show")
      .output()
      .context("Unable to run `task show`.")?;

    if !output.status.success() {
      let output = std::process::Command::new("task")
        .arg("diagnostics")
        .output()
        .context("Unable to run `task diagnostics`.")?;
      return Err(anyhow!(
        "Unable to run `task show`.\n{}\n{}\nPlease check your configuration or open a issue on github.",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
      ));
    }

    let data = String::from_utf8_lossy(&output.stdout);
    let c = Config::new()?;
    let kc = KeyConfig::new(&data)?;

    let output = std::process::Command::new("task")
      .arg("--version")
      .output()
      .context("Unable to run `task --version`")?;

    let task_version =
      Versioning::new(String::from_utf8_lossy(&output.stdout).trim()).ok_or(anyhow!("Unable to get version string"))?;

    let (w, h) = crossterm::terminal::size().unwrap_or((50, 15));

    let data_dir = get_data_dir();

    let mut app = Self {
      tick_rate: c.uda_tick_rate,
      should_quit: false,
      dirty: true,
      task_table_state: TableState::default(),
      tasks: vec![],
      all_tasks: vec![],
      task_details: HashMap::new(),
      marked: HashSet::new(),
      current_selection: 0,
      current_selection_uuid: None,
      current_selection_id: None,
      current_context_filter: "".to_string(),
      current_context: "".to_string(),
      command: Input::default(),
      filter: Input::default(),
      modify: Input::default(),
      mode: Mode::TaskReport,
      previous_mode: None,
      task_report_height: 0,
      task_details_scroll: 0,
      task_report_show_info: c.uda_task_report_show_info,
      config: c,
      task_report_table: TaskReportTable::new(&data, report)?,
      calendar_year: Local::now().year(),
      help_popup: Help::new(),
      last_export: None,
      keyconfig: kc,
      terminal_width: w,
      terminal_height: h,
      filter_history: HistoryContext::new("filter.history", data_dir.clone()),
      command_history: HistoryContext::new("command.history", data_dir.clone()),
      history_status: None,
      completion_list: CompletionList::with_items(vec![]),
      show_completion_pane: false,
      report: report.to_string(),
      projects: ProjectsState::new(),
      contexts: ContextsState::new(),
      task_version,
      error: None,
      requires_redraw: false,
      changes: utils::Changeset::default(),
    };

    app.filter = app.filter.with_value(app.config.filter.clone());

    app.task_report_table.date_time_vague_precise = app.config.uda_task_report_date_time_vague_more_precise;

    // app.update(true)?;

    app.filter_history.load()?;
    app.filter_history.add(app.filter.value());
    app.command_history.load()?;
    app.task_background();

    if app.task_version < *TASKWARRIOR_VERSION_SUPPORTED {
      app.error = Some(format!(
        "Found taskwarrior version {} but taskwarrior-tui works with taskwarrior>={}",
        app.task_version, *TASKWARRIOR_VERSION_SUPPORTED
      ));
      app.mode = Mode::TaskError;
    }

    Ok(app)
  }

  pub async fn run(&mut self) -> Result<()> {
    let mut tui = tui::Tui::new(self.tick_rate as usize)?;
    tui.enter()?;

    let mut events: Vec<KeyEvent> = Vec::new();
    // let mut ticker = 0;

    loop {
      if self.requires_redraw {
        let s = tui.size()?;
        tui.resize(s)?;
        self.requires_redraw = false;
      }
      tui.draw(|f| self.draw(f))?;
      if let Some(event) = tui.next().await {
        trace_dbg!(event);
        let mut maybe_action = match event {
          Event::Quit => Some(Action::Quit),
          Event::Error => Some(Action::Error("Received event error".into())),
          Event::Closed => Some(Action::Quit),
          Event::Tick => {
            events.clear();
            Some(Action::Tick)
          }
          Event::Key(key_event) => {
            events.push(key_event);
            self.handle_event(&events)?
          }
          Event::Mouse(_) => None,
          Event::Resize(x, y) => None,
        };
        while let Some(action) = maybe_action {
          maybe_action = self.update(action)?;
        }
      }

      if self.should_quit {
        break;
      }
    }
    tui.exit()?;
    Ok(())
  }

  pub fn update(&mut self, action: Action) -> Result<Option<Action>> {
    if let Action::Quit = action {
      self.should_quit = true;
      return Ok(None);
    }
    Ok(None)
  }

  pub fn reset_command(&mut self) {
    self.command.reset()
  }

  pub fn get_context(&mut self) -> Result<()> {
    let output = std::process::Command::new("task")
      .arg("_get")
      .arg("rc.context")
      .output()?;
    self.current_context = String::from_utf8_lossy(&output.stdout).to_string();
    self.current_context = self.current_context.strip_suffix('\n').unwrap_or("").to_string();

    // support new format for context
    let output = std::process::Command::new("task")
      .arg("_get")
      .arg(format!("rc.context.{}.read", self.current_context))
      .output()?;
    self.current_context_filter = String::from_utf8_lossy(&output.stdout).to_string();
    self.current_context_filter = self.current_context_filter.strip_suffix('\n').unwrap_or("").to_string();

    // If new format is not used, check if old format is used
    if self.current_context_filter.is_empty() {
      let output = std::process::Command::new("task")
        .arg("_get")
        .arg(format!("rc.context.{}", self.current_context))
        .output()?;
      self.current_context_filter = String::from_utf8_lossy(&output.stdout).to_string();
      self.current_context_filter = self.current_context_filter.strip_suffix('\n').unwrap_or("").to_string();
    }
    Ok(())
  }

  pub fn draw(&mut self, f: &mut Frame<impl Backend>) {
    let rect = f.size();
    self.terminal_width = rect.width;
    self.terminal_height = rect.height;

    let chunks = Layout::default()
      .direction(Direction::Vertical)
      .constraints([Constraint::Length(1), Constraint::Min(0)])
      .split(f.size());

    let tab_layout = chunks[0];
    let main_layout = chunks[1];

    self.draw_tabs(f, tab_layout);
    match self.mode {
      Mode::Calendar => self.draw_calendar(f, main_layout),
      Mode::Projects => self.draw_projects(f, main_layout),
      _ => self.draw_task(f, main_layout),
    }
  }

  fn draw_tabs(&self, f: &mut Frame<impl Backend>, layout: Rect) {
    let titles: Vec<&str> = vec!["Tasks", "Projects", "Calendar"];
    let tab_names: Vec<_> = titles.into_iter().map(Line::from).collect();
    let selected_tab = match self.mode {
      Mode::Projects => 1,
      Mode::Calendar => 2,
      _ => 0,
    };
    let navbar_block = Block::default().style(*self.config.uda_style_navbar);
    let context = Line::from(vec![
      Span::from("["),
      Span::from(if self.current_context.is_empty() {
        "none"
      } else {
        &self.current_context
      }),
      Span::from("]"),
    ]);
    let tabs = Tabs::new(tab_names)
      .block(navbar_block.clone())
      .select(selected_tab)
      .divider(" ")
      .highlight_style(Style::default().add_modifier(Modifier::BOLD));
    let rects = Layout::default()
      .direction(Direction::Horizontal)
      .constraints([Constraint::Min(0), Constraint::Length(context.width() as u16)])
      .split(layout);

    f.render_widget(tabs, rects[0]);
    f.render_widget(Paragraph::new(Text::from(context)).block(navbar_block), rects[1]);
  }

  pub fn draw_debug(&mut self, f: &mut Frame<impl Backend>) {
    let area = centered_rect(f.size(), 50, 50);
    f.render_widget(Clear, area);
    let t = format!("{}", self.current_selection);
    let p =
      Paragraph::new(Text::from(t)).block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded));
    f.render_widget(p, area);
  }

  pub fn draw_projects(&mut self, f: &mut Frame<impl Backend>, rect: Rect) {
    let data = self.projects.data.clone();
    let p = Paragraph::new(Text::from(&data[..]));
    f.render_widget(p, rect);
  }

  fn style_for_project(&self, project: &[String]) -> Style {
    let virtual_tag_names_in_precedence = &self.config.rule_precedence_color;
    let mut style = Style::default();
    for tag_name in virtual_tag_names_in_precedence.iter().rev() {
      match tag_name.as_str() {
        "project." => {
          let s = self
            .config
            .color
            .get(&format!("color.project.{}", project[0]))
            .copied()
            .unwrap_or_default();
          style = style.patch(*s);
        }
        &_ => {}
      }
    }
    style
  }

  pub fn draw_calendar(&mut self, f: &mut Frame<impl Backend>, layout: Rect) {
    let mut c = Calendar::default()
      .today_style(*self.config.uda_style_calendar_today)
      .year(self.calendar_year)
      .date_style(self.get_dates_with_styles())
      .months_per_row(self.config.uda_calendar_months_per_row)
      .start_on_monday(self.config.weekstart);
    c.title_background_color = self.config.uda_style_calendar_title.bg.unwrap_or(Color::Reset);
    f.render_widget(c, layout);
  }

  pub fn draw_task(&mut self, f: &mut Frame<impl Backend>, layout: Rect) {
    let rects = Layout::default()
      .direction(Direction::Vertical)
      .constraints([Constraint::Min(0), Constraint::Length(2)].as_ref())
      .split(layout);

    // render task report and task details if required
    if self.task_report_show_info {
      let split_task_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(rects[0]);

      self.task_report_height = split_task_layout[0].height;
      self.draw_task_report(f, split_task_layout[0]);
      self.draw_task_details(f, split_task_layout[1]);
    } else {
      self.task_report_height = rects[0].height;
      self.draw_task_report(f, rects[0]);
    }

    // calculate selected tasks
    let selected = self.current_selection;
    let task_ids = if self.tasks.is_empty() {
      vec!["0".to_string()]
    } else {
      match self.task_table_state.mode() {
        TableMode::SingleSelection => vec![self.tasks[selected].id().unwrap_or_default().to_string()],
        TableMode::MultipleSelection => {
          let mut tids = vec![];
          for uuid in &self.marked {
            if let Some(t) = self.task_by_uuid(*uuid) {
              tids.push(t.id().unwrap_or_default().to_string());
            }
          }
          tids
        }
      }
    };

    // render task mode
    self.draw_task_mode_action(f, &rects, &task_ids);
  }

  fn draw_task_mode_action(&mut self, f: &mut Frame<impl Backend>, rects: &[Rect], task_ids: &[String]) {
    match self.mode {
      Mode::TaskError => {
        self.draw_command(
          f,
          rects[1],
          "Press any key to continue.",
          (
            Span::styled("Error", Style::default().add_modifier(Modifier::BOLD)),
            None,
          ),
          0,
          false,
          self.error.clone(),
        );
        let text = self.error.clone().unwrap_or_else(|| "Unknown error.".to_string());
        let title = vec![Span::styled("Error", Style::default().add_modifier(Modifier::BOLD))];
        let rect = centered_rect(f.size(), 90, 60);
        f.render_widget(Clear, rect);
        let p = Paragraph::new(Text::from(text))
          .block(
            Block::default()
              .borders(Borders::ALL)
              .border_type(BorderType::Rounded)
              .title(title),
          )
          .wrap(Wrap { trim: true });
        f.render_widget(p, rect);
        // draw error pop up
        let rects = Layout::default()
          .direction(Direction::Vertical)
          .constraints([Constraint::Min(0)].as_ref())
          .split(f.size());
      }
      Mode::TaskReport => {
        // reset error when entering Action::Report
        self.previous_mode = None;
        self.error = None;
        let position = self.command.visual_cursor();
        self.draw_command(
          f,
          rects[1],
          self.filter.value(),
          (Span::raw("Filter Tasks"), self.history_status.as_ref().map(Span::raw)),
          self.filter.visual_cursor(),
          false,
          self.error.clone(),
        );
      }
      Mode::TaskJump => {
        let position = self.command.visual_cursor();
        self.draw_command(
          f,
          rects[1],
          self.command.value(),
          (
            Span::styled("Jump to Task", Style::default().add_modifier(Modifier::BOLD)),
            None,
          ),
          position,
          true,
          self.error.clone(),
        );
      }
      Mode::TaskFilter => {
        let position = self.filter.visual_cursor();
        if self.show_completion_pane {
          self.draw_completion_pop_up(f, rects[1], position);
        }
        self.draw_command(
          f,
          rects[1],
          self.filter.value(),
          (
            Span::styled("Filter Tasks", Style::default().add_modifier(Modifier::BOLD)),
            self
              .history_status
              .as_ref()
              .map(|s| Span::styled(s, Style::default().add_modifier(Modifier::BOLD))),
          ),
          position,
          true,
          self.error.clone(),
        );
      }
      Mode::TaskLog => {
        if self.config.uda_auto_insert_double_quotes_on_log && self.command.value().is_empty() {
          self.command = self.command.clone().with_value(r#""""#.to_string());
        };
        let position = self.command.visual_cursor();
        if self.show_completion_pane {
          self.draw_completion_pop_up(f, rects[1], position);
        }
        self.draw_command(
          f,
          rects[1],
          self.command.value(),
          (
            Span::styled("Log Task", Style::default().add_modifier(Modifier::BOLD)),
            self
              .history_status
              .as_ref()
              .map(|s| Span::styled(s, Style::default().add_modifier(Modifier::BOLD))),
          ),
          position,
          true,
          self.error.clone(),
        );
      }
      Mode::TaskSubprocess => {
        let position = self.command.visual_cursor();
        self.draw_command(
          f,
          rects[1],
          self.command.value(),
          (
            Span::styled("Shell Command", Style::default().add_modifier(Modifier::BOLD)),
            None,
          ),
          position,
          true,
          self.error.clone(),
        );
      }
      Mode::TaskModify => {
        let position = self.modify.visual_cursor();
        if self.show_completion_pane {
          self.draw_completion_pop_up(f, rects[1], position);
        }
        let label = if task_ids.len() > 1 {
          format!("Modify Tasks {}", task_ids.join(","))
        } else {
          format!("Modify Task {}", task_ids.join(","))
        };
        self.draw_command(
          f,
          rects[1],
          self.modify.value(),
          (
            Span::styled(label, Style::default().add_modifier(Modifier::BOLD)),
            self
              .history_status
              .as_ref()
              .map(|s| Span::styled(s, Style::default().add_modifier(Modifier::BOLD))),
          ),
          position,
          true,
          self.error.clone(),
        );
      }
      Mode::TaskAnnotate => {
        if self.config.uda_auto_insert_double_quotes_on_annotate && self.command.value().is_empty() {
          self.command = self.command.clone().with_value(r#""""#.to_string());
        };
        let position = self.command.visual_cursor();
        if self.show_completion_pane {
          self.draw_completion_pop_up(f, rects[1], position);
        }
        let label = if task_ids.len() > 1 {
          format!("Annotate Tasks {}", task_ids.join(","))
        } else {
          format!("Annotate Task {}", task_ids.join(","))
        };
        self.draw_command(
          f,
          rects[1],
          self.command.value(),
          (
            Span::styled(label, Style::default().add_modifier(Modifier::BOLD)),
            self
              .history_status
              .as_ref()
              .map(|s| Span::styled(s, Style::default().add_modifier(Modifier::BOLD))),
          ),
          position,
          true,
          self.error.clone(),
        );
      }
      Mode::TaskAdd => {
        if self.config.uda_auto_insert_double_quotes_on_add && self.command.value().is_empty() {
          self.command = self.command.clone().with_value(r#""""#.to_string());
        };
        let position = self.command.visual_cursor();
        if self.show_completion_pane {
          self.draw_completion_pop_up(f, rects[1], position);
        }
        self.draw_command(
          f,
          rects[1],
          self.command.value(),
          (
            Span::styled("Add Task", Style::default().add_modifier(Modifier::BOLD)),
            self
              .history_status
              .as_ref()
              .map(|s| Span::styled(s, Style::default().add_modifier(Modifier::BOLD))),
          ),
          position,
          true,
          self.error.clone(),
        );
      }
      Mode::TaskHelpPopup => {
        self.draw_command(
          f,
          rects[1],
          self.filter.value(),
          ("Filter Tasks".into(), None),
          self.filter.visual_cursor(),
          false,
          self.error.clone(),
        );
        self.draw_help_popup(f, 80, 90);
      }
      Mode::TaskContextMenu => {
        self.draw_command(
          f,
          rects[1],
          self.filter.value(),
          ("Filter Tasks".into(), None),
          self.filter.visual_cursor(),
          false,
          self.error.clone(),
        );
        self.draw_context_menu(f, 80, 50);
      }
      Mode::TaskDonePrompt => {
        let label = if task_ids.len() > 1 {
          format!("Done Tasks {}?", task_ids.join(","))
        } else {
          format!("Done Task {}?", task_ids.join(","))
        };
        let x = match self.keyconfig.done {
          KeyCode::Char(c) => c.to_string(),
          _ => "Enter".to_string(),
        };
        let q = match self.keyconfig.quit {
          KeyCode::Char(c) => c.to_string(),
          _ => "Esc".to_string(),
        };
        self.draw_command(
          f,
          rects[1],
          &format!("Press <{}> to confirm or <{}> to abort.", x, q),
          (Span::styled(label, Style::default().add_modifier(Modifier::BOLD)), None),
          0,
          false,
          self.error.clone(),
        );
      }
      Mode::TaskDeletePrompt => {
        let label = if task_ids.len() > 1 {
          format!("Delete Tasks {}?", task_ids.join(","))
        } else {
          format!("Delete Task {}?", task_ids.join(","))
        };
        let x = match self.keyconfig.delete {
          KeyCode::Char(c) => c.to_string(),
          _ => "Enter".to_string(),
        };
        let q = match self.keyconfig.quit {
          KeyCode::Char(c) => c.to_string(),
          _ => "Esc".to_string(),
        };
        self.draw_command(
          f,
          rects[1],
          &format!("Press <{}> to confirm or <{}> to abort.", x, q),
          (Span::styled(label, Style::default().add_modifier(Modifier::BOLD)), None),
          0,
          false,
          self.error.clone(),
        );
      }
      Mode::TaskUndoPrompt => {
        let label = "Run `task undo`?";
        let k = match self.keyconfig.undo {
          KeyCode::Char(c) => c.to_string(),
          _ => "Enter".to_string(),
        };
        let q = match self.keyconfig.quit {
          KeyCode::Char(c) => c.to_string(),
          _ => "Esc".to_string(),
        };
        self.draw_command(
          f,
          rects[1],
          &format!("Press <{}> to confirm or <{}> to abort.", k, q),
          (Span::styled(label, Style::default().add_modifier(Modifier::BOLD)), None),
          0,
          false,
          self.error.clone(),
        );
      }
      _ => {}
    }
  }

  pub fn get_dates_with_styles(&self) -> Vec<(chrono::NaiveDate, Style)> {
    if !self.tasks.is_empty() {
      let tasks = &self.tasks;
      tasks
        .iter()
        .filter_map(|t| t.due().map(|d| (d.clone(), self.style_for_task(t))))
        .map(|(d, t)| {
          let now = Local::now();
          let reference = TimeZone::from_utc_datetime(now.offset(), &d);
          (reference.date_naive(), t)
        })
        .collect()
    } else {
      vec![]
    }
  }

  fn draw_help_popup(&mut self, f: &mut Frame<impl Backend>, percent_x: u16, percent_y: u16) {
    let area = centered_rect(f.size(), percent_x, percent_y);
    f.render_widget(Clear, area);

    let chunks = Layout::default()
      .constraints([Constraint::Max(area.height - 1), Constraint::Max(1)].as_ref())
      .margin(0)
      .split(area);

    self.help_popup.scroll = std::cmp::min(
      self.help_popup.scroll,
      (self.help_popup.text_height as u16).saturating_sub(chunks[0].height - 3),
    );

    let ratio = ((self.help_popup.scroll + chunks[0].height) as f64 / self.help_popup.text_height as f64).min(1.0);

    let gauge = LineGauge::default()
      .block(Block::default())
      .gauge_style(Style::default().fg(Color::Gray))
      .ratio(ratio);

    f.render_widget(gauge, chunks[1]);
    f.render_widget(&self.help_popup, chunks[0]);
  }

  fn draw_context_menu(&mut self, f: &mut Frame<impl Backend>, percent_x: u16, percent_y: u16) {
    let rects = Layout::default()
      .direction(Direction::Vertical)
      .constraints([Constraint::Min(0)].as_ref())
      .split(f.size());

    let area = centered_rect(f.size(), percent_x, percent_y);

    f.render_widget(
      Clear,
      area.inner(&Margin {
        vertical: 0,
        horizontal: 0,
      }),
    );

    let (contexts, headers) = self.get_all_contexts();

    let maximum_column_width = area.width;
    let widths = self.calculate_widths(&contexts, &headers, maximum_column_width);

    let selected = self.contexts.table_state.current_selection().unwrap_or_default();
    let header = headers.iter();
    let mut rows = vec![];
    let mut highlight_style = Style::default();
    for (i, context) in contexts.iter().enumerate() {
      let mut style = Style::default();
      if &self.contexts.rows[i].active == "yes" {
        style = *self.config.uda_style_context_active;
      }
      rows.push(Row::StyledData(context.iter(), style));
      if i == self.contexts.table_state.current_selection().unwrap_or_default() {
        highlight_style = style;
      }
    }

    let constraints: Vec<Constraint> = widths
      .iter()
      .map(|i| Constraint::Length((*i).try_into().unwrap_or(maximum_column_width)))
      .collect();

    let highlight_style = highlight_style.add_modifier(Modifier::BOLD);
    let t = Table::new(header, rows.into_iter())
      .block(
        Block::default()
          .borders(Borders::ALL)
          .border_type(BorderType::Rounded)
          .title(Line::from(vec![Span::styled(
            "Context",
            Style::default().add_modifier(Modifier::BOLD),
          )])),
      )
      .header_style(
        self
          .config
          .color
          .get("color.label")
          .copied()
          .unwrap_or_default()
          .add_modifier(Modifier::UNDERLINED),
      )
      .highlight_style(highlight_style)
      .highlight_symbol(&self.config.uda_selection_indicator)
      .widths(&constraints);

    f.render_stateful_widget(t, area, &mut self.contexts.table_state);
  }

  fn draw_completion_pop_up(&mut self, f: &mut Frame<impl Backend>, rect: Rect, cursor_position: usize) {
    if self.completion_list.candidates().is_empty() {
      self.show_completion_pane = false;
      return;
    }
    // Iterate through all elements in the `items` app and append some debug text to it.
    let items: Vec<ListItem> = self
      .completion_list
      .candidates()
      .iter()
      .map(|p| {
        let lines = vec![Line::from(vec![
          Span::styled(p.3.clone(), Style::default().add_modifier(Modifier::BOLD)),
          Span::from(p.4.clone()),
        ])];
        ListItem::new(lines)
      })
      .collect();

    // Create a List from all list items and highlight the currently selected one
    let items = List::new(items)
      .block(Block::default().borders(Borders::NONE).title(""))
      .style(*self.config.uda_style_report_completion_pane)
      .highlight_style(*self.config.uda_style_report_completion_pane_highlight)
      .highlight_symbol(&self.config.uda_selection_indicator);

    let area = f.size();

    let mut rect = rect;
    rect.height = std::cmp::min(area.height / 2, self.completion_list.len() as u16 + 2);
    rect.width = std::cmp::min(
      area.width / 2,
      self
        .completion_list
        .max_width()
        .unwrap_or(40)
        .try_into()
        .unwrap_or(area.width / 2),
    );
    rect.y = rect.y.saturating_sub(rect.height);
    if cursor_position as u16 + rect.width >= area.width {
      rect.x = area.width - rect.width;
    } else {
      rect.x = cursor_position as u16;
    }

    // We can now render the item list
    f.render_widget(Clear, rect);
    f.render_stateful_widget(items, rect, &mut self.completion_list.state);
  }

  fn draw_command(
    &self,
    f: &mut Frame<impl Backend>,
    rect: Rect,
    text: &str,
    title: (Span, Option<Span>),
    position: usize,
    cursor: bool,
    error: Option<String>,
  ) {
    // f.render_widget(Clear, rect);
    if cursor {
      f.set_cursor(
        std::cmp::min(rect.x + position as u16, rect.x + rect.width.saturating_sub(2)),
        rect.y + 1,
      );
    }
    let rects = Layout::default()
      .direction(Direction::Vertical)
      .constraints([Constraint::Length(1), Constraint::Min(0)].as_ref())
      .split(rect);

    // render command title
    let mut style = self.config.uda_style_command.0;
    if error.is_some() {
      style = style.fg(Color::Red);
    };
    let title_spans = if let Some(subtitle) = title.1 {
      Line::from(vec![title.0, Span::from(" ["), subtitle, Span::from("]")])
    } else {
      Line::from(vec![title.0])
    };
    let title = Paragraph::new(Text::from(title_spans)).style(style);
    f.render_widget(title, rects[0]);

    // render command
    let p = Paragraph::new(Text::from(text)).scroll((0, ((position + 2) as u16).saturating_sub(rects[1].width)));
    f.render_widget(p, rects[1]);
  }

  fn draw_task_details(&mut self, f: &mut Frame<impl Backend>, rect: Rect) {
    if self.tasks.is_empty() {
      let p = Paragraph::new(Text::from("Task not found")).block(Block::default().borders(Borders::TOP));
      f.render_widget(p, rect);
      return;
    }
    let selected = self.current_selection;
    let task_id = self.tasks[selected].id().unwrap_or_default();
    let task_uuid = *self.tasks[selected].uuid();

    let data = match self.task_details.get(&task_uuid) {
      Some(s) => s.clone(),
      None => "Loading task details ...".to_string(),
    };
    self.task_details_scroll = std::cmp::min(
      (data.lines().count() as u16)
        .saturating_sub(rect.height)
        .saturating_add(2),
      self.task_details_scroll,
    );
    let p = Paragraph::new(Text::from(&data[..]))
      .block(Block::default().borders(Borders::TOP))
      .scroll((self.task_details_scroll, 0));
    f.render_widget(p, rect);
  }

  fn task_details_scroll_up(&mut self) {
    self.task_details_scroll = self.task_details_scroll.saturating_sub(1);
  }

  fn task_details_scroll_down(&mut self) {
    self.task_details_scroll = self.task_details_scroll.saturating_add(1);
  }

  fn task_by_index(&self, i: usize) -> Option<Task> {
    let tasks = &self.tasks;
    if i >= tasks.len() {
      None
    } else {
      Some(tasks[i].clone())
    }
  }

  fn task_by_uuid(&self, uuid: Uuid) -> Option<Task> {
    let tasks = &self.tasks;
    let m = tasks.iter().find(|t| *t.uuid() == uuid);
    m.cloned()
  }

  fn task_by_id(&self, id: u64) -> Option<Task> {
    let tasks = &self.tasks;
    let m = tasks.iter().find(|t| t.id() == Some(id));
    m.cloned()
  }

  fn task_index_by_id(&self, id: u64) -> Option<usize> {
    let tasks = &self.tasks;
    let m = tasks.iter().position(|t| t.id() == Some(id));
    m
  }

  fn task_index_by_uuid(&self, uuid: Uuid) -> Option<usize> {
    let tasks = &self.tasks;
    let m = tasks.iter().position(|t| *t.uuid() == uuid);
    m
  }

  fn style_for_task(&self, task: &Task) -> Style {
    let virtual_tag_names_in_precedence = &self.config.rule_precedence_color;

    let mut style = Style::default();

    for tag_name in virtual_tag_names_in_precedence.iter().rev() {
      if tag_name == "uda." || tag_name == "priority" {
        if let Some(p) = task.priority() {
          let s = self
            .config
            .color
            .get(&format!("color.uda.priority.{}", p))
            .copied()
            .unwrap_or_default();
          style = style.patch(s.0);
        }
      } else if tag_name == "tag." {
        if let Some(tags) = task.tags() {
          for t in tags {
            let color_tag_name = format!("color.tag.{}", t);
            let s = self.config.color.get(&color_tag_name).copied().unwrap_or_default();
            style = style.patch(s.0);
          }
        }
      } else if tag_name == "project." {
        if let Some(p) = task.project() {
          let s = self
            .config
            .color
            .get(&format!("color.project.{}", p))
            .copied()
            .unwrap_or_default();
          style = style.patch(s.0);
        }
      } else if task
        .tags()
        .unwrap_or(&vec![])
        .contains(&tag_name.to_string().replace('.', "").to_uppercase())
      {
        let color_tag_name = format!("color.{}", tag_name);
        let s = self.config.color.get(&color_tag_name).copied().unwrap_or_default();
        style = style.patch(s.0);
      }
    }

    style
  }

  pub fn calculate_widths(&self, tasks: &[Vec<String>], headers: &[String], maximum_column_width: u16) -> Vec<usize> {
    // naive implementation of calculate widths
    let mut widths = headers.iter().map(String::len).collect::<Vec<usize>>();

    for row in tasks.iter() {
      for (i, cell) in row.iter().enumerate() {
        widths[i] = std::cmp::max(cell.len(), widths[i]);
      }
    }

    for (i, header) in headers.iter().enumerate() {
      if header == "Description" || header == "Definition" {
        // always give description or definition the most room to breath
        widths[i] = maximum_column_width as usize;
        break;
      }
    }
    for (i, header) in headers.iter().enumerate() {
      if i == 0 {
        // always give ID a couple of extra for indicator
        widths[i] += self.config.uda_selection_indicator.as_str().width();
        // if let TableMode::MultipleSelection = self.task_table_state.mode() {
        //     widths[i] += 2
        // };
      }
    }

    // now start trimming
    while (widths.iter().sum::<usize>() as u16) >= maximum_column_width - (headers.len()) as u16 {
      let index = widths
        .iter()
        .position(|i| i == widths.iter().max().unwrap_or(&0))
        .unwrap_or_default();
      if widths[index] == 1 {
        break;
      }
      widths[index] -= 1;
    }

    widths
  }

  fn draw_task_report(&mut self, f: &mut Frame<impl Backend>, rect: Rect) {
    let (tasks, headers) = self.get_task_report();

    if tasks.is_empty() {
      if !self.current_context.is_empty() {
        let context_style = Style::default();
        context_style.add_modifier(Modifier::ITALIC);
      }

      f.render_widget(Block::default(), rect);
      return;
    }

    let maximum_column_width = rect.width;
    let widths = self.calculate_widths(&tasks, &headers, maximum_column_width);

    for (i, header) in headers.iter().enumerate() {
      if header == "Description" || header == "Definition" {
        self.task_report_table.description_width = widths[i] - 1;
        break;
      }
    }
    let selected = self.current_selection;
    let header = headers.iter();
    let mut rows = vec![];
    let mut highlight_style = Style::default();
    let mut pos = 0;
    for (i, task) in tasks.iter().enumerate() {
      let style = self.style_for_task(&self.tasks[i]);
      if i == selected {
        pos = i;
        highlight_style = style.patch(self.config.uda_style_report_selection.0);
        if self.config.uda_selection_bold {
          highlight_style = highlight_style.add_modifier(Modifier::BOLD);
        }
        if self.config.uda_selection_italic {
          highlight_style = highlight_style.add_modifier(Modifier::ITALIC);
        }
        if self.config.uda_selection_dim {
          highlight_style = highlight_style.add_modifier(Modifier::DIM);
        }
        if self.config.uda_selection_blink {
          highlight_style = highlight_style.add_modifier(Modifier::SLOW_BLINK);
        }
        if self.config.uda_selection_reverse {
          highlight_style = highlight_style.add_modifier(Modifier::REVERSED);
        }
      }
      rows.push(Row::StyledData(task.iter(), style));
    }

    let constraints: Vec<Constraint> = widths
      .iter()
      .map(|i| Constraint::Length((*i).try_into().unwrap_or(maximum_column_width)))
      .collect();

    let t = Table::new(header, rows.into_iter())
      .header_style(
        self
          .config
          .color
          .get("color.label")
          .copied()
          .unwrap_or_default()
          .add_modifier(Modifier::UNDERLINED),
      )
      .highlight_style(highlight_style)
      .highlight_symbol(&self.config.uda_selection_indicator)
      .mark_symbol(&self.config.uda_mark_indicator)
      .unmark_symbol(&self.config.uda_unmark_indicator)
      .widths(&constraints);

    f.render_stateful_widget(t, rect, &mut self.task_table_state);
    if tasks.iter().len() as u16 > rect.height.saturating_sub(4) {
      let mut widget = Scrollbar::new(pos, tasks.iter().len());
      widget.pos_style = self.config.uda_style_report_scrollbar.0;
      widget.pos_symbol = self.config.uda_scrollbar_indicator.clone();
      widget.area_style = self.config.uda_style_report_scrollbar_area.0;
      widget.area_symbol = self.config.uda_scrollbar_area.clone();
      f.render_widget(widget, rect);
    }
  }

  fn get_all_contexts(&self) -> (Vec<Vec<String>>, Vec<String>) {
    let contexts = self
      .contexts
      .rows
      .iter()
      .filter(|c| &c.type_ == "read")
      .map(|c| vec![c.name.clone(), c.definition.clone(), c.active.clone()])
      .collect();
    let headers = vec!["Name".to_string(), "Definition".to_string(), "Active".to_string()];
    (contexts, headers)
  }

  fn get_task_report(&mut self) -> (Vec<Vec<String>>, Vec<String>) {
    self.task_report_table.generate_table(&self.tasks);
    let (tasks, headers) = self.task_report_table.simplify_table();
    (tasks, headers)
  }

  // pub fn update(&mut self, force: bool) -> Result<()> {
  //   trace!("self.update({:?});", force);
  //   if force || self.dirty || self.tasks_changed_since(self.last_export).unwrap_or(true) {
  //     self.get_context()?;
  //     let task_uuids = self.selected_task_uuids();
  //     if self.current_selection_uuid.is_none() && self.current_selection_id.is_none() && task_uuids.len() == 1 {
  //       if let Some(uuid) = task_uuids.get(0) {
  //         self.current_selection_uuid = Some(*uuid);
  //       }
  //     }
  //
  //     self.last_export = Some(std::time::SystemTime::now());
  //     self.task_report_table.export_headers(None, &self.report)?;
  //     self.export_tasks()?;
  //     if self.config.uda_task_report_use_all_tasks_for_completion {
  //       self.export_all_tasks()?;
  //     }
  //     self.contexts.update_data()?;
  //     self.projects.update_data()?;
  //     self.update_tags();
  //     self.task_details.clear();
  //     self.dirty = false;
  //     self.save_history()?;
  //   }
  //   self.cursor_fix();
  //   self.update_task_table_state();
  //   if self.task_report_show_info {
  //     self.update_task_details()?;
  //   }
  //   self.selection_fix();
  //
  //   Ok(())
  // }

  pub fn selection_fix(&mut self) {
    if let (Some(t), Some(id)) = (self.task_current(), self.current_selection_id) {
      if t.id() != Some(id) {
        if let Some(i) = self.task_index_by_id(id) {
          self.current_selection = i;
          self.current_selection_id = None;
        }
      }
    }

    if let (Some(t), Some(uuid)) = (self.task_current(), self.current_selection_uuid) {
      if t.uuid() != &uuid {
        if let Some(i) = self.task_index_by_uuid(uuid) {
          self.current_selection = i;
          self.current_selection_uuid = None;
        }
      }
    }
  }

  pub fn save_history(&mut self) -> Result<()> {
    self.filter_history.write()?;
    self.command_history.write()?;
    Ok(())
  }

  pub fn cursor_fix(&mut self) {
    while !self.tasks.is_empty() && self.current_selection >= self.tasks.len() {
      self.task_report_previous();
    }
  }

  pub fn update_task_details(&mut self) -> Result<()> {
    if self.tasks.is_empty() {
      return Ok(());
    }

    // remove task_details of tasks not in task report
    let mut to_delete = vec![];
    for k in self.task_details.keys() {
      if !self.tasks.iter().map(Task::uuid).any(|x| x == k) {
        to_delete.push(*k);
      }
    }
    for k in to_delete {
      self.task_details.remove(&k);
    }

    let selected = self.current_selection;
    if selected >= self.tasks.len() {
      return Ok(());
    }
    let current_task_uuid = *self.tasks[selected].uuid();

    let mut l = vec![selected];

    for s in 1..=self.config.uda_task_detail_prefetch {
      l.insert(0, std::cmp::min(selected.saturating_sub(s), self.tasks.len() - 1));
      l.push(std::cmp::min(selected + s, self.tasks.len() - 1));
    }

    l.dedup();

    let (tx, rx) = std::sync::mpsc::channel();
    let tasks = self.tasks.clone();
    let defaultwidth = self.terminal_width.saturating_sub(2);
    for s in &l {
      if tasks.is_empty() {
        return Ok(());
      }
      if s >= &tasks.len() {
        break;
      }
      let task_uuid = *tasks[*s].uuid();
      if !self.task_details.contains_key(&task_uuid) || task_uuid == current_task_uuid {
        debug!("Running task details for {}", task_uuid);
        let _tx = tx.clone();
        tokio::spawn(async move {
          let output = tokio::process::Command::new("task")
            .arg("rc.color=off")
            .arg("rc._forcecolor=off")
            .arg(format!("rc.defaultwidth={}", defaultwidth))
            .arg(format!("{}", task_uuid))
            .output()
            .await;
          if let Ok(output) = output {
            let data = String::from_utf8_lossy(&output.stdout).to_string();
            _tx.send(Some((task_uuid, data))).unwrap();
          }
        });
      }
    }
    drop(tx);
    while let Some((task_uuid, data)) = rx.recv()? {
      self.task_details.insert(task_uuid, data);
    }
    Ok(())
  }

  pub fn update_task_table_state(&mut self) {
    trace!("self.update_task_table_state()");
    self.task_table_state.select(Some(self.current_selection));

    for uuid in self.marked.clone() {
      if self.task_by_uuid(uuid).is_none() {
        self.marked.remove(&uuid);
      }
    }

    if self.marked.is_empty() {
      self.task_table_state.single_selection();
    }

    self.task_table_state.clear();

    for uuid in &self.marked {
      self.task_table_state.mark(self.task_index_by_uuid(*uuid));
    }
  }

  pub fn context_next(&mut self) {
    let i = match self.contexts.table_state.current_selection() {
      Some(i) => {
        if i >= self.contexts.len() - 1 {
          0
        } else {
          i + 1
        }
      }
      None => 0,
    };
    self.contexts.table_state.select(Some(i));
  }

  pub fn context_previous(&mut self) {
    let i = match self.contexts.table_state.current_selection() {
      Some(i) => {
        if i == 0 {
          self.contexts.len() - 1
        } else {
          i - 1
        }
      }
      None => 0,
    };
    self.contexts.table_state.select(Some(i));
  }

  pub fn context_select(&mut self) -> Result<()> {
    let i = self.contexts.table_state.current_selection().unwrap_or_default();
    let mut command = std::process::Command::new("task");
    command.arg("context").arg(&self.contexts.rows[i].name);
    command.output()?;
    Ok(())
  }

  pub fn task_report_top(&mut self) {
    if self.tasks.is_empty() {
      return;
    }
    self.current_selection = 0;
    self.current_selection_id = None;
    self.current_selection_uuid = None;
  }

  pub fn task_report_bottom(&mut self) {
    if self.tasks.is_empty() {
      return;
    }
    self.current_selection = self.tasks.len() - 1;
    self.current_selection_id = None;
    self.current_selection_uuid = None;
  }

  pub fn task_report_next(&mut self) {
    if self.tasks.is_empty() {
      return;
    }
    let i = {
      if self.current_selection >= self.tasks.len() - 1 {
        if self.config.uda_task_report_looping {
          0
        } else {
          self.current_selection
        }
      } else {
        self.current_selection + 1
      }
    };
    self.current_selection = i;
    self.current_selection_id = None;
    self.current_selection_uuid = None;
  }

  pub fn task_report_previous(&mut self) {
    if self.tasks.is_empty() {
      return;
    }
    let i = {
      if self.current_selection == 0 {
        if self.config.uda_task_report_looping {
          self.tasks.len() - 1
        } else {
          0
        }
      } else {
        self.current_selection - 1
      }
    };
    self.current_selection = i;
    self.current_selection_id = None;
    self.current_selection_uuid = None;
  }

  pub fn task_report_next_page(&mut self) {
    if self.tasks.is_empty() {
      return;
    }
    let i = {
      if self.current_selection == self.tasks.len() - 1 {
        if self.config.uda_task_report_looping {
          0
        } else {
          self.tasks.len() - 1
        }
      } else {
        std::cmp::min(
          self
            .current_selection
            .checked_add(self.task_report_height as usize)
            .unwrap_or(self.tasks.len() - 1),
          self.tasks.len() - 1,
        )
      }
    };
    self.current_selection = i;
    self.current_selection_id = None;
    self.current_selection_uuid = None;
  }

  pub fn task_report_previous_page(&mut self) {
    if self.tasks.is_empty() {
      return;
    }
    let i = {
      if self.current_selection == 0 {
        if self.config.uda_task_report_looping {
          self.tasks.len() - 1
        } else {
          0
        }
      } else {
        self.current_selection.saturating_sub(self.task_report_height as usize)
      }
    };
    self.current_selection = i;
    self.current_selection_id = None;
    self.current_selection_uuid = None;
  }

  pub fn task_report_jump(&mut self) -> Result<()> {
    if self.tasks.is_empty() {
      return Ok(());
    }
    let i = self.command.value().parse::<usize>()?;
    if let Some(task) = self.task_by_id(i as u64) {
      let j = self.task_index_by_uuid(*task.uuid()).unwrap_or_default();
      self.current_selection = j;
      self.current_selection_id = None;
      self.current_selection_uuid = None;
      Ok(())
    } else {
      Err(anyhow!("Cannot locate task id {} in report", i))
    }
  }

  fn get_task_files_max_mtime(&self) -> Result<SystemTime> {
    let data_dir = shellexpand::tilde(&self.config.data_location).into_owned();
    ["backlog.data", "completed.data", "pending.data"]
      .iter()
      .map(|n| fs::metadata(Path::new(&data_dir).join(n)).map(|m| m.modified()))
      .filter_map(Result::ok)
      .filter_map(Result::ok)
      .max()
      .ok_or_else(|| anyhow!("Unable to get task files max time"))
  }

  pub fn tasks_changed_since(&mut self, prev: Option<SystemTime>) -> Result<bool> {
    if let Some(prev) = prev {
      let mtime = self.get_task_files_max_mtime()?;
      if mtime > prev {
        Ok(true)
      } else {
        // Unfortunately, we can not use std::time::Instant which is guaranteed to be monotonic,
        // because we need to compare it to a file mtime as SystemTime, so as a safety for unexpected
        // time shifts, cap maximum wait to 1 min
        let now = SystemTime::now();
        let max_delta = Duration::from_secs(60);
        Ok(now.duration_since(prev)? > max_delta)
      }
    } else {
      Ok(true)
    }
  }

  pub fn export_all_tasks(&mut self) -> Result<()> {
    let mut task = std::process::Command::new("task");

    task
      .arg("rc.json.array=on")
      .arg("rc.confirmation=off")
      .arg("rc.json.depends.array=on")
      .arg("rc.color=off")
      .arg("rc._forcecolor=off");
    // .arg("rc.verbose:override=false");

    task.arg("export");

    task.arg("all");

    info!("Running `{:?}`", task);
    let output = task.output()?;
    let data = String::from_utf8_lossy(&output.stdout);
    let error = String::from_utf8_lossy(&output.stderr);

    if output.status.success() {
      if let Ok(imported) = import(data.as_bytes()) {
        self.all_tasks = imported;
        info!("Imported {} tasks", self.tasks.len());
        self.error = None;
        if self.mode == Mode::TaskError {
          self.mode = self.previous_mode.clone().unwrap_or(Mode::TaskReport);
          self.previous_mode = None;
        }
      } else {
        self.error = Some(format!("Unable to parse output of `{:?}`:\n`{:?}`", task, data));
        self.mode = Mode::TaskError;
        debug!("Unable to parse output: {:?}", data);
      }
    } else {
      self.error = Some(format!(
        "Cannot run `{:?}` - ({}) error:\n{}",
        &task, output.status, error
      ));
    }

    Ok(())
  }

  pub fn export_tasks(&mut self) -> Result<()> {
    let mut task = std::process::Command::new("task");

    task
      .arg("rc.json.array=on")
      .arg("rc.confirmation=off")
      .arg("rc.json.depends.array=on")
      .arg("rc.color=off")
      .arg("rc._forcecolor=off");
    // .arg("rc.verbose:override=false");

    if let Some(args) =
      shlex::split(format!(r#"rc.report.{}.filter='{}'"#, self.report, self.filter.value().trim()).trim())
    {
      for arg in args {
        task.arg(arg);
      }
    }

    if !self.current_context_filter.trim().is_empty() && self.task_version >= *TASKWARRIOR_VERSION_SUPPORTED {
      if let Some(args) = shlex::split(&self.current_context_filter) {
        for arg in args {
          task.arg(arg);
        }
      }
    } else if !self.current_context_filter.trim().is_empty() {
      task.arg(format!("'\\({}\\)'", self.current_context_filter));
    }

    task.arg("export");

    if self.task_version >= *TASKWARRIOR_VERSION_SUPPORTED {
      task.arg(&self.report);
    }

    info!("Running `{:?}`", task);
    let output = task.output()?;
    let data = String::from_utf8_lossy(&output.stdout);
    let error = String::from_utf8_lossy(&output.stderr);

    if output.status.success() {
      if let Ok(imported) = import(data.as_bytes()) {
        self.tasks = imported;
        info!("Imported {} tasks", self.tasks.len());
        self.error = None;
        if self.mode == Mode::TaskError {
          self.mode = self.previous_mode.clone().unwrap_or(Mode::TaskReport);
          self.previous_mode = None;
        }
      } else {
        self.error = Some(format!("Unable to parse output of `{:?}`:\n`{:?}`", task, data));
        self.mode = Mode::TaskError;
        debug!("Unable to parse output: {:?}", data);
      }
    } else {
      self.error = Some(format!(
        "Cannot run `{:?}` - ({}) error:\n{}",
        &task, output.status, error
      ));
    }

    Ok(())
  }

  pub fn selected_task_uuids(&self) -> Vec<Uuid> {
    let selected = match self.task_table_state.mode() {
      TableMode::SingleSelection => vec![self.current_selection],
      TableMode::MultipleSelection => self.task_table_state.marked().copied().collect::<Vec<usize>>(),
    };

    let mut task_uuids = vec![];

    for s in selected {
      if self.tasks.is_empty() {
        break;
      }
      let task_id = self.tasks[s].id().unwrap_or_default();
      let task_uuid = *self.tasks[s].uuid();
      task_uuids.push(task_uuid);
    }

    task_uuids
  }

  pub fn task_subprocess(&mut self) -> Result<(), String> {
    let task_uuids = if self.tasks.is_empty() {
      vec![]
    } else {
      self.selected_task_uuids()
    };

    let shell = self.command.value();

    let r = match shlex::split(shell) {
      Some(cmd) => {
        if cmd.is_empty() {
          Err(format!("Shell command empty: {}", shell))
        } else {
          // first argument must be a binary
          let mut command = std::process::Command::new(&cmd[0]);
          // remaining arguments are args
          for (i, s) in cmd.iter().enumerate() {
            if i == 0 {
              continue;
            }
            command.arg(s);
          }
          let output = command.output();
          match output {
            Ok(o) => {
              let output = String::from_utf8_lossy(&o.stdout);
              if !output.is_empty() {
                Err(format!(
                  r#"Shell command `{}` ran successfully but printed the following output:

                {}

                Suppress output of shell commands to prevent the error prompt from showing up."#,
                  shell, output
                ))
              } else {
                Ok(())
              }
            }
            Err(_) => Err(format!("Shell command `{}` exited with non-zero output", shell)),
          }
        }
      }
      None => Err(format!("Cannot run subprocess. Unable to shlex split `{}`", shell)),
    };

    if task_uuids.len() == 1 {
      if let Some(uuid) = task_uuids.get(0) {
        self.current_selection_uuid = Some(*uuid);
      }
    }

    r
  }

  pub fn task_log(&mut self) -> Result<(), String> {
    let mut command = std::process::Command::new("task");

    command.arg("log");

    let shell = self.command.value();

    match shlex::split(shell) {
      Some(cmd) => {
        for s in cmd {
          command.arg(&s);
        }
        let output = command.output();
        match output {
          Ok(_) => Ok(()),
          Err(_) => Err(format!(
            "Cannot run `task log {}`. Check documentation for more information",
            shell
          )),
        }
      }
      None => Err(format!(
        "Unable to run `{:?}`: shlex::split(`{}`) failed.",
        command, shell
      )),
    }
  }

  pub fn task_background(&mut self) {
    let shell = self.config.uda_background_process.clone();
    if shell.is_empty() {
      return;
    }
    let shell = shellexpand::tilde(&shell).into_owned();
    let period = self.config.uda_background_process_period;
    std::thread::spawn(move || loop {
      std::thread::sleep(Duration::from_secs(period as u64));
      match shlex::split(&shell) {
        Some(cmd) => {
          let mut command = std::process::Command::new(&cmd[0]);
          for s in cmd.iter().skip(1) {
            command.arg(s);
          }
          if let Ok(output) = command.output() {
            if !output.status.success() {
              break;
            }
          } else {
            break;
          }
        }
        None => break,
      };
    });
  }

  pub fn task_shortcut(&mut self, s: usize) -> Result<(), String> {
    // self.pause_tui().await.unwrap();

    let task_uuids = if self.tasks.is_empty() {
      vec![]
    } else {
      self.selected_task_uuids()
    };

    let shell = &self.config.uda_shortcuts[s];

    if shell.is_empty() {
      // self.resume_tui().await.unwrap();
      return Err("Trying to run empty shortcut.".to_string());
    }

    let shell = format!(
      "{} {}",
      shell,
      task_uuids
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<String>>()
        .join(" ")
    );

    let shell = shellexpand::tilde(&shell).into_owned();
    let r = match shlex::split(&shell) {
      Some(cmd) => {
        let mut command = std::process::Command::new(&cmd[0]);
        for i in cmd.iter().skip(1) {
          command.arg(i);
        }
        match command.spawn() {
          Ok(child) => {
            let output = child.wait_with_output();
            match output {
              Ok(o) => {
                if o.status.success() {
                  Ok(())
                } else {
                  Err(format!(
                    "Unable to run shortcut {}. Status Code: {} - stdout: {} stderr: {}",
                    s,
                    o.status.code().unwrap_or_default(),
                    String::from_utf8_lossy(&o.stdout),
                    String::from_utf8_lossy(&o.stderr),
                  ))
                }
              }
              Err(s) => Err(format!("`{}` failed to wait with output: {}", shell, s)),
            }
          }
          Err(err) => Err(format!(
            "`{}` failed: Unable to spawn shortcut number {} - Error: {}",
            shell, s, err
          )),
        }
      }
      None => Err(format!(
        "Unable to run shortcut number {}: shlex::split(`{}`) failed.",
        s, shell
      )),
    };

    if task_uuids.len() == 1 {
      if let Some(uuid) = task_uuids.get(0) {
        self.current_selection_uuid = Some(*uuid);
      }
    }

    // self.resume_tui().await.unwrap();

    r
  }

  pub fn task_modify(&mut self) -> Result<(), String> {
    if self.tasks.is_empty() {
      return Ok(());
    }

    let task_uuids = self.selected_task_uuids();

    let mut command = std::process::Command::new("task");
    command.arg("rc.bulk=0");
    command.arg("rc.confirmation=off");
    command.arg("rc.dependency.confirmation=off");
    command.arg("rc.recurrence.confirmation=off");
    for task_uuid in &task_uuids {
      command.arg(task_uuid.to_string());
    }
    command.arg("modify");

    let shell = self.modify.value();

    let r = match shlex::split(shell) {
      Some(cmd) => {
        for s in cmd {
          command.arg(&s);
        }
        let output = command.output();
        match output {
          Ok(o) => {
            if o.status.success() {
              Ok(())
            } else {
              Err(format!("Modify failed. {}", String::from_utf8_lossy(&o.stdout)))
            }
          }
          Err(_) => Err(format!(
            "Cannot run `task {:?} modify {}`. Check documentation for more information",
            task_uuids, shell,
          )),
        }
      }
      None => Err(format!("Cannot shlex split `{}`", shell)),
    };

    if task_uuids.len() == 1 {
      if let Some(uuid) = task_uuids.get(0) {
        self.current_selection_uuid = Some(*uuid);
      }
    }

    r
  }

  pub fn task_annotate(&mut self) -> Result<(), String> {
    if self.tasks.is_empty() {
      return Ok(());
    }

    let task_uuids = self.selected_task_uuids();

    let mut command = std::process::Command::new("task");
    command.arg("rc.bulk=0");
    command.arg("rc.confirmation=off");
    command.arg("rc.dependency.confirmation=off");
    command.arg("rc.recurrence.confirmation=off");
    for task_uuid in &task_uuids {
      command.arg(task_uuid.to_string());
    }
    command.arg("annotate");

    let shell = self.command.value();

    let r = match shlex::split(shell) {
      Some(cmd) => {
        for s in cmd {
          command.arg(&s);
        }
        let output = command.output();
        match output {
          Ok(o) => {
            if o.status.success() {
              Ok(())
            } else {
              Err(format!("Annotate failed. {}", String::from_utf8_lossy(&o.stdout)))
            }
          }
          Err(_) => Err(format!(
            "Cannot run `task {} annotate {}`. Check documentation for more information",
            task_uuids
              .iter()
              .map(ToString::to_string)
              .collect::<Vec<String>>()
              .join(" "),
            shell
          )),
        }
      }
      None => Err(format!("Cannot shlex split `{}`", shell)),
    };

    if task_uuids.len() == 1 {
      if let Some(uuid) = task_uuids.get(0) {
        self.current_selection_uuid = Some(*uuid);
      }
    }
    r
  }

  pub fn task_add(&mut self) -> Result<(), String> {
    let mut command = std::process::Command::new("task");
    command.arg("add");

    let shell = self.command.value();

    match shlex::split(shell) {
      Some(cmd) => {
        for s in cmd {
          command.arg(&s);
        }
        let output = command.output();
        match output {
          Ok(output) => {
            if output.status.code() == Some(0) {
              let data = String::from_utf8_lossy(&output.stdout);
              let re = Regex::new(r"^Created task (?P<task_id>\d+).\n$").unwrap();
              if self.config.uda_task_report_jump_to_task_on_add {
                if let Some(caps) = re.captures(&data) {
                  self.current_selection_id = Some(caps["task_id"].parse::<u64>().unwrap_or_default());
                }
              }
              Ok(())
            } else {
              Err(format!("Error: {}", String::from_utf8_lossy(&output.stderr)))
            }
          }
          Err(e) => Err(format!("Cannot run `{:?}`: {}", command, e)),
        }
      }
      None => Err(format!(
        "Unable to run `{:?}`: shlex::split(`{}`) failed.",
        command, shell
      )),
    }
  }

  pub fn task_virtual_tags(task_uuid: Uuid) -> Result<String, String> {
    let output = std::process::Command::new("task")
      .arg(format!("{}", task_uuid))
      .output();

    match output {
      Ok(output) => {
        let data = String::from_utf8_lossy(&output.stdout);
        for line in data.split('\n') {
          for prefix in &["Virtual tags", "Virtual"] {
            if line.starts_with(prefix) {
              let line = line.to_string();
              let line = line.replace(prefix, "");
              return Ok(line);
            }
          }
        }
        Err(format!(
          "Cannot find any tags for `task {}`. Check documentation for more information",
          task_uuid
        ))
      }
      Err(_) => Err(format!(
        "Cannot run `task {}`. Check documentation for more information",
        task_uuid
      )),
    }
  }

  pub fn task_start_stop(&mut self) -> Result<(), String> {
    if self.tasks.is_empty() {
      return Ok(());
    }

    let task_uuids = self.selected_task_uuids();

    for task_uuid in &task_uuids {
      let mut command = "start";
      for tag in TaskwarriorTui::task_virtual_tags(*task_uuid)
        .unwrap_or_default()
        .split(' ')
      {
        if tag == "ACTIVE" {
          command = "stop";
        }
      }

      let output = std::process::Command::new("task")
        .arg(task_uuid.to_string())
        .arg(command)
        .output();
      if output.is_err() {
        return Err(format!("Error running `task {}` for task `{}`.", command, task_uuid));
      }
    }

    if task_uuids.len() == 1 {
      if let Some(uuid) = task_uuids.get(0) {
        self.current_selection_uuid = Some(*uuid);
      }
    }

    Ok(())
  }

  pub fn task_quick_tag(&mut self) -> Result<(), String> {
    let tag_name = &self.config.uda_quick_tag_name;
    let ptag_name = format!("+{}", tag_name);
    let ntag_name = format!("-{}", tag_name);
    if self.tasks.is_empty() {
      return Ok(());
    }

    let task_uuids = self.selected_task_uuids();

    for task_uuid in &task_uuids {
      if let Some(task) = self.task_by_uuid(*task_uuid) {
        let mut tag_to_set = &ptag_name;
        for tag in task.tags().unwrap() {
          if tag == tag_name {
            tag_to_set = &ntag_name;
          }
        }

        let output = std::process::Command::new("task")
          .arg(task_uuid.to_string())
          .arg("modify")
          .arg(tag_to_set)
          .output();

        if output.is_err() {
          return Err(format!(
            "Error running `task modify {}` for task `{}`.",
            tag_to_set, task_uuid,
          ));
        }
      }
    }

    if task_uuids.len() == 1 {
      if let Some(uuid) = task_uuids.get(0) {
        self.current_selection_uuid = Some(*uuid);
      }
    }

    Ok(())
  }

  pub fn task_delete(&mut self) -> Result<(), String> {
    if self.tasks.is_empty() {
      return Ok(());
    }

    let task_uuids = self.selected_task_uuids();

    let mut cmd = std::process::Command::new("task");
    cmd
      .arg("rc.bulk=0")
      .arg("rc.confirmation=off")
      .arg("rc.dependency.confirmation=off")
      .arg("rc.recurrence.confirmation=off");
    for task_uuid in &task_uuids {
      cmd.arg(task_uuid.to_string());
    }
    cmd.arg("delete");
    let output = cmd.output();
    let r = match output {
      Ok(_) => Ok(()),
      Err(_) => Err(format!(
        "Cannot run `task delete` for tasks `{}`. Check documentation for more information",
        task_uuids
          .iter()
          .map(ToString::to_string)
          .collect::<Vec<String>>()
          .join(" ")
      )),
    };
    self.current_selection_uuid = None;
    self.current_selection_id = None;
    r
  }

  pub fn task_done(&mut self) -> Result<(), String> {
    if self.tasks.is_empty() {
      return Ok(());
    }
    let task_uuids = self.selected_task_uuids();
    let mut cmd = std::process::Command::new("task");
    cmd
      .arg("rc.bulk=0")
      .arg("rc.confirmation=off")
      .arg("rc.dependency.confirmation=off")
      .arg("rc.recurrence.confirmation=off");
    for task_uuid in &task_uuids {
      cmd.arg(task_uuid.to_string());
    }
    cmd.arg("done");
    let output = cmd.output();
    let r = match output {
      Ok(_) => Ok(()),
      Err(_) => Err(format!(
        "Cannot run `task done` for task `{}`. Check documentation for more information",
        task_uuids
          .iter()
          .map(ToString::to_string)
          .collect::<Vec<String>>()
          .join(" ")
      )),
    };
    self.current_selection_uuid = None;
    self.current_selection_id = None;
    r
  }

  pub fn task_undo(&mut self) -> Result<(), String> {
    let output = std::process::Command::new("task")
      .arg("rc.confirmation=off")
      .arg("undo")
      .output();

    match output {
      Ok(output) => {
        let data = String::from_utf8_lossy(&output.stdout);
        let re =
          Regex::new(r"(?P<task_uuid>[a-fA-F0-9]{8}-[a-fA-F0-9]{4}-[a-fA-F0-9]{4}-[a-fA-F0-9]{4}-[a-fA-F0-9]{12})")
            .unwrap();
        if let Some(caps) = re.captures(&data) {
          if let Ok(uuid) = Uuid::parse_str(&caps["task_uuid"]) {
            self.current_selection_uuid = Some(uuid);
          }
        }
        Ok(())
      }
      Err(_) => Err("Cannot run `task undo`. Check documentation for more information".to_string()),
    }
  }

  pub fn task_edit(&mut self) -> Result<(), String> {
    if self.tasks.is_empty() {
      return Ok(());
    }

    // self.pause_tui().await.unwrap();

    let selected = self.current_selection;
    let task_id = self.tasks[selected].id().unwrap_or_default();
    let task_uuid = *self.tasks[selected].uuid();

    let r = std::process::Command::new("task")
      .arg(format!("{}", task_uuid))
      .arg("edit")
      .spawn();

    let r = match r {
      Ok(child) => {
        let output = child.wait_with_output();
        match output {
          Ok(output) => {
            if output.status.success() {
              Ok(())
            } else {
              Err(format!(
                "`task edit` for task `{}` failed. {}{}",
                task_uuid,
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr),
              ))
            }
          }
          Err(err) => Err(format!("Cannot run `task edit` for task `{}`. {}", task_uuid, err)),
        }
      }
      _ => Err(format!(
        "Cannot start `task edit` for task `{}`. Check documentation for more information",
        task_uuid
      )),
    };

    self.current_selection_uuid = Some(task_uuid);

    // self.resume_tui().await.unwrap();

    r
  }

  pub fn task_current(&self) -> Option<Task> {
    if self.tasks.is_empty() {
      return None;
    }
    let selected = self.current_selection;
    Some(self.tasks[selected].clone())
  }

  pub fn update_tags(&mut self) {
    let tasks = &mut self.tasks;

    // dependency scan
    for l_i in 0..tasks.len() {
      let default_deps = vec![];
      let deps = tasks[l_i].depends().unwrap_or(&default_deps).clone();
      tasks[l_i].add_tag("UNBLOCKED".to_string());
      for dep in deps {
        for r_i in 0..tasks.len() {
          if tasks[r_i].uuid() == &dep {
            let l_status = tasks[l_i].status();
            let r_status = tasks[r_i].status();
            if l_status != &TaskStatus::Completed
              && l_status != &TaskStatus::Deleted
              && r_status != &TaskStatus::Completed
              && r_status != &TaskStatus::Deleted
            {
              tasks[l_i].remove_tag("UNBLOCKED");
              tasks[l_i].add_tag("BLOCKED".to_string());
              tasks[r_i].add_tag("BLOCKING".to_string());
            }
            break;
          }
        }
      }
    }

    // other virtual tags
    // TODO: support all virtual tags that taskwarrior supports
    for task in tasks.iter_mut() {
      match task.status() {
        TaskStatus::Waiting => task.add_tag("WAITING".to_string()),
        TaskStatus::Completed => task.add_tag("COMPLETED".to_string()),
        TaskStatus::Pending => task.add_tag("PENDING".to_string()),
        TaskStatus::Deleted => task.add_tag("DELETED".to_string()),
        TaskStatus::Recurring => (),
      }
      if task.start().is_some() {
        task.add_tag("ACTIVE".to_string());
      }
      if task.scheduled().is_some() {
        task.add_tag("SCHEDULED".to_string());
      }
      if task.parent().is_some() {
        task.add_tag("INSTANCE".to_string());
      }
      if task.until().is_some() {
        task.add_tag("UNTIL".to_string());
      }
      if task.annotations().is_some() {
        task.add_tag("ANNOTATED".to_string());
      }
      let virtual_tags = self.task_report_table.virtual_tags.clone();
      if task.tags().is_some() && task.tags().unwrap().iter().any(|s| !virtual_tags.contains(s)) {
        task.add_tag("TAGGED".to_string());
      }
      if !task.uda().is_empty() {
        task.add_tag("UDA".to_string());
      }
      if task.mask().is_some() {
        task.add_tag("TEMPLATE".to_string());
      }
      if task.project().is_some() {
        task.add_tag("PROJECT".to_string());
      }
      if task.priority().is_some() {
        task.add_tag("PRIORITY".to_string());
      }
      if task.recur().is_some() {
        task.add_tag("RECURRING".to_string());
        let r = task.recur().unwrap();
      }
      if let Some(d) = task.due() {
        let status = task.status();
        // due today
        if status != &TaskStatus::Completed && status != &TaskStatus::Deleted {
          let now = Local::now();
          let reference = TimeZone::from_utc_datetime(now.offset(), d);
          let now = TimeZone::from_utc_datetime(now.offset(), &now.naive_utc());
          let d = d.clone();
          if (reference - chrono::Duration::nanoseconds(1)).month() == now.month() {
            task.add_tag("MONTH".to_string());
          }
          if (reference - chrono::Duration::nanoseconds(1)).month() % 4 == now.month() % 4 {
            task.add_tag("QUARTER".to_string());
          }
          if reference.year() == now.year() {
            task.add_tag("YEAR".to_string());
          }
          match get_date_state(&d, self.config.due) {
            DateState::EarlierToday | DateState::LaterToday => {
              task.add_tag("DUE".to_string());
              task.add_tag("TODAY".to_string());
              task.add_tag("DUETODAY".to_string());
            }
            DateState::AfterToday => {
              task.add_tag("DUE".to_string());
              if reference.date_naive() == (now + chrono::Duration::days(1)).date_naive() {
                task.add_tag("TOMORROW".to_string());
              }
            }
            _ => (),
          }
        }
      }
      if let Some(d) = task.due() {
        let status = task.status();
        // overdue
        if status != &TaskStatus::Completed && status != &TaskStatus::Deleted && status != &TaskStatus::Recurring {
          let now = Local::now().naive_utc();
          let d = NaiveDateTime::new(d.date(), d.time());
          if d < now {
            task.add_tag("OVERDUE".to_string());
          }
        }
      }
    }
  }

  pub fn toggle_mark(&mut self) {
    if !self.tasks.is_empty() {
      let selected = self.current_selection;
      let task_id = self.tasks[selected].id().unwrap_or_default();
      let task_uuid = *self.tasks[selected].uuid();

      if !self.marked.insert(task_uuid) {
        self.marked.remove(&task_uuid);
      }
    }
  }

  pub fn toggle_mark_all(&mut self) {
    for task in &self.tasks {
      if !self.marked.insert(*task.uuid()) {
        self.marked.remove(task.uuid());
      }
    }
  }

  pub fn escape(s: &str) -> String {
    let mut es = String::with_capacity(s.len() + 2);
    es.push('"');
    for ch in s.chars() {
      match ch {
        '"' => {
          es.push('\\');
          es.push(ch);
        }
        _ => es.push(ch),
      }
    }
    es.push('"');
    es
  }

  pub fn handle_event(&mut self, input: &Vec<KeyEvent>) -> Result<Option<Action>> {
    match self.mode {
      Mode::Projects => {
        ProjectsState::handle_input(self, *input.first().unwrap())?;
        // self.update(false)?;
      }
      Mode::Calendar => {
        // if input == self.keyconfig.quit {
        //   self.should_quit = true;
        // } else if input == self.keyconfig.next_tab {
        //   if self.config.uda_change_focus_rotate {
        //     self.mode = Mode::Tasks(Action::Report);
        //   }
        // } else if input == self.keyconfig.previous_tab {
        //   self.mode = Mode::Projects;
        // } else if input == KeyCode::Up || input == self.keyconfig.up {
        //   if self.calendar_year > 0 {
        //     self.calendar_year -= 1;
        //   }
        // } else if input == KeyCode::Down || input == self.keyconfig.down {
        //   self.calendar_year += 1;
        // } else if input == KeyCode::PageUp || input == self.keyconfig.page_up {
        //   self.task_report_previous_page();
        // } else if input == KeyCode::PageDown || input == self.keyconfig.page_down {
        //   self.calendar_year += 10;
        // // } else if input == KeyCode::Ctrl('e') {
        // //   self.task_details_scroll_down();
        // // } else if input == KeyCode::Ctrl('y') {
        // //   self.task_details_scroll_up();
        // } else if input == self.keyconfig.done {
        //   if self.config.uda_task_report_prompt_on_done {
        //     self.mode = Mode::Tasks(Action::DonePrompt);
        //     if self.task_current().is_none() {
        //       self.mode = Mode::Tasks(Action::Report);
        //     }
        //   } else {
        //     match self.task_done() {
        //       Ok(_) => self.update(true).await?,
        //       Err(e) => {
        //         self.error = Some(e);
        //         self.mode = Mode::Tasks(Action::Error);
        //       }
        //     }
        //     if self.calendar_year > 0 {
        //       self.calendar_year -= 10;
        //     }
        //   }
        // }
      }
      _ => {
        return self.handle_input_by_task_mode(input);
      }
    }
    self.update_task_table_state();
    Ok(None)
  }

  fn handle_input_by_task_mode(&mut self, input: &Vec<KeyEvent>) -> Result<Option<Action>> {
    match self.mode {
      Mode::TaskReport => {
        if let Some(keymap) = self.config.keymap.get("task-report") {
          log::info!("Received input: {:?}", &input);
          log::info!("Action {:?}", keymap.get(input));
          if let Some(action) = keymap.get(input) {
            log::info!("Got action: {:?}", &action);
            return Ok(Some(action.clone()));
          }
        }
      }
      _ => {}
    }
    Ok(None)
  }

  pub fn update_completion_list(&mut self) {
    self.completion_list.clear();

    let tasks = if self.config.uda_task_report_use_all_tasks_for_completion {
      &self.all_tasks
    } else {
      &self.tasks
    };

    if let Mode::TaskModify | Mode::TaskFilter | Mode::TaskAnnotate | Mode::TaskAdd | Mode::TaskLog = self.mode {
      for s in [
        "project:".to_string(),
        "priority:".to_string(),
        "due:".to_string(),
        "scheduled:".to_string(),
        "wait:".to_string(),
        "depends:".to_string(),
      ] {
        self.completion_list.insert(("attribute".to_string(), s));
      }
    }

    if let Mode::TaskModify | Mode::TaskFilter | Mode::TaskAnnotate | Mode::TaskAdd | Mode::TaskLog = self.mode {
      for s in [
        ".before:",
        ".under:",
        ".below:",
        ".after:",
        ".over:",
        ".above:",
        ".by:",
        ".none:",
        ".any:",
        ".is:",
        ".equals:",
        ".isnt:",
        ".not:",
        ".has:",
        ".contains:",
        ".hasnt:",
        ".startswith:",
        ".left:",
        ".endswith:",
        ".right:",
        ".word:",
        ".noword:",
      ] {
        self.completion_list.insert(("modifier".to_string(), s.to_string()));
      }
    }

    if let Mode::TaskModify | Mode::TaskFilter | Mode::TaskAnnotate | Mode::TaskAdd | Mode::TaskLog = self.mode {
      for priority in &self.config.uda_priority_values {
        let p = priority.to_string();
        self.completion_list.insert(("priority".to_string(), p));
      }
      let virtual_tags = self.task_report_table.virtual_tags.clone();
      for task in tasks {
        if let Some(tags) = task.tags() {
          for tag in tags {
            if !virtual_tags.contains(tag) {
              self
                .completion_list
                .insert(("tag".to_string(), format!("tag:{}", &tag)));
            }
          }
        }
      }
      for task in tasks {
        if let Some(tags) = task.tags() {
          for tag in tags {
            if !virtual_tags.contains(tag) {
              self.completion_list.insert(("+".to_string(), format!("+{}", &tag)));
            }
          }
        }
      }
      for task in tasks {
        if let Some(project) = task.project() {
          let p = if project.contains(' ') {
            format!(r#""{}""#, &project)
          } else {
            project.to_string()
          };
          self.completion_list.insert(("project".to_string(), p));
        }
      }
      for task in tasks {
        if let Some(date) = task.due() {
          self
            .completion_list
            .insert(("due".to_string(), get_formatted_datetime(date)));
        }
      }
      for task in tasks {
        if let Some(date) = task.wait() {
          self
            .completion_list
            .insert(("wait".to_string(), get_formatted_datetime(date)));
        }
      }
      for task in tasks {
        if let Some(date) = task.scheduled() {
          self
            .completion_list
            .insert(("scheduled".to_string(), get_formatted_datetime(date)));
        }
      }
      for task in tasks {
        if let Some(date) = task.end() {
          self
            .completion_list
            .insert(("end".to_string(), get_formatted_datetime(date)));
        }
      }
    }

    if self.mode == Mode::TaskFilter {
      self.completion_list.insert(("status".to_string(), "pending".into()));
      self.completion_list.insert(("status".to_string(), "completed".into()));
      self.completion_list.insert(("status".to_string(), "deleted".into()));
      self.completion_list.insert(("status".to_string(), "recurring".into()));
    }
  }

  pub fn update_input_for_completion(&mut self) {
    match self.mode {
      Mode::TaskAdd | Mode::TaskAnnotate | Mode::TaskLog => {
        let i = get_start_word_under_cursor(self.command.value(), self.command.cursor());
        let input = self.command.value()[i..self.command.cursor()].to_string();
        self.completion_list.input(input, "".to_string());
      }
      Mode::TaskModify => {
        let i = get_start_word_under_cursor(self.modify.value(), self.modify.cursor());
        let input = self.modify.value()[i..self.modify.cursor()].to_string();
        self.completion_list.input(input, "".to_string());
      }
      Mode::TaskFilter => {
        let i = get_start_word_under_cursor(self.filter.value(), self.filter.cursor());
        let input = self.filter.value()[i..self.filter.cursor()].to_string();
        self.completion_list.input(input, "".to_string());
      }
      _ => {}
    }
  }
}

#[cfg(test)]
mod tests {}
