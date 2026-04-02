use std::{
  borrow::Borrow,
  cmp::Ordering,
  collections::{HashMap, HashSet},
  convert::TryInto,
  fs, io,
  io::{Read, Write},
  path::Path,
  sync::{Arc, Mutex, mpsc},
  time::{Duration, Instant, SystemTime},
};

use anyhow::{Context as AnyhowContext, Result, anyhow};
use chrono::{DateTime, Datelike, FixedOffset, Local, NaiveDate, NaiveDateTime, TimeZone, Timelike};
use crossterm::{
  event::{DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture},
  execute,
  style::style,
  terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use futures::SinkExt;
use log::{Level, LevelFilter, debug, error, info, log_enabled, trace, warn};
use ratatui::{
  Frame, Terminal,
  backend::{Backend, CrosstermBackend},
  layout::{Alignment, Constraint, Direction, Layout, Margin, Position, Rect},
  style::{Color, Modifier, Style},
  symbols::bar::FULL,
  text::{Line, Span, Text},
  widgets::{Block, BorderType, Borders, Clear, Gauge, LineGauge, List, ListItem, Paragraph, Tabs, Wrap},
};
use regex::Regex;
use rustyline::{At, Editor, Word, history::SearchDirection as HistoryDirection, line_buffer::LineBuffer};
use std::sync::LazyLock;
use task_hookrs::{date::Date, import::import, project::Project, status::TaskStatus, task::Task, uda::UDAValue};
use unicode_segmentation::{Graphemes, UnicodeSegmentation};
use unicode_width::UnicodeWidthStr;
use uuid::Uuid;
use versions::Versioning;

use crate::{
  action::Action,
  calendar::Calendar,
  completion::{CompletionList, get_start_word_under_cursor},
  config,
  config::Config,
  datetime,
  event::{Event, KeyCode},
  help::Help,
  history::HistoryContext,
  keyconfig::KeyConfig,
  pane::{
    Pane,
    context::{ContextDetails, ContextsState},
    project::ProjectsState,
    report::ReportsState,
  },
  scrollbar::Scrollbar,
  table::{Row, Table, TableMode, TaskwarriorTuiTableState},
  task_report::TaskReportTable,
  ui, utils,
};

const MAX_LINE: usize = 4096;

static START_TIME: LazyLock<Instant> = LazyLock::new(Instant::now);
static TASKWARRIOR_VERSION_SUPPORTED: LazyLock<Versioning> = LazyLock::new(|| Versioning::new("3.0.0").unwrap());

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
  let reference = datetime::local_from_utc(reference);

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

  if reference <= now + chrono::Duration::days(due as i64) {
    DateState::AfterToday
  } else {
    DateState::NotDue
  }
}

fn get_formatted_datetime(date: &Date) -> String {
  datetime::format_taskwarrior_datetime_literal(date)
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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
  Tasks(Action),
  Projects,
  Calendar,
}

pub struct TaskwarriorTui {
  pub should_quit: bool,
  pub dirty: bool,
  pub task_table_state: TaskwarriorTuiTableState,
  pub current_context_filter: String,
  pub current_context: String,
  pub command: LineBuffer,
  pub filter: LineBuffer,
  pub modify: LineBuffer,
  pub tasks: Vec<Task>,
  pub all_tasks: Vec<Task>,
  pub task_details: HashMap<Uuid, String>,
  pub task_details_modified: HashMap<Uuid, Option<Date>>,
  pub marked: HashSet<Uuid>,
  // stores index of current task that is highlighted
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
  pub task_details_defaultwidth: u16,
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
  pub reports: ReportsState,
  pub task_version: Versioning,
  pub error: Option<String>,
  pub event_loop: crate::event::EventLoop,
  pub requires_redraw: bool,
  pub changes: utils::Changeset,
  pub tasklist_vertical: bool,
  pub task_exe: String,
}

impl TaskwarriorTui {
  pub async fn new(report: &str, init_event_loop: bool) -> Result<Self> {
    let task_exe = std::env::var("TASKWARRIOR_TUI_TASKWARRIOR_CLI").unwrap_or_else(|_| "task".to_string());

    let output = std::process::Command::new(&task_exe)
      .arg("rc.color=off")
      .arg("rc._forcecolor=off")
      .arg("rc.defaultwidth=0")
      .arg("show")
      .output()
      .context("Unable to run `task show`.")?;

    if !output.status.success() {
      let output = std::process::Command::new(&task_exe)
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
    let c = Config::new(&data, report)?;
    let kc = KeyConfig::new(&data)?;

    let output = std::process::Command::new(&task_exe)
      .arg("--version")
      .output()
      .context("Unable to run `task --version`")?;

    let task_version = Versioning::new(String::from_utf8_lossy(&output.stdout).trim()).context("Unable to get version string")?;

    let (w, h) = crossterm::terminal::size().unwrap_or((50, 15));

    let tick_rate = if c.uda_tick_rate > 0 {
      Some(std::time::Duration::from_millis(c.uda_tick_rate))
    } else {
      None
    };
    let event_loop = crate::event::EventLoop::new(tick_rate, init_event_loop);

    let mut app = Self {
      should_quit: false,
      dirty: true,
      task_table_state: TaskwarriorTuiTableState::default(),
      tasks: vec![],
      all_tasks: vec![],
      task_details: HashMap::new(),
      task_details_modified: HashMap::new(),
      marked: HashSet::new(),
      current_selection: 0,
      current_selection_uuid: None,
      current_selection_id: None,
      current_context_filter: "".to_string(),
      current_context: "".to_string(),
      command: LineBuffer::with_capacity(MAX_LINE),
      filter: LineBuffer::with_capacity(MAX_LINE),
      modify: LineBuffer::with_capacity(MAX_LINE),
      mode: Mode::Tasks(Action::Report),
      previous_mode: None,
      task_report_height: 0,
      task_details_scroll: 0,
      task_details_defaultwidth: 0,
      task_report_show_info: c.uda_task_report_show_info,
      tasklist_vertical: c.uda_tasklist_vertical,
      config: c,
      task_report_table: TaskReportTable::new(&data, report, &task_exe)?,
      calendar_year: Local::now().year(),
      help_popup: Help::new(&kc),
      last_export: None,
      keyconfig: kc,
      terminal_width: w,
      terminal_height: h,
      filter_history: HistoryContext::new("filter.history"),
      command_history: HistoryContext::new("command.history"),
      history_status: None,
      completion_list: CompletionList::with_items(vec![]),
      show_completion_pane: false,
      report: report.to_string(),
      projects: ProjectsState::new(),
      contexts: ContextsState::new(),
      reports: ReportsState::new(),
      task_version,
      error: None,
      event_loop,
      requires_redraw: false,
      changes: utils::Changeset::default(),
      task_exe,
    };

    for c in app.config.filter.chars() {
      app.filter.insert(c, 1, &mut app.changes);
    }

    app.task_report_table.date_time_vague_precise = app.config.uda_task_report_date_time_vague_more_precise;

    app.update(true).await?;

    app.filter_history.load()?;
    app.filter_history.add(app.filter.as_str());
    app.command_history.load()?;
    app.task_background();

    if app.task_version < *TASKWARRIOR_VERSION_SUPPORTED {
      app.error = Some(format!(
        "Found taskwarrior version {} but taskwarrior-tui works with taskwarrior>={}",
        app.task_version, *TASKWARRIOR_VERSION_SUPPORTED
      ));
      app.mode = Mode::Tasks(Action::Error);
    }

    Ok(app)
  }

  pub fn start_tui(&mut self) -> Result<Terminal<CrosstermBackend<std::io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture, EnableBracketedPaste)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;
    Ok(terminal)
  }

  pub async fn resume_tui(&mut self) -> Result<()> {
    self.resume_event_loop().await?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture, EnableBracketedPaste)?;
    enable_raw_mode()?;
    self.requires_redraw = true;
    terminal.hide_cursor()?;
    Ok(())
  }

  pub async fn abort_event_loop(&mut self) -> Result<()> {
    self.event_loop.abort.send(())?;
    while let Some(event) = self.next().await {
      if let Event::Closed = event {
        break;
      }
    }
    Ok(())
  }

  pub async fn resume_event_loop(&mut self) -> Result<()> {
    let tick_rate = if self.config.uda_tick_rate > 0 {
      Some(std::time::Duration::from_millis(self.config.uda_tick_rate))
    } else {
      None
    };
    self.event_loop = crate::event::EventLoop::new(tick_rate, true);
    Ok(())
  }

  pub async fn pause_tui(&mut self) -> Result<()> {
    self.abort_event_loop().await?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture, DisableBracketedPaste)?;
    terminal.show_cursor()?;
    Ok(())
  }

  pub async fn next(&mut self) -> Option<Event<KeyCode>> {
    self.event_loop.rx.recv().await
  }

  pub async fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    loop {
      if self.requires_redraw {
        terminal.clear()?;
        self.requires_redraw = false;
      }
      terminal.draw(|f| self.draw(f))?;
      // Handle input
      if let Some(event) = self.next().await {
        match event {
          Event::Input(input) => {
            debug!("Received input = {:?}", input);
            self.handle_input(input).await?;
          }
          Event::Paste(paste) => {
            debug!("Received paste of {} bytes", paste.len());
            self.handle_paste(&paste);
          }
          Event::Tick => {
            debug!("Tick event");
            self.update(false).await?;
          }
          Event::Closed => {
            debug!("Event loop closed");
          }
        }
      }

      if self.should_quit {
        break;
      }
    }
    Ok(())
  }

  pub fn reset_command(&mut self) {
    self.command.update("", 0, &mut self.changes)
  }

  fn insert_text(linebuffer: &mut LineBuffer, text: &str, changes: &mut utils::Changeset) {
    for c in text.chars() {
      linebuffer.insert(c, 1, changes);
    }
  }

  fn handle_paste(&mut self, text: &str) {
    if text.is_empty() {
      return;
    }

    match self.mode {
      Mode::Tasks(Action::Add | Action::Annotate | Action::Log) => {
        self.command_history.reset();
        Self::insert_text(&mut self.command, text, &mut self.changes);
        self.update_input_for_completion();
      }
      Mode::Tasks(Action::Modify) => {
        self.command_history.reset();
        Self::insert_text(&mut self.modify, text, &mut self.changes);
        self.update_input_for_completion();
      }
      Mode::Tasks(Action::Filter) => {
        self.filter_history.reset();
        Self::insert_text(&mut self.filter, text, &mut self.changes);
        self.update_input_for_completion();
      }
      Mode::Tasks(Action::Subprocess | Action::Jump) => {
        Self::insert_text(&mut self.command, text, &mut self.changes);
      }
      Mode::Tasks(Action::ContextMenu) => {
        self.contexts.search.push_str(text);
        self.contexts.table_state.select(Some(0));
      }
      Mode::Tasks(Action::ReportMenu) => {
        self.reports.search.push_str(text);
        self.reports.table_state.select(Some(0));
      }
      _ => {}
    }
  }

  pub fn get_context(&mut self) -> Result<()> {
    let output = std::process::Command::new(&self.task_exe).arg("_get").arg("rc.context").output()?;
    self.current_context = String::from_utf8_lossy(&output.stdout).to_string();
    self.current_context = self.current_context.strip_suffix('\n').unwrap_or("").to_string();

    // support new format for context
    let output = std::process::Command::new(&self.task_exe)
      .arg("_get")
      .arg(format!("rc.context.{}.read", self.current_context))
      .output()?;
    self.current_context_filter = String::from_utf8_lossy(&output.stdout).to_string();
    self.current_context_filter = self.current_context_filter.strip_suffix('\n').unwrap_or("").to_string();

    // If new format is not used, check if old format is used
    if self.current_context_filter.is_empty() {
      let output = std::process::Command::new(&self.task_exe)
        .arg("_get")
        .arg(format!("rc.context.{}", self.current_context))
        .output()?;
      self.current_context_filter = String::from_utf8_lossy(&output.stdout).to_string();
      self.current_context_filter = self.current_context_filter.strip_suffix('\n').unwrap_or("").to_string();
    }
    Ok(())
  }

  pub fn draw(&mut self, f: &mut Frame) {
    let rect = f.area();
    self.terminal_width = rect.width;
    self.terminal_height = rect.height;

    let chunks = Layout::default()
      .direction(Direction::Vertical)
      .constraints([Constraint::Length(1), Constraint::Min(0)])
      .split(f.area());

    let tab_layout = chunks[0];
    let main_layout = chunks[1];

    self.draw_tabs(f, tab_layout);
    match self.mode {
      Mode::Tasks(action) => self.draw_task(f, main_layout, action),
      Mode::Calendar => self.draw_calendar(f, main_layout),
      Mode::Projects => self.draw_projects(f, main_layout),
    }
  }

  fn draw_tabs(&self, f: &mut Frame, layout: Rect) {
    let titles: Vec<&str> = vec!["Tasks", "Projects", "Calendar"];
    let tab_names: Vec<_> = titles.into_iter().map(Line::from).collect();
    let selected_tab = match self.mode {
      Mode::Tasks(_) => 0,
      Mode::Projects => 1,
      Mode::Calendar => 2,
    };
    let navbar_block = Block::default().style(self.config.uda_style_navbar);
    let context = Line::from(vec![
      Span::from(&self.report),
      Span::from(" "),
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

  pub fn draw_debug(&mut self, f: &mut Frame) {
    let area = centered_rect(50, 50, f.area());
    f.render_widget(Clear, area);
    let t = format!("{}", self.current_selection);
    let p = Paragraph::new(Text::from(t)).block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded));
    f.render_widget(p, area);
  }

  pub fn draw_projects(&mut self, f: &mut Frame, rect: Rect) {
    let data = self.projects.data.clone();
    let p = Paragraph::new(Text::from(&data[..]));
    f.render_widget(p, rect);
  }

  fn style_for_project(&self, project: &[String]) -> Style {
    let virtual_tag_names_in_precedence = &self.config.rule_precedence_color;
    let mut style = Style::default();
    for tag_name in virtual_tag_names_in_precedence.iter().rev() {
      if tag_name.as_str() == "project." {
        let s = self
          .config
          .color
          .get(&format!("color.project.{}", project[0]))
          .copied()
          .unwrap_or_default();
        style = style.patch(s);
      }
    }
    style
  }

  pub fn draw_calendar(&mut self, f: &mut Frame, layout: Rect) {
    let mut c = Calendar::default()
      .today_style(self.config.uda_style_calendar_today)
      .year(self.calendar_year)
      .date_style(self.get_dates_with_styles())
      .months_per_row(self.config.uda_calendar_months_per_row)
      .start_on_monday(self.config.weekstart);
    c.title_background_color = self.config.uda_style_calendar_title.bg.unwrap_or(Color::Reset);
    f.render_widget(c, layout);
  }

  pub fn draw_task(&mut self, f: &mut Frame, layout: Rect, action: Action) {
    let rects = Layout::default()
      .direction(Direction::Vertical)
      .constraints([Constraint::Min(0), Constraint::Length(2)].as_ref())
      .split(layout);

    // render task report and task details if required
    if self.task_report_show_info {
      let direction = match self.tasklist_vertical {
        true => Direction::Vertical,
        false => Direction::Horizontal,
      };
      let split_task_layout = Layout::default()
        .direction(direction)
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
    self.handle_task_mode_action(f, &rects, &task_ids, action);
  }

  fn handle_task_mode_action(&mut self, f: &mut Frame, rects: &[Rect], task_ids: &[String], action: Action) {
    match action {
      Action::Error => {
        self.draw_command(
          f,
          rects[1],
          "Press any key to continue.",
          (Span::styled("Error", Style::default().add_modifier(Modifier::BOLD)), None),
          0,
          false,
          self.error.clone(),
          None,
        );
        let text = self.error.clone().unwrap_or_else(|| "Unknown error.".to_string());
        let title = vec![Span::styled("Error", Style::default().add_modifier(Modifier::BOLD))];
        let rect = centered_rect(90, 60, f.area());
        f.render_widget(Clear, rect);
        let p = Paragraph::new(Text::from(text))
          .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).title(title))
          .wrap(Wrap { trim: true });
        f.render_widget(p, rect);
        // draw error pop up
        let rects = Layout::default()
          .direction(Direction::Vertical)
          .constraints([Constraint::Min(0)].as_ref())
          .split(f.area());
      }
      Action::Report => {
        // reset error when entering Action::Report
        self.previous_mode = None;
        self.error = None;
        let position = Self::get_position(&self.command);
        self.draw_command(
          f,
          rects[1],
          self.filter.as_str(),
          (Span::raw("Filter Tasks"), self.history_status.as_ref().map(Span::raw)),
          Self::get_position(&self.filter),
          false,
          self.error.clone(),
          None,
        );
      }
      Action::Jump => {
        let position = Self::get_position(&self.command);
        self.draw_command(
          f,
          rects[1],
          self.command.as_str(),
          (Span::styled("Jump to Task", Style::default().add_modifier(Modifier::BOLD)), None),
          position,
          true,
          self.error.clone(),
          None,
        );
      }
      Action::Filter => {
        let position = Self::get_position(&self.filter);
        if self.show_completion_pane {
          self.draw_completion_pop_up(f, rects[1], position);
        }
        let ghost = if !self.show_completion_pane {
          self.completion_list.ghost_text()
        } else {
          None
        };
        self.draw_command(
          f,
          rects[1],
          self.filter.as_str(),
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
          ghost.as_deref(),
        );
      }
      Action::Log => {
        if self.config.uda_auto_insert_double_quotes_on_log && self.command.is_empty() {
          self.command.update(r#""""#, 1, &mut self.changes);
        };
        let position = Self::get_position(&self.command);
        if self.show_completion_pane {
          self.draw_completion_pop_up(f, rects[1], position);
        }
        let ghost = if !self.show_completion_pane {
          self.completion_list.ghost_text()
        } else {
          None
        };
        self.draw_command(
          f,
          rects[1],
          self.command.as_str(),
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
          ghost.as_deref(),
        );
      }
      Action::Subprocess => {
        let position = Self::get_position(&self.command);
        self.draw_command(
          f,
          rects[1],
          self.command.as_str(),
          (Span::styled("Shell Command", Style::default().add_modifier(Modifier::BOLD)), None),
          position,
          true,
          self.error.clone(),
          None,
        );
      }
      Action::Modify => {
        let position = Self::get_position(&self.modify);
        if self.show_completion_pane {
          self.draw_completion_pop_up(f, rects[1], position);
        }
        let ghost = if !self.show_completion_pane {
          self.completion_list.ghost_text()
        } else {
          None
        };
        let label = if task_ids.len() > 1 {
          format!("Modify Tasks {}", task_ids.join(","))
        } else {
          format!("Modify Task {}", task_ids.join(","))
        };
        self.draw_command(
          f,
          rects[1],
          self.modify.as_str(),
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
          ghost.as_deref(),
        );
      }
      Action::Annotate => {
        if self.config.uda_auto_insert_double_quotes_on_annotate && self.command.is_empty() {
          self.command.update(r#""""#, 1, &mut self.changes);
        };
        let position = Self::get_position(&self.command);
        if self.show_completion_pane {
          self.draw_completion_pop_up(f, rects[1], position);
        }
        let ghost = if !self.show_completion_pane {
          self.completion_list.ghost_text()
        } else {
          None
        };
        let label = if task_ids.len() > 1 {
          format!("Annotate Tasks {}", task_ids.join(","))
        } else {
          format!("Annotate Task {}", task_ids.join(","))
        };
        self.draw_command(
          f,
          rects[1],
          self.command.as_str(),
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
          ghost.as_deref(),
        );
      }
      Action::Add => {
        if self.config.uda_auto_insert_double_quotes_on_add && self.command.is_empty() {
          self.command.update(r#""""#, 1, &mut self.changes);
        };
        let position = Self::get_position(&self.command);
        if self.show_completion_pane {
          self.draw_completion_pop_up(f, rects[1], position);
        }
        let ghost = if !self.show_completion_pane {
          self.completion_list.ghost_text()
        } else {
          None
        };
        self.draw_command(
          f,
          rects[1],
          self.command.as_str(),
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
          ghost.as_deref(),
        );
      }
      Action::HelpPopup => {
        self.draw_command(
          f,
          rects[1],
          self.filter.as_str(),
          ("Filter Tasks".into(), None),
          Self::get_position(&self.filter),
          false,
          self.error.clone(),
          None,
        );
        self.draw_help_popup(f, 80, 90);
      }
      Action::ContextMenu => {
        self.draw_command(
          f,
          rects[1],
          self.filter.as_str(),
          ("Filter Tasks".into(), None),
          Self::get_position(&self.filter),
          false,
          self.error.clone(),
          None,
        );
        self.draw_context_menu(f, 80, 50);
      }
      Action::ReportMenu => {
        self.draw_command(
          f,
          rects[1],
          self.filter.as_str(),
          ("Filter Tasks".into(), None),
          Self::get_position(&self.filter),
          false,
          self.error.clone(),
          None,
        );
        self.draw_report_menu(f, 80, 50);
      }
      Action::DonePrompt => {
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
          None,
        );
      }
      Action::DeletePrompt => {
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
          None,
        );
      }
      Action::UndoPrompt => {
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
          None,
        );
      }
    }
  }

  pub fn get_dates_with_styles(&self) -> Vec<(chrono::NaiveDate, Style)> {
    if !self.tasks.is_empty() {
      let tasks = &self.tasks;
      tasks
        .iter()
        .filter_map(|t| t.due().map(|d| (d.clone(), self.style_for_task(t))))
        .map(|(d, t)| (datetime::local_from_utc(&d).date_naive(), t))
        .collect()
    } else {
      vec![]
    }
  }

  pub fn get_position(lb: &LineBuffer) -> usize {
    utils::display_width(&lb.as_str()[..lb.pos()])
  }

  fn draw_help_popup(&mut self, f: &mut Frame, percent_x: u16, percent_y: u16) {
    let area = centered_rect(percent_x, percent_y, f.area());
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
      .filled_style(self.config.uda_style_help_gauge)
      .ratio(ratio);

    f.render_widget(gauge, chunks[1]);
    f.render_widget(&self.help_popup, chunks[0]);
  }

  fn draw_context_menu(&mut self, f: &mut Frame, percent_x: u16, percent_y: u16) {
    let area = centered_rect(percent_x, percent_y, f.area());
    f.render_widget(Clear, area.inner(Margin { vertical: 0, horizontal: 0 }));

    // Split the popup: 1 row for search bar, remainder for the table.
    let chunks = Layout::default()
      .direction(Direction::Vertical)
      .constraints([Constraint::Length(3), Constraint::Min(0)])
      .split(area);

    // --- Search bar ---
    let search_text = format!(" {}", self.contexts.search);
    let cursor_x = chunks[0].x + 2 + self.contexts.search.len() as u16;
    let cursor_y = chunks[0].y + 1;
    f.render_widget(
      Paragraph::new(search_text).block(
        Block::default()
          .borders(Borders::ALL)
          .border_type(BorderType::Rounded)
          .title(Span::styled("Context  (type to filter)", Style::default().add_modifier(Modifier::BOLD))),
      ),
      chunks[0],
    );
    f.set_cursor_position(Position {
      x: cursor_x.min(chunks[0].x + chunks[0].width.saturating_sub(2)),
      y: cursor_y,
    });

    // --- Filtered table ---
    let indices = self.contexts.filtered_indices();
    let selected_filtered = self.contexts.table_state.current_selection().unwrap_or(0);

    let headers = ["Name", "Definition", "Active"];
    let filtered_rows: Vec<Vec<String>> = indices
      .iter()
      .map(|&ri| {
        let r = &self.contexts.rows[ri];
        vec![r.name.clone(), r.definition.clone(), r.active.clone()]
      })
      .collect();

    let maximum_column_width = chunks[1].width;
    let widths = self.calculate_widths(
      &filtered_rows,
      &headers.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
      maximum_column_width,
    );
    let constraints: Vec<Constraint> = widths
      .iter()
      .map(|w| Constraint::Length((*w).try_into().unwrap_or(maximum_column_width)))
      .collect();

    let mut highlight_style = Style::default();
    let rows: Vec<Row<std::slice::Iter<String>>> = filtered_rows
      .iter()
      .enumerate()
      .map(|(fi, row)| {
        let ri = indices[fi];
        let mut style = Style::default();
        if self.contexts.rows[ri].active == "yes" {
          style = self.config.uda_style_context_active;
        }
        if fi == selected_filtered {
          highlight_style = style;
        }
        Row::StyledData(row.iter(), style)
      })
      .collect();

    let highlight_style = highlight_style.add_modifier(Modifier::BOLD);
    let t = Table::new(headers.iter(), rows.into_iter())
      .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded))
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

    f.render_stateful_widget(t, chunks[1], &mut self.contexts.table_state);
  }

  fn draw_report_menu(&mut self, f: &mut Frame, percent_x: u16, percent_y: u16) {
    let area = centered_rect(percent_x, percent_y, f.area());
    f.render_widget(Clear, area.inner(Margin { vertical: 0, horizontal: 0 }));

    // Split the popup: 1 row for search bar, remainder for the table.
    let chunks = Layout::default()
      .direction(Direction::Vertical)
      .constraints([Constraint::Length(3), Constraint::Min(0)])
      .split(area);

    // --- Search bar ---
    let search_text = format!(" {}", self.reports.search);
    let cursor_x = chunks[0].x + 2 + self.reports.search.len() as u16;
    let cursor_y = chunks[0].y + 1;
    f.render_widget(
      Paragraph::new(search_text).block(
        Block::default()
          .borders(Borders::ALL)
          .border_type(BorderType::Rounded)
          .title(Span::styled("Report  (type to filter)", Style::default().add_modifier(Modifier::BOLD))),
      ),
      chunks[0],
    );
    f.set_cursor_position(Position {
      x: cursor_x.min(chunks[0].x + chunks[0].width.saturating_sub(2)),
      y: cursor_y,
    });

    // --- Filtered table ---
    let indices = self.reports.filtered_indices();
    let selected_filtered = self.reports.table_state.current_selection().unwrap_or(0);

    let headers = ["Name", "Description", "Active"];
    let filtered_rows: Vec<Vec<String>> = indices
      .iter()
      .map(|&ri| {
        let r = &self.reports.rows[ri];
        vec![r.name.clone(), r.description.clone(), r.active.clone()]
      })
      .collect();

    let maximum_column_width = chunks[1].width;
    let widths = self.calculate_widths(
      &filtered_rows,
      &headers.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
      maximum_column_width,
    );
    let constraints: Vec<Constraint> = widths
      .iter()
      .map(|w| Constraint::Length((*w).try_into().unwrap_or(maximum_column_width)))
      .collect();

    let mut highlight_style = Style::default();
    let rows: Vec<Row<std::slice::Iter<String>>> = filtered_rows
      .iter()
      .enumerate()
      .map(|(fi, row)| {
        let ri = indices[fi];
        let mut style = Style::default();
        if self.reports.rows[ri].active == "yes" {
          style = self.config.uda_style_report_menu_active;
        }
        if fi == selected_filtered {
          highlight_style = style;
        }
        Row::StyledData(row.iter(), style)
      })
      .collect();

    let highlight_style = highlight_style.add_modifier(Modifier::BOLD);
    let t = Table::new(headers.iter(), rows.into_iter())
      .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded))
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

    f.render_stateful_widget(t, chunks[1], &mut self.reports.table_state);
  }

  fn draw_completion_pop_up(&mut self, f: &mut Frame, rect: Rect, cursor_position: usize) {
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
      .style(self.config.uda_style_report_completion_pane)
      .highlight_style(self.config.uda_style_report_completion_pane_highlight)
      .highlight_symbol(self.config.uda_selection_indicator.as_str());

    let area = f.area();

    let mut rect = rect;
    rect.height = std::cmp::min(area.height / 2, self.completion_list.len() as u16 + 2);
    rect.width = std::cmp::min(
      area.width / 2,
      self.completion_list.max_width().unwrap_or(40).try_into().unwrap_or(area.width / 2),
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
    f: &mut Frame,
    rect: Rect,
    text: &str,
    title: (Span, Option<Span>),
    position: usize,
    cursor: bool,
    error: Option<String>,
    ghost_text: Option<&str>,
  ) {
    let rendered_text = utils::display_control_chars(text);
    let rendered_ghost_text = ghost_text.map(utils::display_control_chars);

    // f.render_widget(Clear, rect);
    if cursor {
      let position = Position::new(std::cmp::min(rect.x + position as u16, rect.x + rect.width.saturating_sub(2)), rect.y + 1);
      f.set_cursor_position(position);
    }
    let rects = Layout::default()
      .direction(Direction::Vertical)
      .constraints([Constraint::Length(1), Constraint::Min(0)].as_ref())
      .split(rect);

    // render command title
    let mut style = self.config.uda_style_command;
    if error.is_some() {
      style = self.config.uda_style_command_error;
    };
    let title_spans = if let Some(subtitle) = title.1 {
      Line::from(vec![title.0, Span::from(" ["), subtitle, Span::from("]")])
    } else {
      Line::from(vec![title.0])
    };
    let title = Paragraph::new(Text::from(title_spans)).style(style);
    f.render_widget(title, rects[0]);

    // render command with optional ghost text suffix
    let scroll = (0, ((position + 2) as u16).saturating_sub(rects[1].width));
    let p = if let Some(ghost) = rendered_ghost_text {
      let line = Line::from(vec![Span::raw(rendered_text), Span::styled(ghost, Style::default().fg(Color::DarkGray))]);
      Paragraph::new(Text::from(line)).scroll(scroll)
    } else {
      Paragraph::new(Text::from(rendered_text)).scroll(scroll)
    };
    f.render_widget(p, rects[1]);
  }

  fn draw_task_details(&mut self, f: &mut Frame, rect: Rect) {
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
      (data.lines().count() as u16).saturating_sub(rect.height).saturating_add(2),
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
    if i >= tasks.len() { None } else { Some(tasks[i].clone()) }
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
    tasks.iter().position(|t| t.id() == Some(id))
  }

  fn task_index_by_uuid(&self, uuid: Uuid) -> Option<usize> {
    let tasks = &self.tasks;
    tasks.iter().position(|t| *t.uuid() == uuid)
  }

  fn style_for_task(&self, task: &Task) -> Style {
    let virtual_tag_names_in_precedence = &self.config.rule_precedence_color;

    let mut style = Style::default();

    for tag_name in virtual_tag_names_in_precedence.iter().rev() {
      if tag_name == "uda.priority." || tag_name == "priority" {
        if let Some(p) = task.priority() {
          let s = self.config.color.get(&format!("color.uda.priority.{}", p)).copied().unwrap_or_default();
          style = style.patch(s);
        }
      } else if tag_name == "uda." {
        for (uda_name, uda_value) in task.uda().iter() {
          let uda_value_str = match uda_value {
            UDAValue::Str(s) => s.to_string(),
            UDAValue::F64(f) => f.to_string(),
            UDAValue::U64(u) => u.to_string(),
          };
          let color_uda_name = format!("color.uda.{}.{}", uda_name, uda_value_str);
          let s = self.config.color.get(&color_uda_name).copied().unwrap_or_default();
          style = style.patch(s);
        }
      } else if tag_name == "tag." {
        if let Some(tags) = task.tags() {
          for t in tags {
            let color_tag_name = format!("color.tag.{}", t);
            let s = self.config.color.get(&color_tag_name).copied().unwrap_or_default();
            style = style.patch(s);
          }
        }
      } else if tag_name == "project." {
        if let Some(p) = task.project() {
          let s = self.config.color.get(&format!("color.project.{}", p)).copied().unwrap_or_default();
          style = style.patch(s);
        }
      } else if tag_name == "keyword." {
        let desc = task.description();
        for (keyword, kw_style) in &self.config.color_keywords {
          if desc.contains(keyword.as_str()) {
            style = style.patch(*kw_style);
          }
        }
      } else if task
        .tags()
        .unwrap_or(&vec![])
        .contains(&tag_name.to_string().replace('.', "").to_uppercase())
      {
        let color_tag_name = format!("color.{}", tag_name);
        let s = self.config.color.get(&color_tag_name).copied().unwrap_or_default();
        style = style.patch(s);
      }
    }

    style
  }

  fn task_report_row_style(&self, row_index: usize, task: &Task) -> Style {
    let base_style = if row_index % 2 == 1 && self.config.uda_task_report_use_alternate_style {
      self.config.color.get("color.alternate").copied().unwrap_or_default()
    } else {
      Style::default()
    };

    base_style.patch(self.style_for_task(task))
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
      let index = widths.iter().position(|i| i == widths.iter().max().unwrap_or(&0)).unwrap_or_default();
      if widths[index] == 1 {
        break;
      }
      widths[index] -= 1;
    }

    widths
  }

  fn draw_task_report(&mut self, f: &mut Frame, rect: Rect) {
    let (tasks, headers) = self.get_task_report();

    if tasks.is_empty() {
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
      let style = self.task_report_row_style(i, &self.tasks[i]);
      if i == selected {
        pos = i;
        highlight_style = style.patch(self.config.uda_style_report_selection);
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
      .mark_highlight_symbol(&self.config.uda_mark_highlight_indicator)
      .unmark_highlight_symbol(&self.config.uda_unmark_highlight_indicator)
      .mark_symbol(&self.config.uda_mark_indicator)
      .unmark_symbol(&self.config.uda_unmark_indicator)
      .widths(&constraints);

    f.render_stateful_widget(t, rect, &mut self.task_table_state);
    if tasks.iter().len() as u16 > rect.height.saturating_sub(4) {
      let mut widget = Scrollbar::new(pos, tasks.iter().len());
      widget.pos_style = self.config.uda_style_report_scrollbar;
      widget.pos_symbol.clone_from(&self.config.uda_scrollbar_indicator);
      widget.area_style = self.config.uda_style_report_scrollbar_area;
      widget.area_symbol.clone_from(&self.config.uda_scrollbar_area);
      f.render_widget(widget, rect);
    }
  }

  fn get_task_report(&mut self) -> (Vec<Vec<String>>, Vec<String>) {
    self.task_report_table.generate_table(&self.tasks);
    let (tasks, headers) = self.task_report_table.simplify_table();
    (tasks, headers)
  }

  pub async fn update(&mut self, force: bool) -> Result<()> {
    trace!("self.update({:?});", force);
    if force || self.dirty || self.tasks_changed_since(self.last_export).unwrap_or(true) {
      self.get_context()?;
      let task_uuids = self.selected_task_uuids();
      if self.current_selection_uuid.is_none()
        && self.current_selection_id.is_none()
        && let [uuid] = task_uuids.as_slice()
      {
        self.current_selection_uuid = Some(*uuid);
      }

      self.task_report_table.export_headers(None, &self.report, &self.task_exe)?;
      self.export_tasks()?;
      if self.config.uda_task_report_use_all_tasks_for_completion {
        self.export_all_tasks()?;
      }
      self.contexts.update_data(&self.task_exe)?;
      self.reports.update_data(&self.report, &Self::task_show_output(&self.task_exe)?);
      self.projects.update_data(&self.task_exe)?;
      self.update_tags();
      self.task_details.clear();
      self.task_details_modified.clear();
      self.dirty = false;
      self.save_history()?;

      // Some operations like export or summary change the taskwarrior database.
      // The export time therefore gets set at the end, to avoid an infinite update loop.
      self.last_export = Some(std::time::SystemTime::now());
    }
    self.cursor_fix();
    self.update_task_table_state();
    if self.task_report_show_info {
      self.update_task_details().await?;
    }
    self.selection_fix();

    Ok(())
  }

  pub fn selection_fix(&mut self) {
    if let (Some(t), Some(id)) = (self.task_current(), self.current_selection_id)
      && t.id() != Some(id)
      && let Some(i) = self.task_index_by_id(id)
    {
      self.current_selection = i;
      self.current_selection_id = None;
    }

    if let (Some(t), Some(uuid)) = (self.task_current(), self.current_selection_uuid)
      && t.uuid() != &uuid
      && let Some(i) = self.task_index_by_uuid(uuid)
    {
      self.current_selection = i;
      self.current_selection_uuid = None;
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

  pub async fn update_task_details(&mut self) -> Result<()> {
    if self.tasks.is_empty() {
      return Ok(());
    }

    let defaultwidth = self.terminal_width.saturating_sub(2);
    if self.task_details_defaultwidth != defaultwidth {
      self.task_details.clear();
      self.task_details_modified.clear();
      self.task_details_defaultwidth = defaultwidth;
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
      self.task_details_modified.remove(&k);
    }

    let selected = self.current_selection;
    if selected >= self.tasks.len() {
      return Ok(());
    }

    let mut l = vec![selected];

    for s in 1..=self.config.uda_task_detail_prefetch {
      l.insert(0, std::cmp::min(selected.saturating_sub(s), self.tasks.len() - 1));
      l.push(std::cmp::min(selected + s, self.tasks.len() - 1));
    }

    l.dedup();

    let (tx, mut rx) = tokio::sync::mpsc::channel(100);
    let tasks = self.tasks.clone();
    let task_exe = self.task_exe.clone();
    for s in &l {
      if tasks.is_empty() {
        return Ok(());
      }
      if s >= &tasks.len() {
        break;
      }
      let task_uuid = *tasks[*s].uuid();
      let task_modified = tasks[*s].modified().cloned();
      let cached_modified = self.task_details_modified.get(&task_uuid).cloned();
      if !self.task_details.contains_key(&task_uuid) || cached_modified != Some(task_modified.clone()) {
        debug!("Running task details for {}", task_uuid);
        let _tx = tx.clone();
        let task_exe = task_exe.clone();
        tokio::spawn(async move {
          let output = tokio::process::Command::new(&task_exe)
            .arg("rc.color=off")
            .arg("rc._forcecolor=off")
            .arg(format!("rc.defaultwidth={}", defaultwidth))
            .arg(format!("{}", task_uuid))
            .output()
            .await;
          if let Ok(output) = output {
            let data = String::from_utf8_lossy(&output.stdout).to_string();
            _tx.send(Some((task_uuid, task_modified, data))).await.unwrap();
          }
        });
      }
    }
    drop(tx);
    while let Some(Some((task_uuid, task_modified, data))) = rx.recv().await {
      self.task_details.insert(task_uuid, data);
      self.task_details_modified.insert(task_uuid, task_modified);
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
    let n = self.contexts.filtered_indices().len();
    if n == 0 {
      return;
    }
    let i = match self.contexts.table_state.current_selection() {
      Some(i) => {
        if i >= n - 1 {
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
    let n = self.contexts.filtered_indices().len();
    if n == 0 {
      return;
    }
    let i = match self.contexts.table_state.current_selection() {
      Some(i) => {
        if i == 0 {
          n - 1
        } else {
          i - 1
        }
      }
      None => 0,
    };
    self.contexts.table_state.select(Some(i));
  }

  pub fn context_select(&mut self) -> Result<bool> {
    let fi = self.contexts.table_state.current_selection().unwrap_or_default();
    let indices = self.contexts.filtered_indices();
    let Some(ri) = indices.get(fi).copied() else {
      return Ok(false);
    };
    let context = self.contexts.rows.get(ri).map(|r| r.name.clone()).unwrap_or_default();
    if context.is_empty() {
      return Ok(false);
    }
    let mut command = std::process::Command::new(&self.task_exe);
    command.arg("context").arg(context);
    command.output()?;
    Ok(true)
  }

  fn context_selected_row_index(&self) -> Option<usize> {
    let fi = self.contexts.table_state.current_selection()?;
    self.contexts.filtered_indices().get(fi).copied()
  }

  fn restore_context_selection(&mut self, previous_row_index: Option<usize>) {
    let indices = self.contexts.filtered_indices();
    let selection = previous_row_index
      .and_then(|ri| indices.iter().position(|&filtered_ri| filtered_ri == ri))
      .or_else(|| (!indices.is_empty()).then_some(0));
    self.contexts.table_state.select(selection);
  }

  async fn maybe_autoselect_context_menu(&mut self) -> Result<()> {
    if !self.config.uda_context_menu_select_on_move {
      return Ok(());
    }

    if self.error.is_some() {
      self.previous_mode = Some(self.mode.clone());
      self.mode = Mode::Tasks(Action::Error);
      return Ok(());
    }

    match self.context_select() {
      Ok(true) => self.update(true).await?,
      Ok(false) => {}
      Err(e) => {
        self.error = Some(e.to_string());
      }
    }

    Ok(())
  }

  pub fn report_next(&mut self) {
    let n = self.reports.filtered_indices().len();
    if n == 0 {
      return;
    }
    let i = match self.reports.table_state.current_selection() {
      Some(i) => {
        if i >= n - 1 {
          0
        } else {
          i + 1
        }
      }
      None => 0,
    };
    self.reports.table_state.select(Some(i));
  }

  pub fn report_previous(&mut self) {
    let n = self.reports.filtered_indices().len();
    if n == 0 {
      return;
    }
    let i = match self.reports.table_state.current_selection() {
      Some(i) => {
        if i == 0 {
          n - 1
        } else {
          i - 1
        }
      }
      None => 0,
    };
    self.reports.table_state.select(Some(i));
  }

  pub fn report_select(&mut self, data: &str) -> Result<bool> {
    let fi = self.reports.table_state.current_selection().unwrap_or_default();
    let indices = self.reports.filtered_indices();
    let Some(ri) = indices.get(fi).copied() else {
      return Ok(false);
    };
    let report = self.reports.rows.get(ri).map(|r| r.name.clone()).unwrap_or_default();
    if report.is_empty() {
      return Ok(false);
    }

    self.report = report;
    self.config.filter = Config::get_filter(data, &self.report)?;
    if !self.config.filter.trim().is_empty() {
      self.config.filter = format!("{} ", self.config.filter.trim());
    }

    self.filter = LineBuffer::with_capacity(MAX_LINE);
    for c in self.config.filter.chars() {
      self.filter.insert(c, 1, &mut self.changes);
    }

    self.task_report_table.export_headers(Some(data), &self.report, &self.task_exe)?;
    Ok(true)
  }

  fn report_selected_row_index(&self) -> Option<usize> {
    let fi = self.reports.table_state.current_selection()?;
    self.reports.filtered_indices().get(fi).copied()
  }

  fn restore_report_selection(&mut self, previous_row_index: Option<usize>) {
    let indices = self.reports.filtered_indices();
    let selection = previous_row_index
      .and_then(|ri| indices.iter().position(|&filtered_ri| filtered_ri == ri))
      .or_else(|| (!indices.is_empty()).then_some(0));
    self.reports.table_state.select(selection);
  }

  async fn maybe_autoselect_report_menu(&mut self) -> Result<()> {
    if !self.config.uda_report_menu_select_on_move {
      return Ok(());
    }

    if self.error.is_some() {
      self.previous_mode = Some(self.mode.clone());
      self.mode = Mode::Tasks(Action::Error);
      return Ok(());
    }

    let data = Self::task_show_output(&self.task_exe)?;
    match self.report_select(&data) {
      Ok(true) => self.update(true).await?,
      Ok(false) => {}
      Err(e) => {
        self.error = Some(e.to_string());
      }
    }

    Ok(())
  }

  fn task_show_output(task_exe: &str) -> Result<String> {
    let output = std::process::Command::new(task_exe)
      .arg("rc.color=off")
      .arg("rc._forcecolor=off")
      .arg("rc.defaultwidth=0")
      .arg("show")
      .output()
      .context("Unable to run `task show`.")?;
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
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
        if self.config.uda_task_report_looping { self.tasks.len() - 1 } else { 0 }
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
        if self.config.uda_task_report_looping { 0 } else { self.tasks.len() - 1 }
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
        if self.config.uda_task_report_looping { self.tasks.len() - 1 } else { 0 }
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
    let i = self.command.as_str().parse::<usize>()?;
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

  fn get_task_database_mtime(&self) -> Result<SystemTime> {
    let data_dir = shellexpand::tilde(&self.config.data_location);
    let database_path = Path::new(data_dir.as_ref()).join("taskchampion.sqlite3");

    let metadata = fs::metadata(database_path).context("Fetching the metadate of the task database failed")?;
    let mtime = metadata
      .modified()
      .context("Could not get mtime of task database, but fetching metadata succeeded")?;

    Ok(mtime)
  }

  pub fn tasks_changed_since(&mut self, prev: Option<SystemTime>) -> Result<bool> {
    if let Some(prev) = prev {
      let mtime = self.get_task_database_mtime()?;
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
    let mut task = std::process::Command::new(&self.task_exe);

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
      let imported = import(data.as_bytes());
      match imported {
        Ok(imported) => {
          self.all_tasks = imported;
          info!("Imported {} tasks", self.tasks.len());
          self.error = None;
          if self.mode == Mode::Tasks(Action::Error) {
            self.mode = self.previous_mode.clone().unwrap_or(Mode::Tasks(Action::Report));
            self.previous_mode = None;
          }
        }
        Err(err) => {
          self.error = Some(format!("Unable to parse output of `{:?}`:\n`{:?}`\n\n{}", task, data, err));
          self.mode = Mode::Tasks(Action::Error);
          debug!("Unable to parse output: {:?}", data);
          debug!("Error: {:?}", err);
        }
      }
    } else {
      self.error = Some(format!("Cannot run `{:?}` - ({}) error:\n{}", &task, output.status, error));
    }

    Ok(())
  }

  pub fn export_tasks(&mut self) -> Result<()> {
    let mut task = std::process::Command::new(&self.task_exe);

    task
      .arg("rc.json.array=on")
      .arg("rc.confirmation=off")
      .arg("rc.json.depends.array=on")
      .arg("rc.color=off")
      .arg("rc._forcecolor=off");
    // .arg("rc.verbose:override=false");

    if let Some(args) = shlex::split(format!(r#"rc.report.{}.filter='{}'"#, self.report, self.filter.trim()).trim()) {
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

    info!("Running `{:#?}`", task);
    let output = task.output()?;
    let data = String::from_utf8_lossy(&output.stdout);
    let error = String::from_utf8_lossy(&output.stderr);

    if output.status.success() {
      let imported = import(data.as_bytes());
      match imported {
        Ok(imported) => {
          self.tasks = imported;
          info!("Imported {} tasks", self.tasks.len());
          self.error = None;
          if self.mode == Mode::Tasks(Action::Error) {
            self.mode = self.previous_mode.clone().unwrap_or(Mode::Tasks(Action::Report));
            self.previous_mode = None;
          }
        }
        Err(err) => {
          self.error = Some(format!("Unable to parse output of `{:?}`:\n`{:?}`", task, data));
          self.mode = Mode::Tasks(Action::Error);
          debug!("Unable to parse output:\n\n{}", data);
          debug!("Error: {:?}", err);
        }
      }
    } else {
      self.error = Some(format!("Cannot run `{:?}` - ({}) error:\n{}", &task, output.status, error));
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
    let task_uuids = if self.tasks.is_empty() { vec![] } else { self.selected_task_uuids() };

    let shell = self.command.as_str();

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
                  "Shell command `{}` ran successfully but printed the following output:\n\n{}\n\nSuppress output of shell commands to prevent the error prompt from showing up.",
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

    if let [uuid] = task_uuids.as_slice() {
      self.current_selection_uuid = Some(*uuid);
    }

    r
  }

  pub fn task_log(&mut self) -> Result<(), String> {
    let mut command = std::process::Command::new(&self.task_exe);

    command.arg("log");

    let shell = self.command.as_str();

    match shlex::split(shell) {
      Some(cmd) => {
        for s in cmd {
          command.arg(&s);
        }
        let output = command.output();
        match output {
          Ok(_) => Ok(()),
          Err(_) => Err(format!("Cannot run `task log {}`. Check documentation for more information", shell)),
        }
      }
      None => Err(format!("Unable to run `{:?}`: shlex::split(`{}`) failed.", command, shell)),
    }
  }

  pub fn task_background(&mut self) {
    let shell = self.config.uda_background_process.clone();
    if shell.is_empty() {
      return;
    }
    let shell = shellexpand::tilde(&shell).into_owned();
    let period = self.config.uda_background_process_period;
    std::thread::spawn(move || {
      loop {
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
      }
    });
  }

  pub async fn task_shortcut(&mut self, s: usize) -> Result<(), String> {
    self.pause_tui().await.unwrap();

    let task_uuids = if self.tasks.is_empty() { vec![] } else { self.selected_task_uuids() };

    let shell = &self.config.uda_shortcuts[s];

    if shell.is_empty() {
      self.resume_tui().await.unwrap();
      return Err("Trying to run empty shortcut.".to_string());
    }

    let shell = format!(
      "{} {}",
      shell,
      task_uuids.iter().map(ToString::to_string).collect::<Vec<String>>().join(" ")
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
          Err(err) => Err(format!("`{}` failed: Unable to spawn shortcut number {} - Error: {}", shell, s, err)),
        }
      }
      None => Err(format!("Unable to run shortcut number {}: shlex::split(`{}`) failed.", s, shell)),
    };

    if let [uuid] = task_uuids.as_slice() {
      self.current_selection_uuid = Some(*uuid);
    }

    self.resume_tui().await.unwrap();

    r
  }

  pub fn task_modify(&mut self) -> Result<(), String> {
    if self.tasks.is_empty() {
      return Ok(());
    }

    let task_uuids = self.selected_task_uuids();

    let mut command = std::process::Command::new(&self.task_exe);
    command.arg("rc.bulk=0");
    command.arg("rc.confirmation=off");
    command.arg("rc.dependency.confirmation=off");
    command.arg("rc.recurrence.confirmation=off");
    for task_uuid in &task_uuids {
      command.arg(task_uuid.to_string());
    }
    command.arg("modify");

    let shell = self.modify.as_str();

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

    if let [uuid] = task_uuids.as_slice() {
      self.current_selection_uuid = Some(*uuid);
    }

    r
  }

  pub fn task_annotate(&mut self) -> Result<(), String> {
    if self.tasks.is_empty() {
      return Ok(());
    }

    let task_uuids = self.selected_task_uuids();

    let mut command = std::process::Command::new(&self.task_exe);
    command.arg("rc.bulk=0");
    command.arg("rc.confirmation=off");
    command.arg("rc.dependency.confirmation=off");
    command.arg("rc.recurrence.confirmation=off");
    for task_uuid in &task_uuids {
      command.arg(task_uuid.to_string());
    }
    command.arg("annotate");

    let shell = self.command.as_str();

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
            task_uuids.iter().map(ToString::to_string).collect::<Vec<String>>().join(" "),
            shell
          )),
        }
      }
      None => Err(format!("Cannot shlex split `{}`", shell)),
    };

    if let [uuid] = task_uuids.as_slice() {
      self.current_selection_uuid = Some(*uuid);
    }
    r
  }

  pub fn task_add(&mut self) -> Result<(), String> {
    let mut command = std::process::Command::new(&self.task_exe);
    command.arg("add");

    let shell = self.command.as_str();

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
              if self.config.uda_task_report_jump_to_task_on_add
                && let Some(caps) = re.captures(&data)
              {
                self.current_selection_id = Some(caps["task_id"].parse::<u64>().unwrap_or_default());
              }
              Ok(())
            } else {
              Err(format!("Error: {}", String::from_utf8_lossy(&output.stderr)))
            }
          }
          Err(e) => Err(format!("Cannot run `{:?}`: {}", command, e)),
        }
      }
      None => Err(format!("Unable to run `{:?}`: shlex::split(`{}`) failed.", command, shell)),
    }
  }

  pub fn task_virtual_tags(task_uuid: Uuid, task_exe: &str) -> Result<String, String> {
    let output = std::process::Command::new(task_exe).arg(format!("{}", task_uuid)).output();

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
      Err(_) => Err(format!("Cannot run `task {}`. Check documentation for more information", task_uuid)),
    }
  }

  pub fn task_start_stop(&mut self) -> Result<(), String> {
    if self.tasks.is_empty() {
      return Ok(());
    }

    let task_uuids = self.selected_task_uuids();

    for task_uuid in &task_uuids {
      let mut command = "start";
      for tag in TaskwarriorTui::task_virtual_tags(*task_uuid, &self.task_exe)
        .unwrap_or_default()
        .split(' ')
      {
        if tag == "ACTIVE" {
          command = "stop";
        }
      }

      let output = std::process::Command::new(&self.task_exe)
        .arg(task_uuid.to_string())
        .arg(command)
        .output();
      if output.is_err() {
        return Err(format!("Error running `task {}` for task `{}`.", command, task_uuid));
      }
    }

    if let [uuid] = task_uuids.as_slice() {
      self.current_selection_uuid = Some(*uuid);
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

        let output = std::process::Command::new(&self.task_exe)
          .arg(task_uuid.to_string())
          .arg("modify")
          .arg(tag_to_set)
          .output();

        if output.is_err() {
          return Err(format!("Error running `task modify {}` for task `{}`.", tag_to_set, task_uuid,));
        }
      }
    }

    if let [uuid] = task_uuids.as_slice() {
      self.current_selection_uuid = Some(*uuid);
    }

    Ok(())
  }

  pub fn task_delete(&mut self) -> Result<(), String> {
    if self.tasks.is_empty() {
      return Ok(());
    }

    let task_uuids = self.selected_task_uuids();

    let mut cmd = std::process::Command::new(&self.task_exe);
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
        task_uuids.iter().map(ToString::to_string).collect::<Vec<String>>().join(" ")
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
    let mut cmd = std::process::Command::new(&self.task_exe);
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
        task_uuids.iter().map(ToString::to_string).collect::<Vec<String>>().join(" ")
      )),
    };
    self.current_selection_uuid = None;
    self.current_selection_id = None;
    r
  }

  pub fn task_priority(&mut self, priority: &str) -> Result<(), String> {
    if self.tasks.is_empty() {
      return Ok(());
    }
    let mut priority_arg = String::from("priority:");
    priority_arg.push_str(priority);
    let task_uuids = self.selected_task_uuids();
    let mut cmd = std::process::Command::new(&self.task_exe);
    cmd
      .arg("rc.bulk=0")
      .arg("rc.confirmation=off")
      .arg("rc.dependency.confirmation=off")
      .arg("rc.recurrence.confirmation=off")
      .arg("modify")
      .arg(&priority_arg);
    for task_uuid in &task_uuids {
      cmd.arg(task_uuid.to_string());
    }
    let output = cmd.output();
    let r = match output {
      Ok(_) => Ok(()),
      Err(_) => Err(format!(
        "Cannot run `task modify priority` for task `{}`. Check documentation for more information",
        task_uuids.iter().map(ToString::to_string).collect::<Vec<String>>().join(" ")
      )),
    };
    self.current_selection_uuid = None;
    self.current_selection_id = None;
    r
  }

  pub fn task_undo(&mut self) -> Result<(), String> {
    let output = std::process::Command::new(&self.task_exe).arg("rc.confirmation=off").arg("undo").output();

    match output {
      Ok(output) => {
        let data = String::from_utf8_lossy(&output.stdout);
        let re = Regex::new(r"(?P<task_uuid>[a-fA-F0-9]{8}-[a-fA-F0-9]{4}-[a-fA-F0-9]{4}-[a-fA-F0-9]{4}-[a-fA-F0-9]{12})").unwrap();
        if let Some(caps) = re.captures(&data)
          && let Ok(uuid) = Uuid::parse_str(&caps["task_uuid"])
        {
          self.current_selection_uuid = Some(uuid);
        }
        Ok(())
      }
      Err(_) => Err("Cannot run `task undo`. Check documentation for more information".to_string()),
    }
  }

  pub fn task_duplicate(&mut self) -> Result<(), String> {
    if self.tasks.is_empty() {
      return Ok(());
    }

    let task_uuids = self.selected_task_uuids();

    let mut command = std::process::Command::new(&self.task_exe);
    command.arg("rc.bulk=0");
    command.arg("rc.confirmation=off");
    command.arg("rc.dependency.confirmation=off");
    command.arg("rc.recurrence.confirmation=off");
    for task_uuid in &task_uuids {
      command.arg(task_uuid.to_string());
    }
    command.arg("duplicate");

    let output = command.output();
    let r = match output {
      Ok(o) => {
        if o.status.success() {
          Ok(())
        } else {
          Err(format!("Modify failed. {}", String::from_utf8_lossy(&o.stdout)))
        }
      }
      Err(_) => Err(format!(
        "Cannot run `task {:?} duplicate`. Check documentation for more information",
        task_uuids,
      )),
    };

    if let [uuid] = task_uuids.as_slice() {
      self.current_selection_uuid = Some(*uuid);
    }

    r
  }

  pub async fn task_edit(&mut self) -> Result<(), String> {
    if self.tasks.is_empty() {
      return Ok(());
    }

    self.pause_tui().await.unwrap();

    let selected = self.current_selection;
    let task_id = self.tasks[selected].id().unwrap_or_default();
    let task_uuid = *self.tasks[selected].uuid();

    let r = std::process::Command::new(&self.task_exe)
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

    self.resume_tui().await.unwrap();

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
      add_tag(&mut tasks[l_i], "UNBLOCKED".to_string());
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
              remove_tag(&mut tasks[l_i], "UNBLOCKED");
              add_tag(&mut tasks[l_i], "BLOCKED".to_string());
              add_tag(&mut tasks[r_i], "BLOCKING".to_string());
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
        TaskStatus::Waiting => add_tag(task, "WAITING".to_string()),
        TaskStatus::Completed => add_tag(task, "COMPLETED".to_string()),
        TaskStatus::Pending => add_tag(task, "PENDING".to_string()),
        TaskStatus::Deleted => add_tag(task, "DELETED".to_string()),
        TaskStatus::Recurring => (),
      }
      if task.start().is_some() {
        add_tag(task, "ACTIVE".to_string());
      }
      if task.scheduled().is_some() {
        add_tag(task, "SCHEDULED".to_string());
      }
      if task.parent().is_some() {
        add_tag(task, "INSTANCE".to_string());
      }
      if task.until().is_some() {
        add_tag(task, "UNTIL".to_string());
      }
      if task.annotations().is_some() {
        add_tag(task, "ANNOTATED".to_string());
      }
      let virtual_tags = self.task_report_table.virtual_tags.clone();
      if task.tags().is_some() && task.tags().unwrap().iter().any(|s| !virtual_tags.contains(s)) {
        add_tag(task, "TAGGED".to_string());
      }
      if !task.uda().is_empty() {
        add_tag(task, "UDA".to_string());
      }
      if task.mask().is_some() {
        add_tag(task, "TEMPLATE".to_string());
      }
      if task.project().is_some() {
        add_tag(task, "PROJECT".to_string());
      }
      if task.priority().is_some() {
        add_tag(task, "PRIORITY".to_string());
      }
      if task.recur().is_some() {
        add_tag(task, "RECURRING".to_string());
        let r = task.recur().unwrap();
      }
      if let Some(d) = task.due() {
        let status = task.status();
        // due today
        if status != &TaskStatus::Completed && status != &TaskStatus::Deleted {
          let now = Local::now();
          let reference = datetime::local_from_utc(d);
          let d = d.clone();
          if (reference - chrono::Duration::nanoseconds(1)).month() == now.month() {
            add_tag(task, "MONTH".to_string());
          }
          if (reference - chrono::Duration::nanoseconds(1)).month() % 4 == now.month() % 4 {
            add_tag(task, "QUARTER".to_string());
          }
          if reference.year() == now.year() {
            add_tag(task, "YEAR".to_string());
          }
          match get_date_state(&d, self.config.due) {
            DateState::EarlierToday | DateState::LaterToday => {
              add_tag(task, "DUE".to_string());
              add_tag(task, "TODAY".to_string());
              add_tag(task, "DUETODAY".to_string());
            }
            DateState::AfterToday => {
              add_tag(task, "DUE".to_string());
              if reference.date_naive() == (now + chrono::Duration::days(1)).date_naive() {
                add_tag(task, "TOMORROW".to_string());
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
            add_tag(task, "OVERDUE".to_string());
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

  pub async fn handle_input(&mut self, input: KeyCode) -> Result<()> {
    match self.mode {
      Mode::Tasks(_) => {
        self.handle_input_by_task_mode(input).await?;
      }
      Mode::Projects => {
        ProjectsState::handle_input(self, input)?;
        self.update(false).await?;
      }
      Mode::Calendar => {
        if input == self.keyconfig.quit || input == KeyCode::Ctrl('c') {
          self.should_quit = true;
        } else if input == self.keyconfig.next_tab {
          if self.config.uda_change_focus_rotate {
            self.mode = Mode::Tasks(Action::Report);
          }
        } else if input == self.keyconfig.previous_tab {
          self.mode = Mode::Projects;
        } else if input == KeyCode::Up || input == self.keyconfig.up {
          if self.calendar_year > 0 {
            self.calendar_year -= 1;
          }
        } else if input == KeyCode::Down || input == self.keyconfig.down {
          self.calendar_year += 1;
        } else if input == KeyCode::PageUp || input == self.keyconfig.page_up {
          self.task_report_previous_page();
        } else if input == KeyCode::PageDown || input == self.keyconfig.page_down {
          self.calendar_year += 10;
        } else if input == KeyCode::Ctrl('e') {
          self.task_details_scroll_down();
        } else if input == KeyCode::Ctrl('y') {
          self.task_details_scroll_up();
        } else if input == self.keyconfig.done {
          if self.config.uda_task_report_prompt_on_done {
            self.mode = Mode::Tasks(Action::DonePrompt);
            if self.task_current().is_none() {
              self.mode = Mode::Tasks(Action::Report);
            }
          } else {
            match self.task_done() {
              Ok(_) => self.update(true).await?,
              Err(e) => {
                self.error = Some(e);
                self.mode = Mode::Tasks(Action::Error);
              }
            }
            if self.calendar_year > 0 {
              self.calendar_year -= 10;
            }
          }
        }
      }
    }
    self.update_task_table_state();
    Ok(())
  }

  async fn handle_input_by_task_mode(&mut self, input: KeyCode) -> Result<()> {
    if let Mode::Tasks(task_mode) = &self.mode {
      match task_mode {
        Action::Report => {
          if input == KeyCode::Esc {
            self.marked.clear();
          } else if input == self.keyconfig.quit || input == KeyCode::Ctrl('c') {
            self.should_quit = true;
          } else if input == self.keyconfig.select {
            self.task_table_state.multiple_selection();
            self.toggle_mark();
          } else if input == self.keyconfig.select_all {
            self.task_table_state.multiple_selection();
            self.toggle_mark_all();
          } else if input == self.keyconfig.refresh {
            self.update(true).await?;
          } else if input == self.keyconfig.go_to_bottom || input == KeyCode::End {
            self.task_report_bottom();
          } else if input == self.keyconfig.go_to_top || input == KeyCode::Home {
            self.task_report_top();
          } else if input == KeyCode::Down || input == self.keyconfig.down {
            self.task_report_next();
          } else if input == KeyCode::Up || input == self.keyconfig.up {
            self.task_report_previous();
          } else if input == KeyCode::PageDown || input == self.keyconfig.page_down {
            self.task_report_next_page();
          } else if input == KeyCode::PageUp || input == self.keyconfig.page_up {
            self.task_report_previous_page();
          } else if input == KeyCode::Ctrl('e') {
            self.task_details_scroll_down();
          } else if input == KeyCode::Ctrl('y') {
            self.task_details_scroll_up();
          } else if input == self.keyconfig.done {
            if self.config.uda_task_report_prompt_on_done {
              self.mode = Mode::Tasks(Action::DonePrompt);
              if self.task_current().is_none() {
                self.mode = Mode::Tasks(Action::Report);
              }
            } else {
              match self.task_done() {
                Ok(_) => self.update(true).await?,
                Err(e) => {
                  self.error = Some(e);
                  self.mode = Mode::Tasks(Action::Error);
                }
              }
            }
          } else if input == self.keyconfig.delete {
            if self.config.uda_task_report_prompt_on_delete {
              self.mode = Mode::Tasks(Action::DeletePrompt);
              if self.task_current().is_none() {
                self.mode = Mode::Tasks(Action::Report);
              }
            } else {
              match self.task_delete() {
                Ok(_) => self.update(true).await?,
                Err(e) => {
                  self.error = Some(e);
                  self.mode = Mode::Tasks(Action::Error);
                }
              }
            }
          } else if input == self.keyconfig.start_stop {
            match self.task_start_stop() {
              Ok(_) => self.update(true).await?,
              Err(e) => {
                self.error = Some(e);
                self.mode = Mode::Tasks(Action::Error);
              }
            }
          } else if input == self.keyconfig.quick_tag {
            match self.task_quick_tag() {
              Ok(_) => self.update(true).await?,
              Err(e) => {
                self.error = Some(e);
                self.mode = Mode::Tasks(Action::Error);
              }
            }
          } else if input == self.keyconfig.edit {
            match self.task_edit().await {
              Ok(_) => self.update(true).await?,
              Err(e) => {
                self.error = Some(e);
                self.mode = Mode::Tasks(Action::Error);
              }
            }
          } else if input == self.keyconfig.duplicate {
            match self.task_duplicate() {
              Ok(_) => self.update(true).await?,
              Err(e) => {
                self.error = Some(e);
                self.mode = Mode::Tasks(Action::Error);
              }
            }
          } else if input == self.keyconfig.undo {
            if self.config.uda_task_report_prompt_on_undo {
              self.mode = Mode::Tasks(Action::UndoPrompt);
              if self.task_current().is_none() {
                self.mode = Mode::Tasks(Action::Report);
              }
            } else {
              match self.task_undo() {
                Ok(_) => self.update(true).await?,
                Err(e) => {
                  self.error = Some(e);
                  self.mode = Mode::Tasks(Action::Error);
                }
              }
            }
          } else if input == self.keyconfig.modify {
            self.mode = Mode::Tasks(Action::Modify);
            self.command_history.reset();
            self.history_status = Some(format!(
              "{} / {}",
              self
                .command_history
                .history_index()
                .unwrap_or_else(|| self.command_history.history_len().saturating_sub(1))
                .saturating_add(1),
              self.command_history.history_len()
            ));
            self.update_completion_list();
            match self.task_table_state.mode() {
              TableMode::SingleSelection => match self.task_current() {
                Some(t) => {
                  let mut s = format!("{} ", Self::escape(t.description()));
                  if self.config.uda_prefill_task_metadata {
                    if t.tags().is_some() {
                      let virtual_tags = self.task_report_table.virtual_tags.clone();
                      for tag in t.tags().unwrap() {
                        if !virtual_tags.contains(tag) {
                          s = format!("{}+{} ", s, tag);
                        }
                      }
                    }
                    if t.project().is_some() {
                      s = format!("{}project:{} ", s, t.project().unwrap());
                    }
                    if t.priority().is_some() {
                      s = format!("{}priority:{} ", s, t.priority().unwrap());
                    }
                    if t.due().is_some() {
                      let date = t.due().unwrap();
                      s = format!("{}due:{} ", s, get_formatted_datetime(date));
                    }
                  }
                  self.modify.update(&s, s.as_str().len(), &mut self.changes);
                }
                None => self.modify.update("", 0, &mut self.changes),
              },
              TableMode::MultipleSelection => self.modify.update("", 0, &mut self.changes),
            }
          } else if input == self.keyconfig.shell {
            self.mode = Mode::Tasks(Action::Subprocess);
          } else if input == self.keyconfig.log {
            self.mode = Mode::Tasks(Action::Log);
            self.command_history.reset();
            self.history_status = Some(format!(
              "{} / {}",
              self
                .command_history
                .history_index()
                .unwrap_or_else(|| self.command_history.history_len().saturating_sub(1))
                .saturating_add(1),
              self.command_history.history_len()
            ));
            self.update_completion_list();
          } else if input == self.keyconfig.add {
            self.mode = Mode::Tasks(Action::Add);
            self.command_history.reset();
            self.history_status = Some(format!(
              "{} / {}",
              self
                .command_history
                .history_index()
                .unwrap_or_else(|| self.command_history.history_len().saturating_sub(1))
                .saturating_add(1),
              self.command_history.history_len()
            ));
            self.update_completion_list();
          } else if input == self.keyconfig.annotate {
            self.mode = Mode::Tasks(Action::Annotate);
            self.command_history.reset();
            self.history_status = Some(format!(
              "{} / {}",
              self
                .command_history
                .history_index()
                .unwrap_or_else(|| self.command_history.history_len().saturating_sub(1))
                .saturating_add(1),
              self.command_history.history_len()
            ));
            self.update_completion_list();
          } else if input == self.keyconfig.help {
            self.mode = Mode::Tasks(Action::HelpPopup);
          } else if input == self.keyconfig.filter {
            self.mode = Mode::Tasks(Action::Filter);
            self.filter_history.reset();
            self.history_status = Some(format!(
              "{} / {}",
              self
                .filter_history
                .history_index()
                .unwrap_or_else(|| self.filter_history.history_len().saturating_sub(1))
                .saturating_add(1),
              self.filter_history.history_len()
            ));
            self.update_completion_list();
          } else if input == KeyCode::Char(':') {
            self.mode = Mode::Tasks(Action::Jump);
          } else if input == self.keyconfig.shortcut1 {
            match self.task_shortcut(1).await {
              Ok(_) => self.update(true).await?,
              Err(e) => {
                self.update(true).await?;
                self.error = Some(e);
                self.mode = Mode::Tasks(Action::Error);
              }
            }
          } else if input == self.keyconfig.priority_h {
            match self.task_priority("H") {
              Ok(_) => self.update(true).await?,
              Err(e) => {
                self.error = Some(e);
                self.mode = Mode::Tasks(Action::Error);
              }
            }
          } else if input == self.keyconfig.priority_m {
            match self.task_priority("M") {
              Ok(_) => self.update(true).await?,
              Err(e) => {
                self.error = Some(e);
                self.mode = Mode::Tasks(Action::Error);
              }
            }
          } else if input == self.keyconfig.priority_l {
            match self.task_priority("L") {
              Ok(_) => self.update(true).await?,
              Err(e) => {
                self.error = Some(e);
                self.mode = Mode::Tasks(Action::Error);
              }
            }
          } else if input == self.keyconfig.priority_n {
            match self.task_priority("") {
              Ok(_) => self.update(true).await?,
              Err(e) => {
                self.error = Some(e);
                self.mode = Mode::Tasks(Action::Error);
              }
            }
          } else if input == self.keyconfig.shortcut2 {
            match self.task_shortcut(2).await {
              Ok(_) => self.update(true).await?,
              Err(e) => {
                self.update(true).await?;
                self.error = Some(e);
                self.mode = Mode::Tasks(Action::Error);
              }
            }
          } else if input == self.keyconfig.shortcut3 {
            match self.task_shortcut(3).await {
              Ok(_) => self.update(true).await?,
              Err(e) => {
                self.update(true).await?;
                self.error = Some(e);
                self.mode = Mode::Tasks(Action::Error);
              }
            }
          } else if input == self.keyconfig.shortcut4 {
            match self.task_shortcut(4).await {
              Ok(_) => self.update(true).await?,
              Err(e) => {
                self.update(true).await?;
                self.error = Some(e);
                self.mode = Mode::Tasks(Action::Error);
              }
            }
          } else if input == self.keyconfig.shortcut5 {
            match self.task_shortcut(5).await {
              Ok(_) => self.update(true).await?,
              Err(e) => {
                self.update(true).await?;
                self.error = Some(e);
                self.mode = Mode::Tasks(Action::Error);
              }
            }
          } else if input == self.keyconfig.shortcut6 {
            match self.task_shortcut(6).await {
              Ok(_) => self.update(true).await?,
              Err(e) => {
                self.update(true).await?;
                self.error = Some(e);
                self.mode = Mode::Tasks(Action::Error);
              }
            }
          } else if input == self.keyconfig.shortcut7 {
            match self.task_shortcut(7).await {
              Ok(_) => self.update(true).await?,
              Err(e) => {
                self.update(true).await?;
                self.error = Some(e);
                self.mode = Mode::Tasks(Action::Error);
              }
            }
          } else if input == self.keyconfig.shortcut8 {
            match self.task_shortcut(8).await {
              Ok(_) => self.update(true).await?,
              Err(e) => {
                self.update(true).await?;
                self.error = Some(e);
                self.mode = Mode::Tasks(Action::Error);
              }
            }
          } else if input == self.keyconfig.shortcut9 {
            match self.task_shortcut(9).await {
              Ok(_) => self.update(true).await?,
              Err(e) => {
                self.update(true).await?;
                self.error = Some(e);
                self.mode = Mode::Tasks(Action::Error);
              }
            }
          } else if input == self.keyconfig.zoom {
            self.task_report_show_info = !self.task_report_show_info;
          } else if input == self.keyconfig.transpose {
            self.tasklist_vertical = !self.tasklist_vertical
          } else if input == self.keyconfig.context_menu {
            self.contexts.search.clear();
            // Pre-select the active context.
            let active_pos = self
              .contexts
              .filtered_indices()
              .iter()
              .position(|&ri| self.contexts.rows[ri].active == "yes")
              .unwrap_or(0);
            self.contexts.table_state.select(Some(active_pos));
            self.mode = Mode::Tasks(Action::ContextMenu);
          } else if input == self.keyconfig.report_menu {
            self.reports.search.clear();
            // Pre-select the active report.
            let active_pos = self
              .reports
              .filtered_indices()
              .iter()
              .position(|&ri| self.reports.rows[ri].active == "yes")
              .unwrap_or(0);
            self.reports.table_state.select(Some(active_pos));
            self.mode = Mode::Tasks(Action::ReportMenu);
          } else if input == self.keyconfig.previous_tab {
            if self.config.uda_change_focus_rotate {
              self.mode = Mode::Calendar;
            }
          } else if input == self.keyconfig.next_tab {
            self.mode = Mode::Projects;
          }
        }
        Action::ContextMenu => {
          match input {
            // Esc: clear search first; if already empty, close the menu.
            KeyCode::Esc => {
              if !self.contexts.search.is_empty() {
                let previous_row_index = self.context_selected_row_index();
                self.contexts.search.clear();
                self.restore_context_selection(previous_row_index);
                self.maybe_autoselect_context_menu().await?;
              } else {
                self.mode = Mode::Tasks(Action::Report);
              }
            }
            // Backspace removes the last search character.
            KeyCode::Backspace | KeyCode::Ctrl('h') => {
              let previous_row_index = self.context_selected_row_index();
              self.contexts.search.pop();
              self.restore_context_selection(previous_row_index);
              self.maybe_autoselect_context_menu().await?;
            }
            KeyCode::Char('\n') => {
              if self.error.is_some() {
                self.previous_mode = Some(self.mode.clone());
                self.mode = Mode::Tasks(Action::Error);
              } else {
                match self.context_select() {
                  Ok(true) => {
                    self.update(true).await?;
                    if self.config.uda_context_menu_close_on_select {
                      self.mode = Mode::Tasks(Action::Report);
                    }
                  }
                  Ok(false) => {}
                  Err(e) => {
                    self.error = Some(e.to_string());
                    self.mode = Mode::Tasks(Action::Error);
                  }
                }
              }
            }
            // Printable characters feed the search bar (except quit when search is empty).
            KeyCode::Char(c) => {
              if self.contexts.search.is_empty() && input == self.keyconfig.quit {
                self.mode = Mode::Tasks(Action::Report);
              } else {
                let previous_row_index = self.context_selected_row_index();
                self.contexts.search.push(c);
                self.restore_context_selection(previous_row_index);
                self.maybe_autoselect_context_menu().await?;
              }
            }
            _ if input == KeyCode::Down || input == self.keyconfig.down => {
              self.context_next();
              self.maybe_autoselect_context_menu().await?;
            }
            _ if input == KeyCode::Up || input == self.keyconfig.up => {
              self.context_previous();
              self.maybe_autoselect_context_menu().await?;
            }
            _ => {}
          }
        }
        Action::ReportMenu => {
          match input {
            // Esc: clear search first; if already empty, close the menu.
            KeyCode::Esc => {
              if !self.reports.search.is_empty() {
                let previous_row_index = self.report_selected_row_index();
                self.reports.search.clear();
                self.restore_report_selection(previous_row_index);
                self.maybe_autoselect_report_menu().await?;
              } else {
                self.mode = Mode::Tasks(Action::Report);
              }
            }
            // Backspace removes the last search character.
            KeyCode::Backspace | KeyCode::Ctrl('h') => {
              let previous_row_index = self.report_selected_row_index();
              self.reports.search.pop();
              self.restore_report_selection(previous_row_index);
              self.maybe_autoselect_report_menu().await?;
            }
            KeyCode::Char('\n') => {
              if self.error.is_some() {
                self.previous_mode = Some(self.mode.clone());
                self.mode = Mode::Tasks(Action::Error);
              } else {
                let data = Self::task_show_output(&self.task_exe)?;
                match self.report_select(&data) {
                  Ok(true) => {
                    self.update(true).await?;
                    if self.config.uda_report_menu_close_on_select {
                      self.mode = Mode::Tasks(Action::Report);
                    }
                  }
                  Ok(false) => {}
                  Err(e) => {
                    self.error = Some(e.to_string());
                    self.mode = Mode::Tasks(Action::Error);
                  }
                }
              }
            }
            // Printable characters feed the search bar (except quit when search is empty).
            KeyCode::Char(c) => {
              if self.reports.search.is_empty() && input == self.keyconfig.quit {
                self.mode = Mode::Tasks(Action::Report);
              } else {
                let previous_row_index = self.report_selected_row_index();
                self.reports.search.push(c);
                self.restore_report_selection(previous_row_index);
                self.maybe_autoselect_report_menu().await?;
              }
            }
            _ if input == KeyCode::Down || input == self.keyconfig.down => {
              self.report_next();
              self.maybe_autoselect_report_menu().await?;
            }
            _ if input == KeyCode::Up || input == self.keyconfig.up => {
              self.report_previous();
              self.maybe_autoselect_report_menu().await?;
            }
            _ => {}
          }
        }
        Action::HelpPopup => {
          if input == self.keyconfig.quit || input == KeyCode::Esc {
            self.mode = Mode::Tasks(Action::Report);
          } else if input == self.keyconfig.down {
            self.help_popup.scroll = self.help_popup.scroll.checked_add(1).unwrap_or(0);
            let th = (self.help_popup.text_height as u16).saturating_sub(1);
            if self.help_popup.scroll > th {
              self.help_popup.scroll = th;
            }
          } else if input == self.keyconfig.up {
            self.help_popup.scroll = self.help_popup.scroll.saturating_sub(1);
          }
        }
        Action::Modify => match input {
          KeyCode::Esc => {
            if self.show_completion_pane {
              self.show_completion_pane = false;
              self.completion_list.unselect();
            } else {
              self.modify.update("", 0, &mut self.changes);
              self.mode = Mode::Tasks(Action::Report);
            }
          }
          KeyCode::Char('\n') => {
            if self.show_completion_pane {
              self.show_completion_pane = false;
              if let Some((_, (r, _, o, _, _))) = self.completion_list.selected() {
                Self::apply_completion_to_buffer(&mut self.modify, &r, &o, &mut self.changes);
              }
              self.completion_list.unselect();
            } else if self.error.is_some() {
              self.previous_mode = Some(self.mode.clone());
              self.mode = Mode::Tasks(Action::Error);
            } else {
              match self.task_modify() {
                Ok(_) => {
                  self.mode = Mode::Tasks(Action::Report);
                  self.command_history.add(self.modify.as_str());
                  self.modify.update("", 0, &mut self.changes);
                  self.update(true).await?;
                }
                Err(e) => {
                  self.error = Some(e);
                  self.mode = Mode::Tasks(Action::Error);
                }
              }
            }
          }
          KeyCode::Tab | KeyCode::Ctrl('n') => {
            if !self.completion_list.is_empty() {
              self.update_input_for_completion();
              let candidates = self.completion_list.candidates();
              if candidates.len() == 1 {
                let (r, _, o, _, _) = candidates.into_iter().next().unwrap();
                Self::apply_completion_to_buffer(&mut self.modify, &r, &o, &mut self.changes);
                self.show_completion_pane = false;
                self.completion_list.unselect();
                self.update_input_for_completion();
              } else {
                if !self.show_completion_pane {
                  self.show_completion_pane = true;
                }
                self.completion_list.next();
              }
            }
          }
          KeyCode::BackTab | KeyCode::Ctrl('p') => {
            if self.show_completion_pane && !self.completion_list.is_empty() {
              self.completion_list.previous();
            }
          }

          KeyCode::Up => {
            if self.show_completion_pane && !self.completion_list.is_empty() {
              self.completion_list.previous();
            } else if let Some(s) = self
              .command_history
              .history_search(&self.modify.as_str()[..self.modify.pos()], HistoryDirection::Reverse)
            {
              let p = self.modify.pos();
              self.modify.update("", 0, &mut self.changes);
              self.modify.update(&s, std::cmp::min(s.len(), p), &mut self.changes);
              self.history_status = Some(format!(
                "{} / {}",
                self
                  .command_history
                  .history_index()
                  .unwrap_or_else(|| self.command_history.history_len().saturating_sub(1))
                  .saturating_add(1),
                self.command_history.history_len()
              ));
            }
          }
          KeyCode::Down => {
            if self.show_completion_pane && !self.completion_list.is_empty() {
              self.completion_list.next();
            } else if let Some(s) = self
              .command_history
              .history_search(&self.modify.as_str()[..self.modify.pos()], HistoryDirection::Forward)
            {
              let p = self.modify.pos();
              self.modify.update("", 0, &mut self.changes);
              self.modify.update(&s, std::cmp::min(s.len(), p), &mut self.changes);
              self.history_status = Some(format!(
                "{} / {}",
                self
                  .command_history
                  .history_index()
                  .unwrap_or_else(|| self.command_history.history_len().saturating_sub(1))
                  .saturating_add(1),
                self.command_history.history_len()
              ));
            }
          }
          _ => {
            self.command_history.reset();
            handle_movement(&mut self.modify, input, &mut self.changes);
            self.update_input_for_completion();
          }
        },
        Action::Subprocess => match input {
          KeyCode::Char('\n') => {
            if self.error.is_some() {
              self.previous_mode = Some(self.mode.clone());
              self.mode = Mode::Tasks(Action::Error);
            } else {
              match self.task_subprocess() {
                Ok(_) => {
                  self.mode = Mode::Tasks(Action::Report);
                  self.reset_command();
                  self.update(true).await?;
                }
                Err(e) => {
                  self.error = Some(e);
                  self.mode = Mode::Tasks(Action::Error);
                }
              }
            }
          }
          KeyCode::Esc => {
            self.reset_command();
            self.mode = Mode::Tasks(Action::Report);
          }
          _ => handle_movement(&mut self.command, input, &mut self.changes),
        },
        Action::Log => match input {
          KeyCode::Esc => {
            if self.show_completion_pane {
              self.show_completion_pane = false;
              self.completion_list.unselect();
            } else {
              self.reset_command();
              self.history_status = None;
              self.mode = Mode::Tasks(Action::Report);
            }
          }
          KeyCode::Char('\n') => {
            if self.show_completion_pane {
              self.show_completion_pane = false;
              if let Some((_, (r, _, o, _, _))) = self.completion_list.selected() {
                Self::apply_completion_to_buffer(&mut self.command, &r, &o, &mut self.changes);
              }
              self.completion_list.unselect();
            } else if self.error.is_some() {
              self.previous_mode = Some(self.mode.clone());
              self.mode = Mode::Tasks(Action::Error);
            } else {
              match self.task_log() {
                Ok(_) => {
                  self.mode = Mode::Tasks(Action::Report);
                  self.command_history.add(self.command.as_str());
                  self.reset_command();
                  self.history_status = None;
                  self.update(true).await?;
                }
                Err(e) => {
                  self.error = Some(e);
                  self.mode = Mode::Tasks(Action::Error);
                }
              }
            }
          }
          KeyCode::Tab | KeyCode::Ctrl('n') => {
            if !self.completion_list.is_empty() {
              self.update_input_for_completion();
              let candidates = self.completion_list.candidates();
              if candidates.len() == 1 {
                let (r, _, o, _, _) = candidates.into_iter().next().unwrap();
                Self::apply_completion_to_buffer(&mut self.command, &r, &o, &mut self.changes);
                self.show_completion_pane = false;
                self.completion_list.unselect();
                self.update_input_for_completion();
              } else {
                if !self.show_completion_pane {
                  self.show_completion_pane = true;
                }
                self.completion_list.next();
              }
            }
          }
          KeyCode::BackTab | KeyCode::Ctrl('p') => {
            if self.show_completion_pane && !self.completion_list.is_empty() {
              self.completion_list.previous();
            }
          }

          KeyCode::Up => {
            if self.show_completion_pane && !self.completion_list.is_empty() {
              self.completion_list.previous();
            } else if let Some(s) = self
              .command_history
              .history_search(&self.command.as_str()[..self.command.pos()], HistoryDirection::Reverse)
            {
              let p = self.command.pos();
              self.command.update("", 0, &mut self.changes);
              self.command.update(&s, std::cmp::min(s.len(), p), &mut self.changes);
              self.history_status = Some(format!(
                "{} / {}",
                self
                  .command_history
                  .history_index()
                  .unwrap_or_else(|| self.command_history.history_len().saturating_sub(1))
                  .saturating_add(1),
                self.command_history.history_len()
              ));
            }
          }
          KeyCode::Down => {
            if self.show_completion_pane && !self.completion_list.is_empty() {
              self.completion_list.next();
            } else if let Some(s) = self
              .command_history
              .history_search(&self.command.as_str()[..self.command.pos()], HistoryDirection::Forward)
            {
              let p = self.command.pos();
              self.command.update("", 0, &mut self.changes);
              self.command.update(&s, std::cmp::min(s.len(), p), &mut self.changes);
              self.history_status = Some(format!(
                "{} / {}",
                self
                  .command_history
                  .history_index()
                  .unwrap_or_else(|| self.command_history.history_len().saturating_sub(1))
                  .saturating_add(1),
                self.command_history.history_len()
              ));
            }
          }
          _ => {
            self.command_history.reset();
            handle_movement(&mut self.command, input, &mut self.changes);
            self.update_input_for_completion();
          }
        },
        Action::Annotate => match input {
          KeyCode::Esc => {
            if self.show_completion_pane {
              self.show_completion_pane = false;
              self.completion_list.unselect();
            } else {
              self.reset_command();
              self.mode = Mode::Tasks(Action::Report);
              self.history_status = None;
            }
          }
          KeyCode::Char('\n') => {
            if self.show_completion_pane {
              self.show_completion_pane = false;
              if let Some((_, (r, _, o, _, _))) = self.completion_list.selected() {
                Self::apply_completion_to_buffer(&mut self.command, &r, &o, &mut self.changes);
              }
              self.completion_list.unselect();
            } else if self.error.is_some() {
              self.previous_mode = Some(self.mode.clone());
              self.mode = Mode::Tasks(Action::Error);
            } else {
              match self.task_annotate() {
                Ok(_) => {
                  self.mode = Mode::Tasks(Action::Report);
                  self.command_history.add(self.command.as_str());
                  self.reset_command();
                  self.history_status = None;
                  self.update(true).await?;
                }
                Err(e) => {
                  self.error = Some(e);
                  self.mode = Mode::Tasks(Action::Error);
                }
              }
            }
          }
          KeyCode::Tab | KeyCode::Ctrl('n') => {
            if !self.completion_list.is_empty() {
              self.update_input_for_completion();
              let candidates = self.completion_list.candidates();
              if candidates.len() == 1 {
                let (r, _, o, _, _) = candidates.into_iter().next().unwrap();
                Self::apply_completion_to_buffer(&mut self.command, &r, &o, &mut self.changes);
                self.show_completion_pane = false;
                self.completion_list.unselect();
                self.update_input_for_completion();
              } else {
                if !self.show_completion_pane {
                  self.show_completion_pane = true;
                }
                self.completion_list.next();
              }
            }
          }
          KeyCode::BackTab | KeyCode::Ctrl('p') => {
            if self.show_completion_pane && !self.completion_list.is_empty() {
              self.completion_list.previous();
            }
          }
          KeyCode::Up => {
            if self.show_completion_pane && !self.completion_list.is_empty() {
              self.completion_list.previous();
            } else if let Some(s) = self
              .command_history
              .history_search(&self.command.as_str()[..self.command.pos()], HistoryDirection::Reverse)
            {
              let p = self.command.pos();
              self.command.update("", 0, &mut self.changes);
              self.command.update(&s, std::cmp::min(s.len(), p), &mut self.changes);
              self.history_status = Some(format!(
                "{} / {}",
                self
                  .command_history
                  .history_index()
                  .unwrap_or_else(|| self.command_history.history_len().saturating_sub(1))
                  .saturating_add(1),
                self.command_history.history_len()
              ));
            }
          }
          KeyCode::Down => {
            if self.show_completion_pane && !self.completion_list.is_empty() {
              self.completion_list.next();
            } else if let Some(s) = self
              .command_history
              .history_search(&self.command.as_str()[..self.command.pos()], HistoryDirection::Forward)
            {
              let p = self.command.pos();
              self.command.update("", 0, &mut self.changes);
              self.command.update(&s, std::cmp::min(s.len(), p), &mut self.changes);
              self.history_status = Some(format!(
                "{} / {}",
                self
                  .command_history
                  .history_index()
                  .unwrap_or_else(|| self.command_history.history_len().saturating_sub(1))
                  .saturating_add(1),
                self.command_history.history_len()
              ));
            }
          }

          _ => {
            self.command_history.reset();
            handle_movement(&mut self.command, input, &mut self.changes);
            self.update_input_for_completion();
          }
        },
        Action::Jump => match input {
          KeyCode::Char('\n') => {
            if self.error.is_some() {
              self.previous_mode = Some(self.mode.clone());
              self.mode = Mode::Tasks(Action::Error);
            } else {
              match self.task_report_jump() {
                Ok(_) => {
                  self.mode = Mode::Tasks(Action::Report);
                  self.reset_command();
                  self.update(true).await?;
                }
                Err(e) => {
                  self.reset_command();
                  self.error = Some(e.to_string());
                  self.mode = Mode::Tasks(Action::Error);
                }
              }
            }
          }
          KeyCode::Esc => {
            self.reset_command();
            self.mode = Mode::Tasks(Action::Report);
          }
          _ => handle_movement(&mut self.command, input, &mut self.changes),
        },
        Action::Add => match input {
          KeyCode::Esc => {
            if self.show_completion_pane {
              self.show_completion_pane = false;
              self.completion_list.unselect();
            } else {
              self.reset_command();
              self.history_status = None;
              self.mode = Mode::Tasks(Action::Report);
            }
          }
          KeyCode::Char('\n') => {
            if self.show_completion_pane {
              self.show_completion_pane = false;
              if let Some((_, (r, _, o, _, _))) = self.completion_list.selected() {
                Self::apply_completion_to_buffer(&mut self.command, &r, &o, &mut self.changes);
              }
              self.completion_list.unselect();
            } else if self.error.is_some() {
              self.previous_mode = Some(self.mode.clone());
              self.mode = Mode::Tasks(Action::Error);
            } else {
              match self.task_add() {
                Ok(_) => {
                  self.mode = Mode::Tasks(Action::Report);
                  self.command_history.add(self.command.as_str());
                  self.reset_command();
                  self.history_status = None;
                  self.update(true).await?;
                }
                Err(e) => {
                  self.error = Some(e);
                  self.mode = Mode::Tasks(Action::Error);
                }
              }
            }
          }
          KeyCode::Tab | KeyCode::Ctrl('n') => {
            if !self.completion_list.is_empty() {
              self.update_input_for_completion();
              let candidates = self.completion_list.candidates();
              if candidates.len() == 1 {
                let (r, _, o, _, _) = candidates.into_iter().next().unwrap();
                Self::apply_completion_to_buffer(&mut self.command, &r, &o, &mut self.changes);
                self.show_completion_pane = false;
                self.completion_list.unselect();
                self.update_input_for_completion();
              } else {
                if !self.show_completion_pane {
                  self.show_completion_pane = true;
                }
                self.completion_list.next();
              }
            }
          }
          KeyCode::BackTab | KeyCode::Ctrl('p') => {
            if self.show_completion_pane && !self.completion_list.is_empty() {
              self.completion_list.previous();
            }
          }
          KeyCode::Up => {
            if self.show_completion_pane && !self.completion_list.is_empty() {
              self.completion_list.previous();
            } else if let Some(s) = self
              .command_history
              .history_search(&self.command.as_str()[..self.command.pos()], HistoryDirection::Reverse)
            {
              let p = self.command.pos();
              self.command.update("", 0, &mut self.changes);
              self.command.update(&s, std::cmp::min(s.len(), p), &mut self.changes);
              self.history_status = Some(format!(
                "{} / {}",
                self
                  .command_history
                  .history_index()
                  .unwrap_or_else(|| self.command_history.history_len().saturating_sub(1))
                  .saturating_add(1),
                self.command_history.history_len()
              ));
            }
          }

          KeyCode::Down => {
            if self.show_completion_pane && !self.completion_list.is_empty() {
              self.completion_list.next();
            } else if let Some(s) = self
              .command_history
              .history_search(&self.command.as_str()[..self.command.pos()], HistoryDirection::Forward)
            {
              let p = self.command.pos();
              self.command.update("", 0, &mut self.changes);
              self.command.update(&s, std::cmp::min(s.len(), p), &mut self.changes);
              self.history_status = Some(format!(
                "{} / {}",
                self
                  .command_history
                  .history_index()
                  .unwrap_or_else(|| self.command_history.history_len().saturating_sub(1))
                  .saturating_add(1),
                self.command_history.history_len()
              ));
            }
          }
          _ => {
            self.command_history.reset();
            handle_movement(&mut self.command, input, &mut self.changes);
            self.update_input_for_completion();
          }
        },
        Action::Filter => match input {
          KeyCode::Esc => {
            if self.show_completion_pane {
              self.show_completion_pane = false;
              self.completion_list.unselect();
            } else {
              self.mode = Mode::Tasks(Action::Report);
              self.filter_history.add(self.filter.as_str());
              if self.config.uda_reset_filter_on_esc {
                self.filter.update("", 0, &mut self.changes);
                for c in self.config.filter.chars() {
                  self.filter.insert(c, 1, &mut self.changes);
                }
                self.update_input_for_completion();
                self.dirty = true;
              }
              self.history_status = None;
              self.update(true).await?;
            }
          }
          KeyCode::Char('\n') => {
            if self.show_completion_pane {
              self.show_completion_pane = false;
              if let Some((_, (r, _, o, _, _))) = self.completion_list.selected() {
                Self::apply_completion_to_buffer(&mut self.filter, &r, &o, &mut self.changes);
              }
              self.completion_list.unselect();
              self.dirty = true;
            } else if self.error.is_some() {
              self.previous_mode = Some(self.mode.clone());
              self.mode = Mode::Tasks(Action::Error);
            } else {
              self.mode = Mode::Tasks(Action::Report);
              self.filter_history.add(self.filter.as_str());
              self.history_status = None;
              self.update(true).await?;
            }
          }
          KeyCode::Up => {
            if self.show_completion_pane && !self.completion_list.is_empty() {
              self.completion_list.previous();
            } else if let Some(s) = self
              .filter_history
              .history_search(&self.filter.as_str()[..self.filter.pos()], HistoryDirection::Reverse)
            {
              let p = self.filter.pos();
              self.filter.update("", 0, &mut self.changes);
              self.filter.update(&s, std::cmp::min(p, s.len()), &mut self.changes);
              self.history_status = Some(format!(
                "{} / {}",
                self
                  .filter_history
                  .history_index()
                  .unwrap_or_else(|| self.filter_history.history_len().saturating_sub(1))
                  .saturating_add(1),
                self.filter_history.history_len()
              ));
              self.dirty = true;
            }
          }
          KeyCode::Down => {
            if self.show_completion_pane && !self.completion_list.is_empty() {
              self.completion_list.next();
            } else if let Some(s) = self
              .filter_history
              .history_search(&self.filter.as_str()[..self.filter.pos()], HistoryDirection::Forward)
            {
              let p = self.filter.pos();
              self.filter.update("", 0, &mut self.changes);
              self.filter.update(&s, std::cmp::min(p, s.len()), &mut self.changes);
              self.history_status = Some(format!(
                "{} / {}",
                self
                  .filter_history
                  .history_index()
                  .unwrap_or_else(|| self.filter_history.history_len().saturating_sub(1))
                  .saturating_add(1),
                self.filter_history.history_len()
              ));
              self.dirty = true;
            }
          }
          KeyCode::Tab | KeyCode::Ctrl('n') => {
            if !self.completion_list.is_empty() {
              self.update_input_for_completion();
              let candidates = self.completion_list.candidates();
              if candidates.len() == 1 {
                let (r, _, o, _, _) = candidates.into_iter().next().unwrap();
                Self::apply_completion_to_buffer(&mut self.filter, &r, &o, &mut self.changes);
                self.show_completion_pane = false;
                self.completion_list.unselect();
                self.update_input_for_completion();
                self.dirty = true;
              } else {
                if !self.show_completion_pane {
                  self.show_completion_pane = true;
                }
                self.completion_list.next();
              }
            }
          }
          KeyCode::BackTab | KeyCode::Ctrl('p') => {
            if self.show_completion_pane && !self.completion_list.is_empty() {
              self.completion_list.previous();
            }
          }
          KeyCode::Ctrl('r') => {
            self.filter.update("", 0, &mut self.changes);
            for c in self.config.filter.chars() {
              self.filter.insert(c, 1, &mut self.changes);
            }
            self.history_status = None;
            self.update_input_for_completion();
            self.dirty = true;
          }
          _ => {
            handle_movement(&mut self.filter, input, &mut self.changes);
            self.update_input_for_completion();
            self.dirty = true;
          }
        },
        Action::DonePrompt => {
          if input == self.keyconfig.done || input == KeyCode::Char('\n') {
            if self.error.is_some() {
              self.previous_mode = Some(self.mode.clone());
              self.mode = Mode::Tasks(Action::Error);
            } else {
              match self.task_done() {
                Ok(_) => {
                  self.mode = Mode::Tasks(Action::Report);
                  self.update(true).await?;
                }
                Err(e) => {
                  self.error = Some(e);
                  self.mode = Mode::Tasks(Action::Error);
                }
              }
            }
          } else if input == self.keyconfig.quit || input == KeyCode::Esc {
            self.mode = Mode::Tasks(Action::Report);
          } else {
            handle_movement(&mut self.command, input, &mut self.changes);
          }
        }
        Action::DeletePrompt => {
          if input == self.keyconfig.delete || input == KeyCode::Char('\n') {
            if self.error.is_some() {
              self.previous_mode = Some(self.mode.clone());
              self.mode = Mode::Tasks(Action::Error);
            } else {
              match self.task_delete() {
                Ok(_) => {
                  self.mode = Mode::Tasks(Action::Report);
                  self.update(true).await?;
                }
                Err(e) => {
                  self.error = Some(e);
                  self.mode = Mode::Tasks(Action::Error);
                }
              }
            }
          } else if input == self.keyconfig.quit || input == KeyCode::Esc {
            self.mode = Mode::Tasks(Action::Report);
          } else {
            handle_movement(&mut self.command, input, &mut self.changes);
          }
        }
        Action::UndoPrompt => {
          if input == self.keyconfig.undo || input == KeyCode::Char('\n') {
            if self.error.is_some() {
              self.previous_mode = Some(self.mode.clone());
              self.mode = Mode::Tasks(Action::Error);
            } else {
              match self.task_undo() {
                Ok(_) => {
                  self.mode = Mode::Tasks(Action::Report);
                  self.update(true).await?;
                }
                Err(e) => {
                  self.error = Some(e);
                  self.mode = Mode::Tasks(Action::Error);
                }
              }
            }
          } else if input == self.keyconfig.quit || input == KeyCode::Esc {
            self.mode = Mode::Tasks(Action::Report);
          } else {
            handle_movement(&mut self.command, input, &mut self.changes);
          }
        }
        Action::Error => {
          // since filter live updates, don't reset error status
          // for other actions, resetting error to None is required otherwise user cannot
          // ever successfully execute mode.
          if self.previous_mode != Some(Mode::Tasks(Action::Filter)) {
            self.error = None;
          }
          self.mode = self.previous_mode.clone().unwrap_or(Mode::Tasks(Action::Report));
          self.previous_mode = None;
        }
      }
    }
    self.update_task_table_state();
    Ok(())
  }

  pub fn update_completion_list(&mut self) {
    self.completion_list.clear();

    let tasks = if self.config.uda_task_report_use_all_tasks_for_completion {
      &self.all_tasks
    } else {
      &self.tasks
    };

    if let Mode::Tasks(Action::Modify | Action::Filter | Action::Annotate | Action::Add | Action::Log) = self.mode {
      for s in [
        "project:".to_string(),
        "priority:".to_string(),
        "due:".to_string(),
        "scheduled:".to_string(),
        "wait:".to_string(),
        "depends:".to_string(),
        "tag:".to_string(),
        "recur:".to_string(),
      ] {
        self.completion_list.insert(("attribute".to_string(), s));
      }
    }

    if let Mode::Tasks(Action::Modify | Action::Filter | Action::Annotate | Action::Add | Action::Log) = self.mode {
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

    if let Mode::Tasks(Action::Modify | Action::Filter | Action::Annotate | Action::Add | Action::Log) = self.mode {
      for priority in &self.config.uda_priority_values {
        let p = priority.to_string();
        self.completion_list.insert(("priority".to_string(), p));
      }
      let virtual_tags = self.task_report_table.virtual_tags.clone();
      for task in tasks {
        if let Some(tags) = task.tags() {
          for tag in tags {
            if !virtual_tags.contains(tag) {
              self.completion_list.insert(("tag".to_string(), tag.to_string()));
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
          self.completion_list.insert(("due".to_string(), get_formatted_datetime(date)));
        }
      }
      for task in tasks {
        if let Some(date) = task.wait() {
          self.completion_list.insert(("wait".to_string(), get_formatted_datetime(date)));
        }
      }
      for task in tasks {
        if let Some(date) = task.scheduled() {
          self.completion_list.insert(("scheduled".to_string(), get_formatted_datetime(date)));
        }
      }
      for task in tasks {
        if let Some(date) = task.end() {
          self.completion_list.insert(("end".to_string(), get_formatted_datetime(date)));
        }
      }
    }

    if self.mode == Mode::Tasks(Action::Filter) {
      self.completion_list.insert(("status".to_string(), "pending".into()));
      self.completion_list.insert(("status".to_string(), "completed".into()));
      self.completion_list.insert(("status".to_string(), "deleted".into()));
      self.completion_list.insert(("status".to_string(), "recurring".into()));
    }
  }

  pub fn update_input_for_completion(&mut self) {
    match self.mode {
      Mode::Tasks(Action::Add | Action::Annotate | Action::Log) => {
        let i = get_start_word_under_cursor(self.command.as_str(), self.command.pos());
        let input = self.command.as_str()[i..self.command.pos()].to_string();
        self.completion_list.input(input, "".to_string());
      }
      Mode::Tasks(Action::Modify) => {
        let i = get_start_word_under_cursor(self.modify.as_str(), self.modify.pos());
        let input = self.modify.as_str()[i..self.modify.pos()].to_string();
        self.completion_list.input(input, "".to_string());
      }
      Mode::Tasks(Action::Filter) => {
        let i = get_start_word_under_cursor(self.filter.as_str(), self.filter.pos());
        let input = self.filter.as_str()[i..self.filter.pos()].to_string();
        self.completion_list.input(input, "".to_string());
      }
      _ => {}
    }
  }

  /// Apply a completion candidate to the given line buffer.
  /// `replacement` is the full candidate string, `original` is the already-typed prefix.
  fn apply_completion_to_buffer(buffer: &mut LineBuffer, replacement: &str, original: &str, changes: &mut utils::Changeset) {
    let (before, after) = buffer.as_str().split_at(buffer.pos());
    let fs = format!("{}{}{}", before.trim_end_matches(original), replacement, after);
    buffer.update(&fs, buffer.pos() + replacement.len() - original.len(), changes);
  }
}

pub fn handle_movement(linebuffer: &mut LineBuffer, input: KeyCode, changes: &mut utils::Changeset) {
  match input {
    KeyCode::Ctrl('f') | KeyCode::Right => {
      linebuffer.move_forward(1);
    }
    KeyCode::Ctrl('b') | KeyCode::Left => {
      linebuffer.move_backward(1);
    }
    KeyCode::Ctrl('h') | KeyCode::Backspace => {
      linebuffer.backspace(1, changes);
    }
    KeyCode::Ctrl('d') | KeyCode::Delete => {
      linebuffer.delete(1, changes);
    }
    KeyCode::Ctrl('a') | KeyCode::Home => {
      linebuffer.move_home();
    }
    KeyCode::Ctrl('e') | KeyCode::End => {
      linebuffer.move_end();
    }
    KeyCode::Ctrl('k') => {
      linebuffer.kill_line(changes);
    }
    KeyCode::Ctrl('u') => {
      linebuffer.discard_line(changes);
    }
    KeyCode::Ctrl('w') | KeyCode::AltBackspace | KeyCode::CtrlBackspace => {
      linebuffer.delete_prev_word(Word::Emacs, 1, changes);
    }
    KeyCode::Alt('d') | KeyCode::AltDelete | KeyCode::CtrlDelete => {
      linebuffer.delete_word(At::AfterEnd, Word::Emacs, 1, changes);
    }
    KeyCode::Alt('f') => {
      linebuffer.move_to_next_word(At::AfterEnd, Word::Emacs, 1);
    }
    KeyCode::Alt('b') => {
      linebuffer.move_to_prev_word(Word::Emacs, 1);
    }
    KeyCode::Alt('t') => {
      linebuffer.transpose_words(1, changes);
    }
    KeyCode::Char(c) => {
      linebuffer.insert(c, 1, changes);
    }
    _ => {}
  }
}

pub fn add_tag(task: &mut Task, tag: String) {
  match task.tags_mut() {
    Some(t) => t.push(tag),
    None => task.set_tags(Some(vec![tag])),
  }
}

pub fn remove_tag(task: &mut Task, tag: &str) {
  if let Some(t) = task.tags_mut()
    && let Some(index) = t.iter().position(|x| *x == tag)
  {
    t.remove(index);
  }
}

#[cfg(test)]
// Disabled, as "'" should be a String for more readable shlex shell escaping.
#[allow(clippy::single_char_pattern)]
mod tests {
  use std::{
    ffi::OsStr,
    fmt::Write,
    fs::File,
    io,
    path::{Path, PathBuf},
  };

  use ratatui::{backend::TestBackend, buffer::Buffer};

  use super::*;

  fn get_taskdata_path() -> PathBuf {
    let taskdata_env_var = std::env::var("TASKDATA").expect("TASKDATA environment variable not set.");
    let taskdata_path = Path::new(&taskdata_env_var).to_owned();

    taskdata_path
  }

  fn task_exe() -> String {
    std::env::var("TASKWARRIOR_TUI_TASKWARRIOR_CLI").unwrap_or_else(|_| "task".to_string())
  }

  fn assert_task_has_tags(task: &Task, expected_tags: &[&str]) {
    let task_tags = task
      .tags()
      .unwrap_or_else(|| panic!("task {} should have tags", task.id().unwrap_or_default()));

    for expected_tag in expected_tags {
      assert!(
        task_tags.contains(&expected_tag.to_string()),
        "expected tag `{expected_tag}` in {:?}",
        task_tags
      );
    }
  }

  /// Returns a string representation of the given buffer for debugging purpose.
  fn buffer_view(buffer: &Buffer) -> String {
    let mut view = String::with_capacity(buffer.content.len() + buffer.area.height as usize * 3);
    for cells in buffer.content.chunks(buffer.area.width as usize) {
      let mut overwritten = vec![];
      let mut skip: usize = 0;
      view.push('"');
      for (x, c) in cells.iter().enumerate() {
        if skip == 0 {
          view.push_str(c.symbol());
        } else {
          overwritten.push((x, c.symbol()))
        }
        skip = std::cmp::max(skip, c.symbol().width()).saturating_sub(1);
      }
      view.push('"');
      if !overwritten.is_empty() {
        write!(&mut view, " Hidden by multi-width symbols: {:?}", overwritten).unwrap();
      }
      view.push('\n');
    }
    view
  }

  #[test]
  fn test_centered_rect() {
    assert_eq!(centered_rect(50, 50, Rect::new(0, 0, 100, 100)), Rect::new(25, 25, 50, 50));
  }

  fn setup() {
    use std::process::Stdio;
    let mut f = File::open(get_taskdata_path().parent().unwrap().join("export.json")).unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    let tasks = task_hookrs::import::import(s.as_bytes()).unwrap();
    // tasks.iter_mut().find(| t | t.id().unwrap() == 1).unwrap().priority_mut().replace(&mut "H".to_string());
    // tasks.iter_mut().find(| t | t.id().unwrap() == 2).unwrap().priority_mut().replace(&mut "H".to_string());
    // tasks.iter_mut().find(| t | t.id().unwrap() == 4).unwrap().tags_mut().replace(&mut vec!["test".to_string(), "another tag".to_string()]);
    assert!(task_hookrs::tw::save(&tasks).is_ok());
  }

  fn teardown() {
    let cd = get_taskdata_path();
    if cd.exists() {
      std::fs::remove_dir_all(cd).unwrap();
    }
  }

  fn reset_test_taskdata() {
    teardown();
    std::process::Command::new(task_exe()).arg("context").arg("none").output().unwrap();
  }

  fn reset_test_context() {
    std::process::Command::new(task_exe()).arg("context").arg("none").output().unwrap();
  }

  fn configure_context_menu_selection(app: &mut TaskwarriorTui, select_on_move: bool, close_on_select: bool) {
    app.config.uda_context_menu_select_on_move = select_on_move;
    app.config.uda_context_menu_close_on_select = close_on_select;
  }

  fn enable_context_menu_move_selection(app: &mut TaskwarriorTui) {
    configure_context_menu_selection(app, true, true);
  }

  fn configure_report_menu_selection(app: &mut TaskwarriorTui, select_on_move: bool, close_on_select: bool) {
    app.config.uda_report_menu_select_on_move = select_on_move;
    app.config.uda_report_menu_close_on_select = close_on_select;
  }

  fn enable_report_menu_move_selection(app: &mut TaskwarriorTui) {
    configure_report_menu_selection(app, true, true);
  }

  fn selected_context_filter(app: &TaskwarriorTui) -> Option<String> {
    let fi = app.contexts.table_state.current_selection()?;
    let ri = *app.contexts.filtered_indices().get(fi)?;
    Some(app.contexts.rows.get(ri)?.definition.clone())
  }

  fn selected_report_name(app: &TaskwarriorTui) -> Option<String> {
    let fi = app.reports.table_state.current_selection()?;
    let ri = *app.reports.filtered_indices().get(fi)?;
    Some(app.reports.rows.get(ri)?.name.clone())
  }

  async fn test_taskwarrior_tui_history() {
    let mut app = TaskwarriorTui::new("next", false).await.unwrap();
    // setup();
    app.mode = Mode::Tasks(Action::Add);
    app.update_completion_list();
    let input = "Wash car";
    for c in input.chars() {
      app.handle_input(KeyCode::Char(c)).await.unwrap();
    }
    app.handle_input(KeyCode::Right).await.unwrap();
    let input = " +test";
    for c in input.chars() {
      app.handle_input(KeyCode::Char(c)).await.unwrap();
    }
    app.handle_input(KeyCode::Char('\n')).await.unwrap();

    app.mode = Mode::Tasks(Action::Add);

    app.update_completion_list();

    let backend = TestBackend::new(50, 15);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
      .draw(|f| {
        app.draw(f);
        app.draw(f);
      })
      .unwrap();

    let input = "Buy groceries";
    for c in input.chars() {
      app.handle_input(KeyCode::Char(c)).await.unwrap();
    }
    app.handle_input(KeyCode::Right).await.unwrap();
    let input = " +test";
    for c in input.chars() {
      app.handle_input(KeyCode::Char(c)).await.unwrap();
    }
    app.update(true).await.unwrap();
    app.handle_input(KeyCode::Down).await.unwrap();

    assert_eq!("\"Buy groceries\" +test", app.command.as_str());

    app.handle_input(KeyCode::Char('\n')).await.unwrap();

    app.mode = Mode::Tasks(Action::Add);
    app.update_completion_list();

    let backend = TestBackend::new(50, 15);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
      .draw(|f| {
        app.draw(f);
        app.draw(f);
      })
      .unwrap();

    let input = "Buy groceries";
    for c in input.chars() {
      app.handle_input(KeyCode::Char(c)).await.unwrap();
    }
    app.handle_input(KeyCode::Right).await.unwrap();
    app.handle_input(KeyCode::Backspace).await.unwrap();
    app.update(true).await.unwrap();
    app.handle_input(KeyCode::Down).await.unwrap();

    assert_eq!("\"Buy groceries", app.command.as_str());

    app.update(true).await.unwrap();

    app.handle_input(KeyCode::Up).await.unwrap();

    assert_eq!("\"Buy groceries\" +test", app.command.as_str());
    // teardown();
  }

  async fn test_multiline_paste_in_add_prompt_does_not_submit() {
    let mut app = TaskwarriorTui::new("next", false).await.unwrap();
    app.mode = Mode::Tasks(Action::Add);
    app.update_completion_list();

    app.handle_paste("hello \n world");

    assert_eq!(Mode::Tasks(Action::Add), app.mode);
    assert_eq!("hello \n world", app.command.as_str());
  }

  async fn test_draw_command_shows_control_characters() {
    let app = TaskwarriorTui::new("next", false).await.unwrap();

    let backend = TestBackend::new(20, 3);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
      .draw(|f| {
        app.draw_command(
          f,
          f.area(),
          "hello \n world",
          (Span::styled("Add Task", Style::default().add_modifier(Modifier::BOLD)), None),
          utils::display_width("hello \n world"),
          false,
          None,
          None,
        );
      })
      .unwrap();

    assert!(buffer_view(terminal.backend().buffer()).contains("\"hello ^J world      \""));
  }

  #[tokio::test]
  async fn test_taskwarrior_tui() {
    reset_test_taskdata();

    let app = TaskwarriorTui::new("next", false).await.unwrap();

    assert!(
      app.task_by_index(0).is_none(),
      "Expected task data to be empty but found {} tasks. Delete contents of {:?} and {:?} and run the tests again.",
      app.tasks.len(),
      get_taskdata_path(),
      get_taskdata_path().parent().unwrap().join(".config")
    );

    let app = TaskwarriorTui::new("next", false).await.unwrap();
    assert!(
      app
        .task_by_uuid(Uuid::parse_str("3f43831b-88dc-45e2-bf0d-4aea6db634cc").unwrap())
        .is_none()
    );

    test_draw_empty_task_report().await;

    test_draw_calendar().await;
    test_draw_help_popup().await;

    setup();

    let app = TaskwarriorTui::new("next", false).await.unwrap();
    assert!(app.task_by_index(0).is_some());

    let app = TaskwarriorTui::new("next", false).await.unwrap();
    assert!(
      app
        .task_by_uuid(Uuid::parse_str("3f43831b-88dc-45e2-bf0d-4aea6db634cc").unwrap())
        .is_some()
    );

    test_draw_task_report_with_extended_modify_command().await;
    // test_draw_task_report();
    test_task_tags().await;
    test_task_style().await;
    test_task_style_keyword_color().await;
    test_task_style_uda_color_with_spaces_in_value().await;
    test_task_report_alternate_style().await;
    test_context_menu_enter_closes_menu().await;
    test_context_menu_enter_can_stay_open().await;
    test_context_menu_enter_with_no_matches_does_not_select().await;
    test_context_menu_selection_config_combinations_select_filtered_match_on_enter().await;
    test_context_menu_selection_config_combinations_keep_menu_open_with_no_matches().await;
    test_context_menu_selection_config_combinations_preserve_selection_while_filtering().await;
    test_context_menu_selection_config_combinations_move_to_top_when_selection_is_filtered_out().await;
    test_context_menu_enter_selects_filtered_match_when_select_on_move_is_enabled().await;
    test_context_menu_type_to_filter_selects_top_match_when_select_on_move_is_enabled().await;
    test_context_menu_backspace_selects_top_match_and_stays_open_when_select_on_move_is_enabled().await;
    test_context_menu_escape_clears_search_selects_top_match_and_stays_open_when_select_on_move_is_enabled().await;
    test_context_menu_enter_with_no_matches_stays_open_when_select_on_move_is_enabled().await;
    test_report_menu_enter_closes_menu().await;
    test_report_menu_enter_can_stay_open().await;
    test_report_menu_enter_with_no_matches_does_not_select().await;
    test_report_menu_selection_config_combinations_select_filtered_match_on_enter().await;
    test_report_menu_selection_config_combinations_keep_menu_open_with_no_matches().await;
    test_report_menu_selection_config_combinations_preserve_selection_while_filtering().await;
    test_report_menu_selection_config_combinations_move_to_top_when_selection_is_filtered_out().await;
    test_report_menu_enter_selects_filtered_match_when_select_on_move_is_enabled().await;
    test_report_menu_type_to_filter_selects_top_match_when_select_on_move_is_enabled().await;
    test_report_menu_backspace_selects_top_match_and_stays_open_when_select_on_move_is_enabled().await;
    test_report_menu_escape_clears_search_selects_top_match_and_stays_open_when_select_on_move_is_enabled().await;
    test_report_menu_enter_with_no_matches_stays_open_when_select_on_move_is_enabled().await;
    test_task_context().await;
    test_task_tomorrow().await;
    test_task_earlier_today().await;
    test_task_later_today().await;
    test_task_details_are_cached_for_selected_task().await;
    test_taskwarrior_tui_history().await;
    test_multiline_paste_in_add_prompt_does_not_submit().await;
    test_draw_command_shows_control_characters().await;

    teardown();
  }

  async fn test_task_tags() {
    // testing tags
    let app = TaskwarriorTui::new("next", false).await.unwrap();
    let task = app.task_by_id(1).unwrap();

    let tags = vec!["PENDING".to_string(), "PRIORITY".to_string()];

    for tag in tags {
      assert!(task.tags().unwrap().contains(&tag));
    }

    let mut app = TaskwarriorTui::new("next", false).await.unwrap();
    let task = app.task_by_id(11).unwrap();
    let tags = ["finance", "UNBLOCKED", "PENDING", "TAGGED", "UDA"]
      .iter()
      .map(ToString::to_string)
      .collect::<Vec<String>>();
    for tag in tags {
      assert!(task.tags().unwrap().contains(&tag));
    }

    if let Some(task) = app.task_by_id(11) {
      let i = app.task_index_by_uuid(*task.uuid()).unwrap_or_default();
      app.current_selection = i;
      app.current_selection_id = None;
      app.current_selection_uuid = None;
    }

    app.task_quick_tag().unwrap();
    app.update(true).await.unwrap();

    let task = app.task_by_id(11).unwrap();
    let tags = ["next", "finance", "UNBLOCKED", "PENDING", "TAGGED", "UDA"]
      .iter()
      .map(ToString::to_string)
      .collect::<Vec<String>>();
    for tag in tags {
      assert!(task.tags().unwrap().contains(&tag));
    }

    app.task_quick_tag().unwrap();
    app.update(true).await.unwrap();

    let task = app.task_by_id(11).unwrap();
    let tags = ["finance", "UNBLOCKED", "PENDING", "TAGGED", "UDA"]
      .iter()
      .map(ToString::to_string)
      .collect::<Vec<String>>();
    for tag in tags {
      assert!(task.tags().unwrap().contains(&tag));
    }
  }

  async fn test_task_style() {
    let app = TaskwarriorTui::new("next", false).await.unwrap();
    let task = app.task_by_id(1).unwrap();
    for r in vec![
      "active",
      "blocked",
      "blocking",
      "completed",
      "deleted",
      "due",
      "due.today",
      "keyword.",
      "overdue",
      "project.",
      "recurring",
      "scheduled",
      "tag.",
      "tagged",
      "uda.",
    ] {
      assert!(app.config.rule_precedence_color.contains(&r.to_string()));
    }
    let style = app.style_for_task(&task);

    assert_eq!(style, Style::default());

    let task = app.task_by_id(11).unwrap();
    let style = app.style_for_task(&task);
  }

  async fn test_task_style_keyword_color() {
    let mut app = TaskwarriorTui::new("next", false).await.unwrap();
    let keyword_style = Style::default()
      .fg(Color::Indexed(4))
      .bg(Color::Indexed(3))
      .add_modifier(Modifier::ITALIC);
    app.config.color_keywords = vec![("urgent".to_string(), keyword_style)];

    let mut task = app.task_by_id(1).unwrap();
    *task.description_mut() = "urgent task".to_string();
    assert_eq!(app.style_for_task(&task), keyword_style);

    *task.description_mut() = "ordinary task".to_string();
    assert_eq!(app.style_for_task(&task), Style::default());
  }

  async fn test_task_style_uda_color_with_spaces_in_value() {
    let app = TaskwarriorTui::new("next", false).await.unwrap();
    let mut task = app.task_by_id(1).unwrap();

    task.uda_mut().insert("jirastatus".to_string(), UDAValue::Str("In Review".to_string()));
    assert_eq!(app.style_for_task(&task), Config::get_tcolor("black on bright cyan"));

    task.uda_mut().insert("jirastatus".to_string(), UDAValue::Str("To Do".to_string()));
    assert_eq!(app.style_for_task(&task), Config::get_tcolor("bright white"));
  }

  async fn test_task_report_alternate_style() {
    let mut app = TaskwarriorTui::new("next", false).await.unwrap();
    assert!(app.update(true).await.is_ok());

    let alternate_style = Style::default()
      .fg(Color::Indexed(5))
      .bg(Color::Indexed(2))
      .add_modifier(Modifier::ITALIC);
    app.config.color.insert("color.alternate".to_string(), alternate_style);
    let task = app.task_by_id(1).unwrap();

    let default_style = app.task_report_row_style(0, &task);
    let odd_row_style = app.task_report_row_style(1, &task);

    assert_eq!(default_style, Style::default());
    assert_eq!(odd_row_style, alternate_style);

    app.config.uda_task_report_use_alternate_style = false;
    let disabled_style = app.task_report_row_style(1, &task);

    assert_eq!(disabled_style, Style::default());
  }

  async fn test_task_details_are_cached_for_selected_task() {
    let mut app = TaskwarriorTui::new("next", false).await.unwrap();
    assert!(app.update(true).await.is_ok());
    assert!(app.update_task_details().await.is_ok());

    let task_uuid = *app.tasks[app.current_selection].uuid();
    let task_modified = app.tasks[app.current_selection].modified().cloned();
    assert!(app.task_details.contains_key(&task_uuid));

    app.task_details.insert(task_uuid, "sentinel".to_string());
    app.task_details_modified.insert(task_uuid, task_modified.clone());
    assert!(app.update_task_details().await.is_ok());
    assert_eq!(app.task_details.get(&task_uuid).map(String::as_str), Some("sentinel"));

    app.task_details_modified.insert(
      task_uuid,
      match task_modified {
        Some(_) => None,
        None => Some(Local::now().naive_utc().into()),
      },
    );
    assert!(app.update_task_details().await.is_ok());
    assert_ne!(app.task_details.get(&task_uuid).map(String::as_str), Some("sentinel"));

    app.task_details.insert(task_uuid, "sentinel".to_string());
    app
      .task_details_modified
      .insert(task_uuid, app.tasks[app.current_selection].modified().cloned());
    app.terminal_width = app.terminal_width.saturating_add(10);
    assert!(app.update_task_details().await.is_ok());
    assert_ne!(app.task_details.get(&task_uuid).map(String::as_str), Some("sentinel"));
  }

  async fn test_context_menu_enter_closes_menu() {
    reset_test_context();

    let mut app = TaskwarriorTui::new("next", false).await.unwrap();
    assert!(app.update(true).await.is_ok());

    app.handle_input(app.keyconfig.context_menu).await.unwrap();
    assert_eq!(app.mode, Mode::Tasks(Action::ContextMenu));

    let finance_index = app
      .contexts
      .filtered_indices()
      .iter()
      .position(|&ri| app.contexts.rows[ri].name == "finance")
      .unwrap();
    app.contexts.table_state.select(Some(finance_index));

    app.handle_input(KeyCode::Char('\n')).await.unwrap();

    assert_eq!(app.mode, Mode::Tasks(Action::Report));
    assert_eq!(app.current_context_filter, "+finance -private");
  }

  async fn test_context_menu_enter_can_stay_open() {
    reset_test_context();

    let mut app = TaskwarriorTui::new("next", false).await.unwrap();
    assert!(app.update(true).await.is_ok());
    app.config.uda_context_menu_close_on_select = false;

    app.handle_input(app.keyconfig.context_menu).await.unwrap();
    assert_eq!(app.mode, Mode::Tasks(Action::ContextMenu));

    let finance_index = app
      .contexts
      .filtered_indices()
      .iter()
      .position(|&ri| app.contexts.rows[ri].name == "finance")
      .unwrap();
    app.contexts.table_state.select(Some(finance_index));

    app.handle_input(KeyCode::Char('\n')).await.unwrap();

    assert_eq!(app.mode, Mode::Tasks(Action::ContextMenu));
    assert_eq!(app.current_context_filter, "+finance -private");
  }

  async fn test_context_menu_enter_with_no_matches_does_not_select() {
    reset_test_context();

    let mut app = TaskwarriorTui::new("next", false).await.unwrap();
    assert!(app.update(true).await.is_ok());

    app.handle_input(app.keyconfig.context_menu).await.unwrap();
    assert_eq!(app.mode, Mode::Tasks(Action::ContextMenu));

    for c in "zzz-no-match".chars() {
      app.handle_input(KeyCode::Char(c)).await.unwrap();
    }

    assert!(app.contexts.filtered_indices().is_empty());

    app.handle_input(KeyCode::Char('\n')).await.unwrap();

    assert_eq!(app.mode, Mode::Tasks(Action::ContextMenu));
    assert_eq!(app.current_context_filter, "");
  }

  async fn test_context_menu_selection_config_combinations_select_filtered_match_on_enter() {
    for (select_on_move, close_on_select) in [(false, false), (false, true), (true, false), (true, true)] {
      reset_test_context();

      let mut app = TaskwarriorTui::new("next", false).await.unwrap();
      assert!(app.update(true).await.is_ok());
      configure_context_menu_selection(&mut app, select_on_move, close_on_select);

      app.handle_input(app.keyconfig.context_menu).await.unwrap();
      assert_eq!(app.mode, Mode::Tasks(Action::ContextMenu));

      let initial_context_filter = app.current_context_filter.clone();
      for c in "work".chars() {
        app.handle_input(KeyCode::Char(c)).await.unwrap();
      }

      let expected_context_filter = selected_context_filter(&app).unwrap();
      let expected_during_search = if select_on_move {
        expected_context_filter.clone()
      } else {
        initial_context_filter.clone()
      };
      assert_eq!(
        app.current_context_filter, expected_during_search,
        "context combo select_on_move={select_on_move} close_on_select={close_on_select}"
      );

      app.handle_input(KeyCode::Char('\n')).await.unwrap();

      let expected_mode = if close_on_select {
        Mode::Tasks(Action::Report)
      } else {
        Mode::Tasks(Action::ContextMenu)
      };
      assert_eq!(
        app.mode, expected_mode,
        "context combo select_on_move={select_on_move} close_on_select={close_on_select}"
      );
      assert_eq!(
        app.current_context_filter, expected_context_filter,
        "context combo select_on_move={select_on_move} close_on_select={close_on_select}"
      );
    }
  }

  async fn test_context_menu_selection_config_combinations_keep_menu_open_with_no_matches() {
    for (select_on_move, close_on_select) in [(false, false), (false, true), (true, false), (true, true)] {
      reset_test_context();

      let mut app = TaskwarriorTui::new("next", false).await.unwrap();
      assert!(app.update(true).await.is_ok());
      configure_context_menu_selection(&mut app, select_on_move, close_on_select);

      app.handle_input(app.keyconfig.context_menu).await.unwrap();
      assert_eq!(app.mode, Mode::Tasks(Action::ContextMenu));

      let initial_context_filter = app.current_context_filter.clone();
      for c in "zzz-no-match".chars() {
        app.handle_input(KeyCode::Char(c)).await.unwrap();
      }

      assert!(
        app.contexts.filtered_indices().is_empty(),
        "context combo select_on_move={select_on_move} close_on_select={close_on_select}"
      );
      assert_eq!(
        app.current_context_filter, initial_context_filter,
        "context combo select_on_move={select_on_move} close_on_select={close_on_select}"
      );

      app.handle_input(KeyCode::Char('\n')).await.unwrap();

      assert_eq!(
        app.mode,
        Mode::Tasks(Action::ContextMenu),
        "context combo select_on_move={select_on_move} close_on_select={close_on_select}"
      );
      assert_eq!(
        app.current_context_filter, initial_context_filter,
        "context combo select_on_move={select_on_move} close_on_select={close_on_select}"
      );
    }
  }

  async fn test_context_menu_selection_config_combinations_preserve_selection_while_filtering() {
    for (select_on_move, close_on_select) in [(false, false), (false, true), (true, false), (true, true)] {
      reset_test_context();

      let mut app = TaskwarriorTui::new("next", false).await.unwrap();
      assert!(app.update(true).await.is_ok());
      configure_context_menu_selection(&mut app, select_on_move, close_on_select);

      app.handle_input(app.keyconfig.context_menu).await.unwrap();
      assert_eq!(app.mode, Mode::Tasks(Action::ContextMenu));

      let work_index = app
        .contexts
        .filtered_indices()
        .iter()
        .position(|&ri| app.contexts.rows[ri].name == "work")
        .unwrap();
      app.contexts.table_state.select(Some(work_index));

      let initial_context_filter = app.current_context_filter.clone();
      let expected_selected_context_filter = selected_context_filter(&app).unwrap();

      app.handle_input(KeyCode::Char('o')).await.unwrap();

      assert_eq!(
        selected_context_filter(&app).as_deref(),
        Some(expected_selected_context_filter.as_str()),
        "context combo select_on_move={select_on_move} close_on_select={close_on_select}"
      );

      let expected_active_context_filter = if select_on_move {
        expected_selected_context_filter.as_str()
      } else {
        initial_context_filter.as_str()
      };
      assert_eq!(
        app.current_context_filter.as_str(),
        expected_active_context_filter,
        "context combo select_on_move={select_on_move} close_on_select={close_on_select}"
      );
    }
  }

  async fn test_context_menu_selection_config_combinations_move_to_top_when_selection_is_filtered_out() {
    for (select_on_move, close_on_select) in [(false, false), (false, true), (true, false), (true, true)] {
      reset_test_context();

      let mut app = TaskwarriorTui::new("next", false).await.unwrap();
      assert!(app.update(true).await.is_ok());
      configure_context_menu_selection(&mut app, select_on_move, close_on_select);

      app.handle_input(app.keyconfig.context_menu).await.unwrap();
      assert_eq!(app.mode, Mode::Tasks(Action::ContextMenu));

      let finance_index = app
        .contexts
        .filtered_indices()
        .iter()
        .position(|&ri| app.contexts.rows[ri].name == "finance")
        .unwrap();
      app.contexts.table_state.select(Some(finance_index));

      let initial_context_filter = app.current_context_filter.clone();
      for c in "work".chars() {
        app.handle_input(KeyCode::Char(c)).await.unwrap();
      }

      assert_eq!(
        selected_context_filter(&app).as_deref(),
        Some("-personal -private"),
        "context combo select_on_move={select_on_move} close_on_select={close_on_select}"
      );

      let expected_active_context_filter = if select_on_move {
        "-personal -private"
      } else {
        initial_context_filter.as_str()
      };
      assert_eq!(
        app.current_context_filter.as_str(),
        expected_active_context_filter,
        "context combo select_on_move={select_on_move} close_on_select={close_on_select}"
      );
    }
  }

  async fn test_context_menu_type_to_filter_selects_top_match_when_select_on_move_is_enabled() {
    reset_test_context();

    let mut app = TaskwarriorTui::new("next", false).await.unwrap();
    assert!(app.update(true).await.is_ok());
    enable_context_menu_move_selection(&mut app);

    app.handle_input(app.keyconfig.context_menu).await.unwrap();
    assert_eq!(app.mode, Mode::Tasks(Action::ContextMenu));

    for c in "finance".chars() {
      app.handle_input(KeyCode::Char(c)).await.unwrap();
    }

    assert_eq!(app.mode, Mode::Tasks(Action::ContextMenu));
    assert_eq!(app.current_context_filter, selected_context_filter(&app).unwrap());
  }

  async fn test_context_menu_backspace_selects_top_match_and_stays_open_when_select_on_move_is_enabled() {
    reset_test_context();

    let mut app = TaskwarriorTui::new("next", false).await.unwrap();
    assert!(app.update(true).await.is_ok());
    enable_context_menu_move_selection(&mut app);

    app.handle_input(app.keyconfig.context_menu).await.unwrap();
    assert_eq!(app.mode, Mode::Tasks(Action::ContextMenu));

    for c in "finance".chars() {
      app.handle_input(KeyCode::Char(c)).await.unwrap();
    }
    app.handle_input(KeyCode::Backspace).await.unwrap();

    assert_eq!(app.mode, Mode::Tasks(Action::ContextMenu));
    assert_eq!(app.current_context_filter, selected_context_filter(&app).unwrap());
  }

  async fn test_context_menu_escape_clears_search_selects_top_match_and_stays_open_when_select_on_move_is_enabled() {
    reset_test_context();

    let mut app = TaskwarriorTui::new("next", false).await.unwrap();
    assert!(app.update(true).await.is_ok());
    enable_context_menu_move_selection(&mut app);

    app.handle_input(app.keyconfig.context_menu).await.unwrap();
    assert_eq!(app.mode, Mode::Tasks(Action::ContextMenu));

    for c in "finance".chars() {
      app.handle_input(KeyCode::Char(c)).await.unwrap();
    }
    app.handle_input(KeyCode::Esc).await.unwrap();

    assert_eq!(app.mode, Mode::Tasks(Action::ContextMenu));
    assert!(app.contexts.search.is_empty());
    assert_eq!(app.current_context_filter, selected_context_filter(&app).unwrap());
  }

  async fn test_context_menu_enter_with_no_matches_stays_open_when_select_on_move_is_enabled() {
    reset_test_context();

    let mut app = TaskwarriorTui::new("next", false).await.unwrap();
    assert!(app.update(true).await.is_ok());
    enable_context_menu_move_selection(&mut app);

    app.handle_input(app.keyconfig.context_menu).await.unwrap();
    assert_eq!(app.mode, Mode::Tasks(Action::ContextMenu));

    let initial_context_filter = app.current_context_filter.clone();
    for c in "zzz-no-match".chars() {
      app.handle_input(KeyCode::Char(c)).await.unwrap();
    }

    assert!(app.contexts.filtered_indices().is_empty());
    assert_eq!(app.current_context_filter, initial_context_filter);

    app.handle_input(KeyCode::Char('\n')).await.unwrap();

    assert_eq!(app.mode, Mode::Tasks(Action::ContextMenu));
    assert_eq!(app.current_context_filter, initial_context_filter);
  }

  async fn test_context_menu_enter_selects_filtered_match_when_select_on_move_is_enabled() {
    reset_test_context();

    let mut app = TaskwarriorTui::new("next", false).await.unwrap();
    assert!(app.update(true).await.is_ok());
    enable_context_menu_move_selection(&mut app);

    app.handle_input(app.keyconfig.context_menu).await.unwrap();
    assert_eq!(app.mode, Mode::Tasks(Action::ContextMenu));

    for c in "finance".chars() {
      app.handle_input(KeyCode::Char(c)).await.unwrap();
    }

    let expected_context_filter = selected_context_filter(&app).unwrap();

    app.handle_input(KeyCode::Char('\n')).await.unwrap();

    assert_eq!(app.mode, Mode::Tasks(Action::Report));
    assert_eq!(app.current_context_filter, expected_context_filter);
  }
  async fn test_report_menu_enter_closes_menu() {
    let mut app = TaskwarriorTui::new("next", false).await.unwrap();
    assert!(app.update(true).await.is_ok());

    app.handle_input(app.keyconfig.report_menu).await.unwrap();
    assert_eq!(app.mode, Mode::Tasks(Action::ReportMenu));

    let day_index = app.reports.rows.iter().position(|row| row.name == "day").unwrap();
    app.reports.table_state.select(Some(day_index));

    app.handle_input(KeyCode::Char('\n')).await.unwrap();

    assert_eq!(app.mode, Mode::Tasks(Action::Report));
    assert_eq!(app.report, "day");
    assert_eq!(app.config.filter.trim(), "status:pending");
  }

  async fn test_report_menu_enter_can_stay_open() {
    let mut app = TaskwarriorTui::new("next", false).await.unwrap();
    assert!(app.update(true).await.is_ok());
    app.config.uda_report_menu_close_on_select = false;

    app.handle_input(app.keyconfig.report_menu).await.unwrap();
    assert_eq!(app.mode, Mode::Tasks(Action::ReportMenu));

    let day_index = app.reports.rows.iter().position(|row| row.name == "day").unwrap();
    app.reports.table_state.select(Some(day_index));

    app.handle_input(KeyCode::Char('\n')).await.unwrap();

    assert_eq!(app.mode, Mode::Tasks(Action::ReportMenu));
    assert_eq!(app.report, "day");
    assert_eq!(app.config.filter.trim(), "status:pending");
  }

  async fn test_report_menu_enter_with_no_matches_does_not_select() {
    let mut app = TaskwarriorTui::new("next", false).await.unwrap();
    assert!(app.update(true).await.is_ok());

    app.handle_input(app.keyconfig.report_menu).await.unwrap();
    assert_eq!(app.mode, Mode::Tasks(Action::ReportMenu));

    for c in "zzz-no-match".chars() {
      app.handle_input(KeyCode::Char(c)).await.unwrap();
    }

    assert!(app.reports.filtered_indices().is_empty());

    app.handle_input(KeyCode::Char('\n')).await.unwrap();

    assert_eq!(app.mode, Mode::Tasks(Action::ReportMenu));
    assert_eq!(app.report, "next");
    assert_eq!(app.config.filter.trim(), "(status:pending or status:waiting)");
  }

  async fn test_report_menu_selection_config_combinations_select_filtered_match_on_enter() {
    for (select_on_move, close_on_select) in [(false, false), (false, true), (true, false), (true, true)] {
      let mut app = TaskwarriorTui::new("next", false).await.unwrap();
      assert!(app.update(true).await.is_ok());
      configure_report_menu_selection(&mut app, select_on_move, close_on_select);

      app.handle_input(app.keyconfig.report_menu).await.unwrap();
      assert_eq!(app.mode, Mode::Tasks(Action::ReportMenu));

      let initial_report = app.report.clone();
      for c in "day".chars() {
        app.handle_input(KeyCode::Char(c)).await.unwrap();
      }

      let expected_report = selected_report_name(&app).unwrap();
      let expected_during_search = if select_on_move {
        expected_report.clone()
      } else {
        initial_report.clone()
      };
      assert_eq!(
        app.report, expected_during_search,
        "report combo select_on_move={select_on_move} close_on_select={close_on_select}"
      );

      app.handle_input(KeyCode::Char('\n')).await.unwrap();

      let expected_mode = if close_on_select {
        Mode::Tasks(Action::Report)
      } else {
        Mode::Tasks(Action::ReportMenu)
      };
      assert_eq!(
        app.mode, expected_mode,
        "report combo select_on_move={select_on_move} close_on_select={close_on_select}"
      );
      assert_eq!(
        app.report, expected_report,
        "report combo select_on_move={select_on_move} close_on_select={close_on_select}"
      );
    }
  }

  async fn test_report_menu_selection_config_combinations_keep_menu_open_with_no_matches() {
    for (select_on_move, close_on_select) in [(false, false), (false, true), (true, false), (true, true)] {
      let mut app = TaskwarriorTui::new("next", false).await.unwrap();
      assert!(app.update(true).await.is_ok());
      configure_report_menu_selection(&mut app, select_on_move, close_on_select);

      app.handle_input(app.keyconfig.report_menu).await.unwrap();
      assert_eq!(app.mode, Mode::Tasks(Action::ReportMenu));

      let initial_report = app.report.clone();
      for c in "zzz-no-match".chars() {
        app.handle_input(KeyCode::Char(c)).await.unwrap();
      }

      assert!(
        app.reports.filtered_indices().is_empty(),
        "report combo select_on_move={select_on_move} close_on_select={close_on_select}"
      );
      assert_eq!(
        app.report, initial_report,
        "report combo select_on_move={select_on_move} close_on_select={close_on_select}"
      );

      app.handle_input(KeyCode::Char('\n')).await.unwrap();

      assert_eq!(
        app.mode,
        Mode::Tasks(Action::ReportMenu),
        "report combo select_on_move={select_on_move} close_on_select={close_on_select}"
      );
      assert_eq!(
        app.report, initial_report,
        "report combo select_on_move={select_on_move} close_on_select={close_on_select}"
      );
    }
  }

  async fn test_report_menu_selection_config_combinations_preserve_selection_while_filtering() {
    for (select_on_move, close_on_select) in [(false, false), (false, true), (true, false), (true, true)] {
      let mut app = TaskwarriorTui::new("next", false).await.unwrap();
      assert!(app.update(true).await.is_ok());
      configure_report_menu_selection(&mut app, select_on_move, close_on_select);

      app.handle_input(app.keyconfig.report_menu).await.unwrap();
      assert_eq!(app.mode, Mode::Tasks(Action::ReportMenu));

      let recurring_index = app.reports.rows.iter().position(|row| row.name == "recurring").unwrap();
      app.reports.table_state.select(Some(recurring_index));

      let initial_report = app.report.clone();
      let expected_selected_report = selected_report_name(&app).unwrap();

      app.handle_input(KeyCode::Char('r')).await.unwrap();

      assert_eq!(
        selected_report_name(&app).as_deref(),
        Some(expected_selected_report.as_str()),
        "report combo select_on_move={select_on_move} close_on_select={close_on_select}"
      );

      let expected_active_report = if select_on_move {
        expected_selected_report.as_str()
      } else {
        initial_report.as_str()
      };
      assert_eq!(
        app.report.as_str(),
        expected_active_report,
        "report combo select_on_move={select_on_move} close_on_select={close_on_select}"
      );
    }
  }

  async fn test_report_menu_selection_config_combinations_move_to_top_when_selection_is_filtered_out() {
    for (select_on_move, close_on_select) in [(false, false), (false, true), (true, false), (true, true)] {
      let mut app = TaskwarriorTui::new("next", false).await.unwrap();
      assert!(app.update(true).await.is_ok());
      configure_report_menu_selection(&mut app, select_on_move, close_on_select);

      app.handle_input(app.keyconfig.report_menu).await.unwrap();
      assert_eq!(app.mode, Mode::Tasks(Action::ReportMenu));

      let day_index = app.reports.rows.iter().position(|row| row.name == "day").unwrap();
      app.reports.table_state.select(Some(day_index));

      let initial_report = app.report.clone();
      for c in "re".chars() {
        app.handle_input(KeyCode::Char(c)).await.unwrap();
      }

      assert_eq!(
        selected_report_name(&app).as_deref(),
        Some("ready"),
        "report combo select_on_move={select_on_move} close_on_select={close_on_select}"
      );

      let expected_active_report = if select_on_move { "ready" } else { initial_report.as_str() };
      assert_eq!(
        app.report.as_str(),
        expected_active_report,
        "report combo select_on_move={select_on_move} close_on_select={close_on_select}"
      );
    }
  }

  async fn test_report_menu_type_to_filter_selects_top_match_when_select_on_move_is_enabled() {
    let mut app = TaskwarriorTui::new("next", false).await.unwrap();
    assert!(app.update(true).await.is_ok());
    enable_report_menu_move_selection(&mut app);

    app.handle_input(app.keyconfig.report_menu).await.unwrap();
    assert_eq!(app.mode, Mode::Tasks(Action::ReportMenu));

    for c in "day".chars() {
      app.handle_input(KeyCode::Char(c)).await.unwrap();
    }

    assert_eq!(app.mode, Mode::Tasks(Action::ReportMenu));
    assert_eq!(app.report, selected_report_name(&app).unwrap());
  }

  async fn test_report_menu_backspace_selects_top_match_and_stays_open_when_select_on_move_is_enabled() {
    let mut app = TaskwarriorTui::new("next", false).await.unwrap();
    assert!(app.update(true).await.is_ok());
    enable_report_menu_move_selection(&mut app);

    app.handle_input(app.keyconfig.report_menu).await.unwrap();
    assert_eq!(app.mode, Mode::Tasks(Action::ReportMenu));

    for c in "recurring".chars() {
      app.handle_input(KeyCode::Char(c)).await.unwrap();
    }
    for _ in 0..7 {
      app.handle_input(KeyCode::Ctrl('h')).await.unwrap();
    }

    assert_eq!(app.mode, Mode::Tasks(Action::ReportMenu));
    assert_eq!(app.reports.search, "re");
    assert_eq!(app.report, selected_report_name(&app).unwrap());
  }

  async fn test_report_menu_escape_clears_search_selects_top_match_and_stays_open_when_select_on_move_is_enabled() {
    let mut app = TaskwarriorTui::new("next", false).await.unwrap();
    assert!(app.update(true).await.is_ok());
    enable_report_menu_move_selection(&mut app);

    app.handle_input(app.keyconfig.report_menu).await.unwrap();
    assert_eq!(app.mode, Mode::Tasks(Action::ReportMenu));

    for c in "day".chars() {
      app.handle_input(KeyCode::Char(c)).await.unwrap();
    }
    app.handle_input(KeyCode::Esc).await.unwrap();

    assert_eq!(app.mode, Mode::Tasks(Action::ReportMenu));
    assert!(app.reports.search.is_empty());
    assert_eq!(app.report, selected_report_name(&app).unwrap());
  }

  async fn test_report_menu_enter_with_no_matches_stays_open_when_select_on_move_is_enabled() {
    let mut app = TaskwarriorTui::new("next", false).await.unwrap();
    assert!(app.update(true).await.is_ok());
    enable_report_menu_move_selection(&mut app);

    app.handle_input(app.keyconfig.report_menu).await.unwrap();
    assert_eq!(app.mode, Mode::Tasks(Action::ReportMenu));

    let initial_report = app.report.clone();
    for c in "zzz-no-match".chars() {
      app.handle_input(KeyCode::Char(c)).await.unwrap();
    }

    assert!(app.reports.filtered_indices().is_empty());
    assert_eq!(app.report, initial_report);

    app.handle_input(KeyCode::Char('\n')).await.unwrap();

    assert_eq!(app.mode, Mode::Tasks(Action::ReportMenu));
    assert_eq!(app.report, initial_report);
  }

  async fn test_report_menu_enter_selects_filtered_match_when_select_on_move_is_enabled() {
    let mut app = TaskwarriorTui::new("next", false).await.unwrap();
    assert!(app.update(true).await.is_ok());
    enable_report_menu_move_selection(&mut app);

    app.handle_input(app.keyconfig.report_menu).await.unwrap();
    assert_eq!(app.mode, Mode::Tasks(Action::ReportMenu));

    for c in "day".chars() {
      app.handle_input(KeyCode::Char(c)).await.unwrap();
    }

    let expected_report = selected_report_name(&app).unwrap();

    app.handle_input(KeyCode::Char('\n')).await.unwrap();

    assert_eq!(app.mode, Mode::Tasks(Action::Report));
    assert_eq!(app.report, expected_report);
  }

  async fn test_task_context() {
    reset_test_context();

    let mut app = TaskwarriorTui::new("next", false).await.unwrap();

    assert!(app.update(true).await.is_ok());

    app.context_select().unwrap();

    assert_eq!(app.tasks.len(), 26);
    assert_eq!(app.current_context_filter, "");

    assert_eq!(app.contexts.table_state.current_selection(), Some(0));
    app.context_next();
    app.context_next();
    app.context_select().unwrap();
    assert_eq!(app.contexts.table_state.current_selection(), Some(2));

    assert!(app.update(true).await.is_ok());

    assert_eq!(app.tasks.len(), 1);
    assert_eq!(app.current_context_filter, "+finance -private");

    assert_eq!(app.contexts.table_state.current_selection(), Some(2));
    app.context_previous();
    app.context_previous();
    app.context_select().unwrap();
    assert_eq!(app.contexts.table_state.current_selection(), Some(0));

    assert!(app.update(true).await.is_ok());

    assert_eq!(app.tasks.len(), 26);
    assert_eq!(app.current_context_filter, "");
  }

  async fn test_task_tomorrow() {
    let total_tasks: u64 = 26;

    let mut app = TaskwarriorTui::new("next", false).await.unwrap();
    assert!(app.update(true).await.is_ok());
    assert_eq!(app.tasks.len(), total_tasks as usize);
    assert_eq!(app.current_context_filter, "");

    let now = Local::now();
    let now = TimeZone::from_utc_datetime(now.offset(), &now.naive_utc());

    let mut command = std::process::Command::new(&task_exe());
    command.arg("add");
    let tomorrow = now + chrono::Duration::days(1);
    let message = format!(
      "'new task for testing tomorrow' due:{:04}-{:02}-{:02}",
      tomorrow.year(),
      tomorrow.month(),
      tomorrow.day(),
    );

    let shell = message.as_str().replace("'", "\\'");
    let cmd = shlex::split(&shell).unwrap();
    for s in cmd {
      command.arg(&s);
    }
    let output = command.output().unwrap();
    let s = String::from_utf8_lossy(&output.stdout);
    let re = Regex::new(r"^Created task (?P<task_id>\d+).\n$").unwrap();
    let caps = re.captures(&s).unwrap();

    let task_id = caps["task_id"].parse::<u64>().unwrap();
    assert_eq!(task_id, total_tasks + 1);

    assert!(app.update(true).await.is_ok());
    assert_eq!(app.tasks.len(), (total_tasks + 1) as usize);
    assert_eq!(app.current_context_filter, "");

    let task = app.task_by_id(task_id).unwrap();
    assert_task_has_tags(&task, &["DUE", "MONTH", "PENDING", "QUARTER", "TOMORROW", "UDA", "UNBLOCKED", "YEAR"]);

    let output = std::process::Command::new(&task_exe())
      .arg("rc.confirmation=off")
      .arg("undo")
      .output()
      .unwrap();

    let mut app = TaskwarriorTui::new("next", false).await.unwrap();
    assert!(app.update(true).await.is_ok());
    assert_eq!(app.tasks.len(), total_tasks as usize);
    assert_eq!(app.current_context_filter, "");
  }

  async fn test_task_earlier_today() {
    let total_tasks: u64 = 26;

    let mut app = TaskwarriorTui::new("next", false).await.unwrap();
    assert!(app.update(true).await.is_ok());
    assert_eq!(app.tasks.len(), total_tasks as usize);
    assert_eq!(app.current_context_filter, "");

    let now = Local::now();
    let now = TimeZone::from_utc_datetime(now.offset(), &now.naive_utc());

    let mut command = std::process::Command::new(&task_exe());
    command.arg("add");
    let message = "'new task for testing earlier today' due:now";

    let shell = message.replace("'", "\\'");
    let cmd = shlex::split(&shell).unwrap();
    for s in cmd {
      command.arg(&s);
    }
    let output = command.output().unwrap();
    let s = String::from_utf8_lossy(&output.stdout);
    let re = Regex::new(r"^Created task (?P<task_id>\d+).\n$").unwrap();
    let caps = re.captures(&s).unwrap();
    let task_id = caps["task_id"].parse::<u64>().unwrap();
    assert_eq!(task_id, total_tasks + 1);

    assert!(app.update(true).await.is_ok());
    assert_eq!(app.tasks.len(), (total_tasks + 1) as usize);
    assert_eq!(app.current_context_filter, "");

    let task = app.task_by_id(task_id).unwrap();
    assert_task_has_tags(
      &task,
      &[
        "DUE",
        "DUETODAY",
        "MONTH",
        "OVERDUE",
        "PENDING",
        "QUARTER",
        "TODAY",
        "UDA",
        "UNBLOCKED",
        "YEAR",
      ],
    );

    let output = std::process::Command::new(&task_exe())
      .arg("rc.confirmation=off")
      .arg("undo")
      .output()
      .unwrap();

    let mut app = TaskwarriorTui::new("next", false).await.unwrap();
    assert!(app.update(true).await.is_ok());
    assert_eq!(app.tasks.len(), total_tasks as usize);
    assert_eq!(app.current_context_filter, "");
  }

  async fn test_task_later_today() {
    let total_tasks: u64 = 26;

    let mut app = TaskwarriorTui::new("next", false).await.unwrap();
    assert!(app.update(true).await.is_ok());
    assert_eq!(app.tasks.len(), total_tasks as usize);
    assert_eq!(app.current_context_filter, "");

    let now = Local::now();
    let now = TimeZone::from_utc_datetime(now.offset(), &now.naive_utc());

    let mut command = std::process::Command::new(&task_exe());
    command.arg("add");
    let later_today = now
      .with_hour(23)
      .and_then(|dt| dt.with_minute(59))
      .and_then(|dt| dt.with_second(59))
      .unwrap();
    assert!(later_today > now, "cannot exercise the later-today path at the end of the local day");
    let message = format!(
      "'new task for testing later today' due:'{:04}-{:02}-{:02}T{:02}:{:02}:{:02}'",
      later_today.year(),
      later_today.month(),
      later_today.day(),
      later_today.hour(),
      later_today.minute(),
      later_today.second(),
    );

    let shell = message.as_str().replace("'", "\\'");
    let cmd = shlex::split(&shell).unwrap();
    for s in cmd {
      command.arg(&s);
    }
    let output = command.output().unwrap();
    let s = String::from_utf8_lossy(&output.stdout);
    let re = Regex::new(r"^Created task (?P<task_id>\d+).\n$").unwrap();
    let caps = re.captures(&s).unwrap();
    let task_id = caps["task_id"].parse::<u64>().unwrap();
    assert_eq!(task_id, total_tasks + 1);

    assert!(app.update(true).await.is_ok());
    assert_eq!(app.tasks.len(), (total_tasks + 1) as usize);
    assert_eq!(app.current_context_filter, "");

    let task = app.task_by_id(task_id).unwrap();
    assert_task_has_tags(
      &task,
      &["DUE", "DUETODAY", "MONTH", "PENDING", "QUARTER", "TODAY", "UDA", "UNBLOCKED", "YEAR"],
    );

    let output = std::process::Command::new(&task_exe())
      .arg("rc.confirmation=off")
      .arg("undo")
      .output()
      .unwrap();

    let mut app = TaskwarriorTui::new("next", false).await.unwrap();
    assert!(app.update(true).await.is_ok());
    assert_eq!(app.tasks.len(), total_tasks as usize);
    assert_eq!(app.current_context_filter, "");
  }

  async fn test_draw_empty_task_report() {
    let mut expected = Buffer::with_lines(vec![
      " Tasks   Projects   Calendar           next [none]",
      "                                                  ",
      "                                                  ",
      "                                                  ",
      "                                                  ",
      "                                                  ",
      "                                                  ",
      "──────────────────────────────────────────────────",
      "Task not found                                    ",
      "                                                  ",
      "                                                  ",
      "                                                  ",
      "                                                  ",
      "Filter Tasks                                      ",
      "(status:pending or status:waiting)                ",
    ]);

    for i in 0..=49 {
      // First line
      expected[(i, 0)].set_style(Style::default().add_modifier(Modifier::REVERSED));
    }
    for i in 1..=5 {
      // Tasks
      expected[(i, 0)].set_style(Style::default().add_modifier(Modifier::BOLD).add_modifier(Modifier::REVERSED));
    }
    for i in 0..=49 {
      // Command line
      expected[(i, 13)].set_style(Style::default().add_modifier(Modifier::REVERSED));
    }

    let mut app = TaskwarriorTui::new("next", false).await.unwrap();
    app.tasklist_vertical = true;

    app.task_report_next();
    app.context_next();

    let total_tasks: u64 = 0;

    assert!(app.update(true).await.is_ok());
    assert_eq!(app.tasks.len(), total_tasks as usize);
    assert_eq!(app.current_context_filter, "");

    let now = Local::now();
    let now = TimeZone::from_utc_datetime(now.offset(), &now.naive_utc());

    app.update(true).await.unwrap();

    let backend = TestBackend::new(50, 15);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
      .draw(|f| {
        app.draw(f);
      })
      .unwrap();

    assert_eq!(terminal.backend().size().unwrap(), expected.area.into());
    terminal.backend().assert_buffer(&expected);
  }

  async fn test_draw_task_report_with_extended_modify_command() {
    let mut expected1 = Buffer::with_lines(vec![
      "Modify Task 10           ",
      " based on your .taskrc   ",
      "                         ",
    ]);

    let mut expected2 = Buffer::with_lines(vec![
      "Modify Task 10           ",
      "Support color for tasks b",
      "                         ",
    ]);

    for i in 0..=13 {
      // Task
      expected1[(i, 0)].set_style(Style::default().add_modifier(Modifier::BOLD));
      expected2[(i, 0)].set_style(Style::default().add_modifier(Modifier::BOLD));
    }
    for i in 0..=24 {
      // Command line
      expected1[(i, 0)].set_style(Style::default().add_modifier(Modifier::REVERSED));
      expected2[(i, 0)].set_style(Style::default().add_modifier(Modifier::REVERSED));
    }

    let mut app = TaskwarriorTui::new("next", false).await.unwrap();

    let total_tasks: u64 = 26;

    assert!(app.update(true).await.is_ok());
    assert_eq!(app.tasks.len(), total_tasks as usize);
    assert_eq!(app.current_context_filter, "");

    let now = Local::now();
    let now = TimeZone::from_utc_datetime(now.offset(), &now.naive_utc());

    app.mode = Mode::Tasks(Action::Modify);
    match app.task_table_state.mode() {
      TableMode::SingleSelection => match app.task_current() {
        Some(t) => {
          let s = format!("{} ", t.description());
          app.modify.update(&s, s.as_str().len(), &mut app.changes)
        }
        None => app.modify.update("", 0, &mut app.changes),
      },
      TableMode::MultipleSelection => app.modify.update("", 0, &mut app.changes),
    }

    app.update(true).await.unwrap();

    let backend = TestBackend::new(25, 3);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
      .draw(|f| {
        let rects = Layout::default()
          .direction(Direction::Vertical)
          .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
          .split(f.area());

        let taskwarrior_position = TaskwarriorTui::get_position(&app.modify);
        let position = Position::new(
          std::cmp::min(rects[1].x + taskwarrior_position as u16, rects[1].x + rects[1].width.saturating_sub(2)),
          rects[1].y + 1,
        );
        f.set_cursor_position(position);
        f.render_widget(Clear, rects[1]);
        let selected = app.current_selection;
        let task_ids = if app.tasks.is_empty() {
          vec!["0".to_string()]
        } else {
          match app.task_table_state.mode() {
            TableMode::SingleSelection => {
              vec![app.tasks[selected].id().unwrap_or_default().to_string()]
            }
            TableMode::MultipleSelection => {
              let mut tids = vec![];
              for uuid in app.marked.iter() {
                if let Some(t) = app.task_by_uuid(*uuid) {
                  tids.push(t.id().unwrap_or_default().to_string());
                }
              }
              tids
            }
          }
        };
        let label = if task_ids.len() > 1 {
          format!("Modify Tasks {}", task_ids.join(","))
        } else {
          format!("Modify Task {}", task_ids.join(","))
        };
        app.draw_command(
          f,
          rects[1],
          app.modify.as_str(),
          (Span::styled(label, Style::default().add_modifier(Modifier::BOLD)), None),
          taskwarrior_position,
          true,
          app.error.clone(),
          None,
        );
      })
      .unwrap();

    assert_eq!(terminal.backend().size().unwrap(), expected1.area.into());
    terminal.backend().assert_buffer(&expected1);

    app.modify.move_home();

    terminal
      .draw(|f| {
        let rects = Layout::default()
          .direction(Direction::Vertical)
          .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
          .split(f.area());

        let taskwarrior_position = TaskwarriorTui::get_position(&app.modify);
        let position = Position::new(
          std::cmp::min(rects[1].x + taskwarrior_position as u16, rects[1].x + rects[1].width.saturating_sub(2)),
          rects[1].y + 1,
        );
        f.set_cursor_position(position);
        f.render_widget(Clear, rects[1]);
        let selected = app.current_selection;
        let task_ids = if app.tasks.is_empty() {
          vec!["0".to_string()]
        } else {
          match app.task_table_state.mode() {
            TableMode::SingleSelection => {
              vec![app.tasks[selected].id().unwrap_or_default().to_string()]
            }
            TableMode::MultipleSelection => {
              let mut tids = vec![];
              for uuid in app.marked.iter() {
                if let Some(t) = app.task_by_uuid(*uuid) {
                  tids.push(t.id().unwrap_or_default().to_string());
                }
              }
              tids
            }
          }
        };
        let label = if task_ids.len() > 1 {
          format!("Modify Tasks {}", task_ids.join(","))
        } else {
          format!("Modify Task {}", task_ids.join(","))
        };
        app.draw_command(
          f,
          rects[1],
          app.modify.as_str(),
          (Span::styled(label, Style::default().add_modifier(Modifier::BOLD)), None),
          taskwarrior_position,
          true,
          app.error.clone(),
          None,
        );
      })
      .unwrap();

    assert_eq!(terminal.backend().size().unwrap(), expected2.area.into());
    terminal.backend().assert_buffer(&expected2);
  }

  async fn test_draw_task_report() {
    let mut expected = Buffer::with_lines(vec![
      "╭Task|Calendar───────────────────────────────────╮",
      "│  ID Age Deps P Projec Tag     Due Descrip Urg  │",
      "│                                                │",
      "│• 27 0s       U                    new ta… 15.00│",
      "│  28 0s       U        none        new ta… 15.00│",
      "╰────────────────────────────────────────────────╯",
      "╭Task 27─────────────────────────────────────────╮",
      "│                                                │",
      "│Name          Value                             │",
      "│------------- ----------------------------------│",
      "│ID            27                                │",
      "╰────────────────────────────────────────────────╯",
      "╭Filter Tasks────────────────────────────────────╮",
      "│(status:pending or status:waiting)              │",
      "╰────────────────────────────────────────────────╯",
    ]);

    for i in 1..=4 {
      // Task
      expected[(i, 0)].set_style(Style::default().add_modifier(Modifier::BOLD));
    }
    for i in 6..=13 {
      // Calendar
      expected[(i, 0)].set_style(Style::default().add_modifier(Modifier::DIM));
    }

    for r in &[
      1..=4,   // ID
      6..=8,   // Age
      10..=13, // Deps
      15..=15, // P
      17..=22, // Projec
      24..=30, // Tag
      32..=34, // Due
      36..=42, // Descr
      44..=48, // Urg
    ] {
      for i in r.clone() {
        expected[(i, 1)].set_style(Style::default().add_modifier(Modifier::UNDERLINED));
      }
    }

    for i in 1..expected.area().width - 1 {
      expected[(i, 3)].set_style(Style::default().fg(Color::Indexed(1)).bg(Color::Reset).add_modifier(Modifier::BOLD));
    }

    for i in 1..expected.area().width - 1 {
      expected[(i, 4)].set_style(Style::default().fg(Color::Indexed(1)).bg(Color::Indexed(4)));
    }

    let mut app = TaskwarriorTui::new("next", false).await.unwrap();
    app.tasklist_vertical = true;

    app.task_report_next();
    app.context_next();

    let total_tasks: u64 = 26;

    assert!(app.update(true).await.is_ok());
    assert_eq!(app.tasks.len(), total_tasks as usize);
    assert_eq!(app.current_context_filter, "");

    let now = Local::now();
    let now = TimeZone::from_utc_datetime(now.offset(), &now.naive_utc());

    let mut command = std::process::Command::new(&task_exe());
    command.arg("add");
    let message = "'new task 1 for testing draw' priority:U";

    let shell = message.replace("'", "\\'");
    let cmd = shlex::split(&shell).unwrap();
    for s in cmd {
      command.arg(&s);
    }
    let output = command.output().unwrap();
    let s = String::from_utf8_lossy(&output.stdout);
    let re = Regex::new(r"^Created task (?P<task_id>\d+).\n$").unwrap();
    let caps = re.captures(&s).unwrap();
    let task_id = caps["task_id"].parse::<u64>().unwrap();
    assert_eq!(task_id, total_tasks + 1);

    let mut command = std::process::Command::new(&task_exe());
    command.arg("add");
    let message = "'new task 2 for testing draw' priority:U +none";

    let shell = message.replace("'", "\\'");
    let cmd = shlex::split(&shell).unwrap();
    for s in cmd {
      command.arg(&s);
    }
    let output = command.output().unwrap();
    let s = String::from_utf8_lossy(&output.stdout);
    let re = Regex::new(r"^Created task (?P<task_id>\d+).\n$").unwrap();
    let caps = re.captures(&s).unwrap();
    let task_id = caps["task_id"].parse::<u64>().unwrap();
    assert_eq!(task_id, total_tasks + 2);

    app.task_report_next();
    app.task_report_previous();
    app.task_report_next_page();
    app.task_report_previous_page();
    app.task_report_bottom();
    app.task_report_top();
    app.update(true).await.unwrap();

    let backend = TestBackend::new(50, 15);
    let mut terminal = Terminal::new(backend).unwrap();
    app.task_report_show_info = !app.task_report_show_info;
    terminal
      .draw(|f| {
        app.draw(f);
        app.draw(f);
      })
      .unwrap();
    app.task_report_show_info = !app.task_report_show_info;
    terminal
      .draw(|f| {
        app.draw(f);
        app.draw(f);
      })
      .unwrap();

    let output = std::process::Command::new(&task_exe())
      .arg("rc.confirmation=off")
      .arg("undo")
      .output()
      .unwrap();
    let output = std::process::Command::new(&task_exe())
      .arg("rc.confirmation=off")
      .arg("undo")
      .output()
      .unwrap();

    assert_eq!(terminal.backend().size().unwrap(), expected.area.into());
    terminal.backend().assert_buffer(&expected);
  }

  async fn test_draw_calendar() {
    let mut expected = Buffer::with_lines(vec![
      " Tasks   Projects   Calendar           next [none]",
      "                                                  ",
      "                       2020                       ",
      "                                                  ",
      "         January               February           ",
      "   Su Mo Tu We Th Fr Sa  Su Mo Tu We Th Fr Sa     ",
      "             1  2  3  4                     1     ",
      "    5  6  7  8  9 10 11   2  3  4  5  6  7  8     ",
      "   12 13 14 15 16 17 18   9 10 11 12 13 14 15     ",
      "   19 20 21 22 23 24 25  16 17 18 19 20 21 22     ",
      "   26 27 28 29 30 31     23 24 25 26 27 28 29     ",
      "                                                  ",
      "                                                  ",
      "                                                  ",
      "                                                  ",
    ]);

    for i in 0..=49 {
      // First line
      expected[(i, 0)].set_style(Style::default().add_modifier(Modifier::REVERSED));
    }
    for i in 20..=27 {
      // Calendar
      expected[(i, 0)].set_style(Style::default().add_modifier(Modifier::BOLD).add_modifier(Modifier::REVERSED));
    }

    for i in 0..=49 {
      expected[(i, 2)].set_style(Style::default().add_modifier(Modifier::UNDERLINED));
    }

    for i in 3..=22 {
      expected[(i, 4)].set_style(Style::default().bg(Color::Reset));
    }

    for i in 25..=44 {
      expected[(i, 4)].set_style(Style::default().bg(Color::Reset));
    }

    for i in 3..=22 {
      expected[(i, 5)].set_style(Style::default().bg(Color::Reset).add_modifier(Modifier::UNDERLINED));
    }

    for i in 25..=44 {
      expected[(i, 5)].set_style(Style::default().bg(Color::Reset).add_modifier(Modifier::UNDERLINED));
    }

    let mut app = TaskwarriorTui::new("next", false).await.unwrap();

    app.task_report_next();
    app.context_next();
    app.update(true).await.unwrap();

    app.calendar_year = 2020;
    app.mode = Mode::Calendar;

    app.update(true).await.unwrap();

    let backend = TestBackend::new(50, 15);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
      .draw(|f| {
        app.draw(f);
        app.draw(f);
      })
      .unwrap();

    assert_eq!(terminal.backend().size().unwrap(), expected.area.into());
    terminal.backend().assert_buffer(&expected);
  }

  async fn test_draw_help_popup() {
    let mut expected = Buffer::with_lines(vec![
      "╭Help──────────────────────────────────╮",
      "│# Keybindings                         │",
      "│                                      │",
      "│Keybindings:                          │",
      "│                                      │",
      "│    Esc:                              │",
      "│                                      │",
      "│    ]: Next view                      │",
      "│                                      │",
      "│    [: Previous view                  │",
      "╰──────────────────────────────────────╯",
      "  7% ───────────────────────────────────",
    ]);

    for i in 1..=4 {
      // Calendar
      expected[(i, 0)].set_style(Style::default().add_modifier(Modifier::BOLD));
    }
    expected[(3, 11)].set_style(Style::default().fg(Color::Reset));
    expected[(4, 11)].set_style(Style::default().fg(Color::Reset));
    expected[(5, 11)].set_style(Style::default().fg(Color::Gray));
    expected[(6, 11)].set_style(Style::default().fg(Color::Gray));

    let mut app = TaskwarriorTui::new("next", false).await.unwrap();

    app.mode = Mode::Tasks(Action::HelpPopup);
    app.task_report_next();
    app.context_next();
    app.update(true).await.unwrap();

    let backend = TestBackend::new(40, 12);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
      .draw(|f| {
        app.draw_help_popup(f, 100, 100);
      })
      .unwrap();

    assert_eq!(terminal.backend().size().unwrap(), expected.area.into());
    terminal.backend().assert_buffer(&expected);
  }

  // #[test]
  async fn test_draw_context_menu() {
    let mut expected = Buffer::with_lines(vec![
      "╭Context───────────────────────────────────────────────────────────────────────╮",
      "│Name       Description                                                  Active│",
      "│                                                                              │",
      "│• none                                                                  yes   │",
      "│  finance  +finance -private                                            no    │",
      "│  personal +personal -private                                           no    │",
      "│  work     -personal -private                                           no    │",
      "│                                                                              │",
      "│                                                                              │",
      "╰──────────────────────────────────────────────────────────────────────────────╯",
    ]);

    for i in 1..=7 {
      // Task
      expected[(i, 0)].set_style(Style::default().add_modifier(Modifier::BOLD));
    }

    for i in 1..=10 {
      // Task
      expected[(i, 1)].set_style(Style::default().add_modifier(Modifier::UNDERLINED));
    }

    for i in 12..=71 {
      // Task
      expected[(i, 1)].set_style(Style::default().add_modifier(Modifier::UNDERLINED));
    }

    for i in 73..=78 {
      // Task
      expected[(i, 1)].set_style(Style::default().add_modifier(Modifier::UNDERLINED));
    }

    for i in 1..=78 {
      // Task
      expected[(i, 3)].set_style(Style::default().add_modifier(Modifier::BOLD));
    }

    let mut app = TaskwarriorTui::new("next", false).await.unwrap();

    app.mode = Mode::Tasks(Action::ContextMenu);
    app.task_report_next();
    app.update(true).await.unwrap();

    let backend = TestBackend::new(80, 10);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
      .draw(|f| {
        app.draw_context_menu(f, 100, 100);
        app.draw_context_menu(f, 100, 100);
      })
      .unwrap();

    assert_eq!(terminal.backend().size().unwrap(), expected.area.into());
    terminal.backend().assert_buffer(&expected);
  }

  // #[test]
  async fn test_graphemes() {
    dbg!("写作业".graphemes(true).count());
    dbg!(UnicodeWidthStr::width("写作业"));
    dbg!(UnicodeWidthStr::width("abc"));

    let mut app = TaskwarriorTui::new("next", false).await.unwrap();

    if let Some(task) = app.task_by_id(27) {
      let i = app.task_index_by_uuid(*task.uuid()).unwrap_or_default();
      app.current_selection = i;
      app.current_selection_id = None;
      app.current_selection_uuid = None;
    }
    app.update(true).await.unwrap();
    app.mode = Mode::Tasks(Action::Modify);
    match app.task_current() {
      Some(t) => {
        let s = format!("{} ", t.description());
        app.modify.update(&s, s.as_str().len(), &mut app.changes)
      }
      None => app.modify.update("", 0, &mut app.changes),
    }
    app.update(true).await.unwrap();

    dbg!(app.modify.as_str());
    dbg!(app.modify.as_str().len());
    dbg!(app.modify.graphemes(true).count());
    dbg!(app.modify.pos());
    let position = TaskwarriorTui::get_position(&app.modify);
    dbg!(position);
  }

  // #[test]
  async fn test_taskwarrior_tui_completion() {
    let mut app = TaskwarriorTui::new("next", false).await.unwrap();
    app.handle_input(KeyCode::Char('z')).await.unwrap();
    app.mode = Mode::Tasks(Action::Add);
    app.update_completion_list();
    let input = "Wash car";
    for c in input.chars() {
      app.handle_input(KeyCode::Char(c)).await.unwrap();
    }
    app.handle_input(KeyCode::Ctrl('e')).await.unwrap();

    let input = " project:CO";
    for c in input.chars() {
      app.handle_input(KeyCode::Char(c)).await.unwrap();
    }

    app.mode = Mode::Tasks(Action::Add);
    app.update_completion_list();
    app.handle_input(KeyCode::Tab).await.unwrap();
    app.handle_input(KeyCode::Char('\n')).await.unwrap();
    let backend = TestBackend::new(80, 50);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
      .draw(|f| {
        app.draw(f);
        app.draw(f);
      })
      .unwrap();
    println!("{}", buffer_view(terminal.backend().buffer()));
  }
}
