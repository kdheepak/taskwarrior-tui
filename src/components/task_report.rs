use std::{collections::HashMap, time::Duration};

use chrono::{DateTime, Datelike, Local, NaiveDateTime, TimeZone};
use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use itertools::Itertools;
use ratatui::{prelude::*, widgets::*};
use serde_derive::{Deserialize, Serialize};
use task_hookrs::{import::import, status::TaskStatus, task::Task, uda::UDAValue};
use tokio::sync::mpsc::UnboundedSender;
use tui_input::{backend::crossterm::EventHandler, Input};
use unicode_truncate::UnicodeTruncateStr;
use unicode_width::UnicodeWidthStr;
use uuid::Uuid;

use super::{Component, Frame};
use crate::{
  action::{Action, TaskCommand},
  config::{Config, KeyBindings},
};
const VIRTUAL_TAGS: [&str; 34] = [
  "PROJECT",
  "BLOCKED",
  "UNBLOCKED",
  "BLOCKING",
  "DUE",
  "DUETODAY",
  "TODAY",
  "OVERDUE",
  "WEEK",
  "MONTH",
  "QUARTER",
  "YEAR",
  "ACTIVE",
  "SCHEDULED",
  "PARENT",
  "CHILD",
  "UNTIL",
  "WAITING",
  "ANNOTATED",
  "READY",
  "YESTERDAY",
  "TOMORROW",
  "TAGGED",
  "PENDING",
  "COMPLETED",
  "DELETED",
  "UDA",
  "ORPHAN",
  "PRIORITY",
  "PROJECT",
  "LATEST",
  "RECURRING",
  "INSTANCE",
  "TEMPLATE",
];

#[derive(Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum Mode {
  #[default]
  Report,
  Filter,
  Add,
  Annotate,
  Subprocess,
  Log,
  Modify,
  HelpPopup,
  ContextMenu,
  Jump,
  DeletePrompt,
  UndoPrompt,
  DonePrompt,
  Error,
}

#[derive(Default)]
pub struct TaskReport {
  pub columns: Vec<String>,
  pub command_tx: Option<UnboundedSender<Action>>,
  pub config: Config,
  pub current_context: String,
  pub current_context_filter: String,
  pub current_filter: String,
  pub current_selection: usize,
  pub current_selection_id: Option<u64>,
  pub current_selection_uuid: Option<Uuid>,
  pub date_time_vague_precise: bool,
  pub description_width: usize,
  pub input: Input,
  pub labels: Vec<String>,
  pub last_export: Option<std::time::SystemTime>,
  pub report: String,
  pub row_heights: Vec<u16>,
  pub rows: Vec<Vec<String>>,
  pub state: TableState,
  pub task_details: HashMap<Uuid, String>,
  pub tasks: Vec<Task>,
  pub virtual_tags: Vec<String>,
  pub mode: Mode,
}

impl TaskReport {
  pub fn new() -> Self {
    let mut s = Self::default();
    s.virtual_tags = VIRTUAL_TAGS.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    s
  }

  pub fn report(mut self, report: String) -> Self {
    self.report = report;
    self
  }

  pub fn refresh(&mut self) -> Result<()> {
    self.last_export = Some(std::time::SystemTime::now());
    Ok(())
  }

  pub fn send_action(&self, command: Action) -> Result<()> {
    if let Some(ref tx) = self.command_tx {
      tx.send(command)?;
    }
    Ok(())
  }

  pub fn export_headers(&mut self) -> Result<()> {
    self.columns = vec![];
    self.labels = vec![];

    let output = std::process::Command::new("task")
      .arg("show")
      .arg("rc.defaultwidth=0")
      .arg(format!("report.{}.columns", &self.report))
      .output()?;
    let data = String::from_utf8_lossy(&output.stdout).into_owned();

    for line in data.split('\n') {
      if line.starts_with(format!("report.{}.columns", &self.report).as_str()) {
        let column_names = line.split_once(' ').unwrap().1;
        for column in column_names.split(',') {
          self.columns.push(column.to_string());
        }
      }
    }

    let output = std::process::Command::new("task")
      .arg("show")
      .arg("rc.defaultwidth=0")
      .arg(format!("report.{}.labels", &self.report))
      .output()?;
    let data = String::from_utf8_lossy(&output.stdout);

    for line in data.split('\n') {
      if line.starts_with(format!("report.{}.labels", &self.report).as_str()) {
        let label_names = line.split_once(' ').unwrap().1;
        for label in label_names.split(',') {
          self.labels.push(label.to_string());
        }
      }
    }

    if self.labels.is_empty() {
      for label in &self.columns {
        let label = label.split('.').collect::<Vec<&str>>()[0];
        let label = if label == "id" { "ID" } else { label };
        let mut c = label.chars();
        let label = match c.next() {
          None => String::new(),
          Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        };
        if !label.is_empty() {
          self.labels.push(label);
        }
      }
    }
    if self.labels.len() != self.columns.len() {
      return Err(color_eyre::eyre::eyre!(format!("`{}` expects to have the same number of labels and columns ({} != {}). Compare their values as shown by `task show report.{}.` and fix your taskwarrior config.", env!("CARGO_PKG_NAME"), self.labels.len(), self.columns.len(), &self.report)));
    }

    Ok(())
  }

  pub fn generate_rows(&mut self) -> Result<()> {
    self.rows = vec![];
    self.row_heights = vec![];
    for task in self.tasks.iter() {
      if self.columns.is_empty() {
        break;
      }
      let mut item = vec![];
      let mut row_height = 1;
      for name in &self.columns {
        let s = if name == "description" {
          let mut s = self.get_string_attribute(name, &task, &self.tasks);
          if let Some(annotations) = task.annotations() {
            if annotations.len() > 0 {
              for annotation in annotations {
                s.push_str(&format!(
                  "\n {} {}",
                  format_date(NaiveDateTime::new(annotation.entry().date(), annotation.entry().time())),
                  annotation.description()
                ));
                row_height += 1;
              }
            }
          }
          s
        } else {
          self.get_string_attribute(name, &task, &self.tasks)
        };
        item.push(s);
      }
      self.rows.push(item);
      self.row_heights.push(row_height);
    }
    Ok(())
  }

  pub fn get_string_attribute(&self, attribute: &str, task: &Task, tasks: &[Task]) -> String {
    match attribute {
      "id" => task.id().unwrap_or_default().to_string(),
      "scheduled.relative" => {
        match task.scheduled() {
          Some(v) => {
            vague_format_date_time(
              Local::now().naive_utc(),
              NaiveDateTime::new(v.date(), v.time()),
              self.date_time_vague_precise,
            )
          },
          None => "".to_string(),
        }
      },
      "due.relative" => {
        match task.due() {
          Some(v) => {
            vague_format_date_time(
              Local::now().naive_utc(),
              NaiveDateTime::new(v.date(), v.time()),
              self.date_time_vague_precise,
            )
          },
          None => "".to_string(),
        }
      },
      "due" => {
        match task.due() {
          Some(v) => format_date(NaiveDateTime::new(v.date(), v.time())),
          None => "".to_string(),
        }
      },
      "until.remaining" => {
        match task.until() {
          Some(v) => {
            vague_format_date_time(
              Local::now().naive_utc(),
              NaiveDateTime::new(v.date(), v.time()),
              self.date_time_vague_precise,
            )
          },
          None => "".to_string(),
        }
      },
      "until" => {
        match task.until() {
          Some(v) => format_date(NaiveDateTime::new(v.date(), v.time())),
          None => "".to_string(),
        }
      },
      "entry.age" => {
        vague_format_date_time(
          NaiveDateTime::new(task.entry().date(), task.entry().time()),
          Local::now().naive_utc(),
          self.date_time_vague_precise,
        )
      },
      "entry" => format_date(NaiveDateTime::new(task.entry().date(), task.entry().time())),
      "start.age" => {
        match task.start() {
          Some(v) => {
            vague_format_date_time(
              NaiveDateTime::new(v.date(), v.time()),
              Local::now().naive_utc(),
              self.date_time_vague_precise,
            )
          },
          None => "".to_string(),
        }
      },
      "start" => {
        match task.start() {
          Some(v) => format_date(NaiveDateTime::new(v.date(), v.time())),
          None => "".to_string(),
        }
      },
      "end.age" => {
        match task.end() {
          Some(v) => {
            vague_format_date_time(
              NaiveDateTime::new(v.date(), v.time()),
              Local::now().naive_utc(),
              self.date_time_vague_precise,
            )
          },
          None => "".to_string(),
        }
      },
      "end" => {
        match task.end() {
          Some(v) => format_date(NaiveDateTime::new(v.date(), v.time())),
          None => "".to_string(),
        }
      },
      "status.short" => task.status().to_string().chars().next().unwrap().to_string(),
      "status" => task.status().to_string(),
      "priority" => {
        match task.priority() {
          Some(p) => p.clone(),
          None => "".to_string(),
        }
      },
      "project" => {
        match task.project() {
          Some(p) => p.to_string(),
          None => "".to_string(),
        }
      },
      "depends.count" => {
        match task.depends() {
          Some(v) => {
            if v.is_empty() {
              "".to_string()
            } else {
              format!("{}", v.len())
            }
          },
          None => "".to_string(),
        }
      },
      "depends" => {
        match task.depends() {
          Some(v) => {
            if v.is_empty() {
              "".to_string()
            } else {
              let mut dt = vec![];
              for u in v {
                if let Some(t) = tasks.iter().find(|t| t.uuid() == u) {
                  dt.push(t.id().unwrap());
                }
              }
              dt.iter().map(ToString::to_string).join(" ")
            }
          },
          None => "".to_string(),
        }
      },
      "tags.count" => {
        match task.tags() {
          Some(v) => {
            let t = v.iter().filter(|t| !self.virtual_tags.contains(t)).count();
            if t == 0 {
              "".to_string()
            } else {
              t.to_string()
            }
          },
          None => "".to_string(),
        }
      },
      "tags" => {
        match task.tags() {
          Some(v) => v.iter().filter(|t| !self.virtual_tags.contains(t)).cloned().collect::<Vec<_>>().join(","),
          None => "".to_string(),
        }
      },
      "recur" => {
        match task.recur() {
          Some(v) => v.clone(),
          None => "".to_string(),
        }
      },
      "wait" => {
        match task.wait() {
          Some(v) => {
            vague_format_date_time(
              NaiveDateTime::new(v.date(), v.time()),
              Local::now().naive_utc(),
              self.date_time_vague_precise,
            )
          },
          None => "".to_string(),
        }
      },
      "wait.remaining" => {
        match task.wait() {
          Some(v) => {
            vague_format_date_time(
              Local::now().naive_utc(),
              NaiveDateTime::new(v.date(), v.time()),
              self.date_time_vague_precise,
            )
          },
          None => "".to_string(),
        }
      },
      "description.count" => {
        let c = if let Some(a) = task.annotations() { format!("[{}]", a.len()) } else { Default::default() };
        format!("{} {}", task.description(), c)
      },
      "description.truncated_count" => {
        let c = if let Some(a) = task.annotations() { format!("[{}]", a.len()) } else { Default::default() };
        let d = task.description().to_string();
        let mut available_width = self.description_width;
        if self.description_width >= c.len() {
          available_width = self.description_width - c.len();
        }
        let (d, _) = d.unicode_truncate(available_width);
        let mut d = d.to_string();
        if d != *task.description() {
          d = format!("{}\u{2026}", d);
        }
        format!("{}{}", d, c)
      },
      "description.truncated" => {
        let d = task.description().to_string();
        let available_width = self.description_width;
        let (d, _) = d.unicode_truncate(available_width);
        let mut d = d.to_string();
        if d != *task.description() {
          d = format!("{}\u{2026}", d);
        }
        d
      },
      "description.desc" | "description" => task.description().to_string(),
      "urgency" => {
        match &task.urgency() {
          Some(f) => format!("{:.2}", *f),
          None => "0.00".to_string(),
        }
      },
      s => {
        let u = &task.uda();
        let v = u.get(s);
        if v.is_none() {
          return "".to_string();
        }
        match v.unwrap() {
          UDAValue::Str(s) => s.to_string(),
          UDAValue::F64(f) => f.to_string(),
          UDAValue::U64(u) => u.to_string(),
        }
      },
    }
  }

  pub fn get_context(&mut self) -> Result<()> {
    let output = std::process::Command::new("task").arg("_get").arg("rc.context").output()?;
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
      let output =
        std::process::Command::new("task").arg("_get").arg(format!("rc.context.{}", self.current_context)).output()?;
      self.current_context_filter = String::from_utf8_lossy(&output.stdout).to_string();
      self.current_context_filter = self.current_context_filter.strip_suffix('\n').unwrap_or("").to_string();
    }
    Ok(())
  }

  pub fn task_export(&mut self) -> Result<()> {
    let mut task = std::process::Command::new("task");

    task
      .arg("rc.json.array=on")
      .arg("rc.confirmation=off")
      .arg("rc.json.depends.array=on")
      .arg("rc.color=off")
      .arg("rc._forcecolor=off");
    // .arg("rc.verbose:override=false");

    if let Some(args) =
      shlex::split(format!(r#"rc.report.{}.filter='{}'"#, self.report, self.current_filter.trim()).trim())
    {
      for arg in args {
        task.arg(arg);
      }
    }

    if !self.current_context_filter.trim().is_empty() {
      if let Some(args) = shlex::split(&self.current_context_filter) {
        for arg in args {
          task.arg(arg);
        }
      }
    }

    task.arg("export");

    task.arg(&self.report);

    log::debug!("Running `{:?}`", task);
    let output = task.output()?;
    let data = String::from_utf8_lossy(&output.stdout);
    let error = String::from_utf8_lossy(&output.stderr);

    if output.status.success() {
      let imported = import(data.as_bytes());
      if imported.is_ok() {
        let highlight_first_element = if self.tasks.is_empty() { true } else { false };
        self.tasks = imported?;
        log::debug!("Imported {} tasks", self.tasks.len());
        if highlight_first_element {
          self.state.select(Some(0));
        }
        self.update_tags();
      } else {
        imported?;
      }
    } else {
      self.send_action(Action::Error(format!("Unable to parse output of `{:?}`:\n`{:?}`", task, data)))?;
    }

    Ok(())
  }

  fn next(&mut self) {
    if self.tasks.is_empty() {
      return;
    }
    let i = {
      if self.current_selection >= self.tasks.len() - 1 {
        if self.config.task_report.looping {
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
    self.state.select(Some(self.current_selection));
    log::info!("{:?}", self.state);
  }

  pub fn previous(&mut self) {
    if self.tasks.is_empty() {
      return;
    }
    let i = {
      if self.current_selection == 0 {
        if self.config.task_report.looping {
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
    self.state.select(Some(self.current_selection));
    log::info!("{:?}", self.state);
  }

  pub fn calculate_widths(&self, maximum_available_width: u16) -> Vec<usize> {
    // naive implementation of calculate widths
    let mut widths = self.labels.iter().map(|s| s.len()).collect::<Vec<usize>>();
    for i in 0..self.labels.len() {
      let max_width = self.rows.iter().map(|row| row[i].len()).max().unwrap_or(0);
      if max_width == 0 {
        widths[i] = 0
      } else {
        widths[i] = widths[i].max(max_width);
      }
    }
    for (i, header) in self.labels.iter().enumerate() {
      if header == "Description" || header == "Definition" {
        // always give description or definition the most room to breath
        widths[i] = maximum_available_width as usize;
        break;
      }
    }
    for (i, header) in self.labels.iter().enumerate() {
      if i == 0 {
        // always give ID a couple of extra for indicator
        widths[i] += self.config.task_report.selection_indicator.as_str().width();
        // if let TableMode::MultipleSelection = self.task_table_state.mode() {
        //     widths[i] += 2
        // };
      }
    }
    // now start trimming
    while (widths.iter().sum::<usize>() as u16) >= maximum_available_width.saturating_sub(self.labels.len() as u16) {
      let index = widths.iter().position(|i| i == widths.iter().max().unwrap_or(&0)).unwrap_or_default();
      if widths[index] == 1 {
        break;
      }
      widths[index] -= 1;
    }
    widths
  }

  fn style_for_task(&self, task: &Task) -> Style {
    let rule_precedence_color = &self.config.taskwarrior.rule_precedence_color;

    let mut style = Style::default();

    for name in rule_precedence_color.iter().rev() {
      if name == "uda." || name == "priority" {
        if let Some(p) = task.priority() {
          let s = self.config.taskwarrior.color.get(&format!("color.priority.{p}")).copied().unwrap_or_default();
          style = style.patch(s);
        }
      } else if name == "tag." {
        if let Some(tags) = task.tags() {
          for t in tags {
            let s = self.config.taskwarrior.color.get(&format!("color.tag.{t}")).copied().unwrap_or_default();
            style = style.patch(s);
          }
        }
      } else if name == "project." {
        if let Some(p) = task.project() {
          let s = self.config.taskwarrior.color.get(&format!("color.project.{p}")).copied().unwrap_or_default();
          style = style.patch(s);
        }
      } else if task.tags().unwrap_or(&vec![]).contains(&name.to_string().replace('.', "").to_uppercase()) {
        let s = self.config.taskwarrior.color.get(&format!("color.{name}")).copied().unwrap_or_default();
        style = style.patch(s);
      }
    }

    style
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
      let virtual_tags = self.virtual_tags.clone();
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
          let reference = TimeZone::from_utc_datetime(now.offset(), d);
          let now = TimeZone::from_utc_datetime(now.offset(), &now.naive_utc());
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
          match get_date_state(&d, self.config.taskwarrior.due) {
            DateState::EarlierToday | DateState::LaterToday => {
              add_tag(task, "DUE".to_string());
              add_tag(task, "TODAY".to_string());
              add_tag(task, "DUETODAY".to_string());
            },
            DateState::AfterToday => {
              add_tag(task, "DUE".to_string());
              if reference.date_naive() == (now + chrono::Duration::days(1)).date_naive() {
                add_tag(task, "TOMORROW".to_string());
              }
            },
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

  fn mode(&mut self, mode: Mode) {
    self.mode = mode
  }
}

impl Component for TaskReport {
  fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
    self.command_tx = Some(tx);
    Ok(())
  }

  fn register_config_handler(&mut self, config: Config) -> Result<()> {
    self.config = config;
    Ok(())
  }

  fn update(&mut self, action: Action) -> Result<Option<Action>> {
    match action {
      Action::Tick => {
        self.task_export()?;
        self.export_headers()?;
        self.generate_rows()?;
      },
      Action::MoveDown => {
        if self.mode == Mode::Report {
          self.next()
        }
      },
      Action::MoveUp => {
        if self.mode == Mode::Report {
          self.previous()
        }
      },
      Action::ExecuteTask(t) => {
        match t {
          TaskCommand::Filter => self.mode(Mode::Filter),
          _ => {},
        }
      },
      _ => {},
    }
    Ok(None)
  }

  fn draw(&mut self, f: &mut Frame<'_>, rect: Rect) -> Result<()> {
    let column_spacing = 1;
    if self.rows.len() == 0 {
      f.render_widget(Paragraph::new("No data found").block(Block::new().borders(Borders::all())), rect);
      return Ok(());
    }
    let widths = self.calculate_widths(rect.width);
    let constraints: Vec<Constraint> = widths.iter().map(|i| Constraint::Min(*i as u16)).collect();
    let rows = self.rows.iter().enumerate().map(|(i, row)| {
      let style = self.style_for_task(&self.tasks[i]);
      Row::new(row.clone()).height(self.row_heights[i]).style(style)
    });
    let table = Table::new(rows)
      .header(Row::new(self.labels.iter().map(|l| {
        Cell::from(l.clone()).style(
          self
            .config
            .taskwarrior
            .color
            .get("color.label")
            .copied()
            .unwrap_or_default()
            .add_modifier(Modifier::UNDERLINED),
        )
      })))
      .widths(&constraints)
      .highlight_symbol(&self.config.task_report.selection_indicator)
      .highlight_spacing(HighlightSpacing::Always)
      .column_spacing(column_spacing);
    f.render_stateful_widget(table, rect, &mut self.state);
    Ok(())
  }
}

#[derive(Debug)]
pub enum DateState {
  BeforeToday,
  EarlierToday,
  LaterToday,
  AfterToday,
  NotDue,
}

pub fn get_date_state(reference: &task_hookrs::date::Date, due: usize) -> DateState {
  let now = Local::now();
  let reference = TimeZone::from_utc_datetime(now.offset(), reference);
  let now = TimeZone::from_utc_datetime(now.offset(), &now.naive_utc());

  if reference.date_naive() < now.date_naive() {
    return DateState::BeforeToday;
  }

  if reference.date_naive() == now.date_naive() {
    return if reference.time() < now.time() { DateState::EarlierToday } else { DateState::LaterToday };
  }

  if reference <= now + chrono::Duration::days(7) {
    DateState::AfterToday
  } else {
    DateState::NotDue
  }
}

pub fn format_date_time(dt: NaiveDateTime) -> String {
  let dt = Local.from_local_datetime(&dt).unwrap();
  dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

fn get_offset_hour_minute() -> (&'static str, i32, i32) {
  let off = Local::now().offset().local_minus_utc();
  let sym = if off >= 0 { "+" } else { "-" };
  let off = off.abs();
  let h = if off > 60 * 60 { off / 60 / 60 } else { 0 };
  let m = if (off - ((off / 60 / 60) * 60 * 60)) > 60 { (off - ((off / 60 / 60) * 60 * 60)) / 60 } else { 0 };
  (sym, h, m)
}

pub fn format_date(dt: NaiveDateTime) -> String {
  let offset = Local.offset_from_utc_datetime(&dt);
  let dt = DateTime::<Local>::from_naive_utc_and_offset(dt, offset);
  dt.format("%Y-%m-%d").to_string()
}

pub fn vague_format_date_time(from_dt: NaiveDateTime, to_dt: NaiveDateTime, with_remainder: bool) -> String {
  let to_dt = Local.from_local_datetime(&to_dt).unwrap();
  let from_dt = Local.from_local_datetime(&from_dt).unwrap();
  let mut seconds = (to_dt - from_dt).num_seconds();
  let minus = if seconds < 0 {
    seconds *= -1;
    "-"
  } else {
    ""
  };

  let year = 60 * 60 * 24 * 365;
  let month = 60 * 60 * 24 * 30;
  let week = 60 * 60 * 24 * 7;
  let day = 60 * 60 * 24;
  let hour = 60 * 60;
  let minute = 60;

  if seconds >= 60 * 60 * 24 * 365 {
    return if with_remainder {
      format!("{}{}y{}mo", minus, seconds / year, (seconds - year * (seconds / year)) / month)
    } else {
      format!("{}{}y", minus, seconds / year)
    };
  } else if seconds >= 60 * 60 * 24 * 90 {
    return if with_remainder {
      format!("{}{}mo{}w", minus, seconds / month, (seconds - month * (seconds / month)) / week)
    } else {
      format!("{}{}mo", minus, seconds / month)
    };
  } else if seconds >= 60 * 60 * 24 * 14 {
    return if with_remainder {
      format!("{}{}w{}d", minus, seconds / week, (seconds - week * (seconds / week)) / day)
    } else {
      format!("{}{}w", minus, seconds / week)
    };
  } else if seconds >= 60 * 60 * 24 {
    return if with_remainder {
      format!("{}{}d{}h", minus, seconds / day, (seconds - day * (seconds / day)) / hour)
    } else {
      format!("{}{}d", minus, seconds / day)
    };
  } else if seconds >= 60 * 60 {
    return if with_remainder {
      format!("{}{}h{}min", minus, seconds / hour, (seconds - hour * (seconds / hour)) / minute)
    } else {
      format!("{}{}h", minus, seconds / hour)
    };
  } else if seconds >= 60 {
    return if with_remainder {
      format!("{}{}min{}s", minus, seconds / minute, (seconds - minute * (seconds / minute)))
    } else {
      format!("{}{}min", minus, seconds / minute)
    };
  }
  format!("{}{}s", minus, seconds)
}

pub fn add_tag(task: &mut Task, tag: String) {
  match task.tags_mut() {
    Some(t) => t.push(tag),
    None => task.set_tags(Some(vec![tag])),
  }
}

pub fn remove_tag(task: &mut Task, tag: &str) {
  if let Some(t) = task.tags_mut() {
    if let Some(index) = t.iter().position(|x| *x == tag) {
      t.remove(index);
    }
  }
}

mod tests {
  use pretty_assertions::assert_eq;

  use super::*;

  #[test]
  fn test_export() -> Result<()> {
    let mut tr = TaskReport::new().report("next".into());
    tr.task_export()?;
    assert_eq!(tr.tasks.len(), 33);
    Ok(())
  }
}
