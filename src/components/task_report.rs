use std::{collections::HashMap, time::Duration};

use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use itertools::Itertools;
use ratatui::{prelude::*, widgets::*};
use serde_derive::{Deserialize, Serialize};
use task_hookrs::{import::import, task::Task, uda::UDAValue};
use tokio::sync::mpsc::UnboundedSender;
use tui_input::backend::crossterm::EventHandler;
use unicode_truncate::UnicodeTruncateStr;
use unicode_width::UnicodeWidthStr;
use uuid::Uuid;

use super::{Component, Frame};
use crate::{
  action::Action,
  config::{Config, KeyBindings},
};

#[derive(Default)]
pub struct TaskReport {
  pub config: Config,
  pub command_tx: Option<UnboundedSender<Action>>,
  pub last_export: Option<std::time::SystemTime>,
  pub report: String,
  pub filter: String,
  pub current_context_filter: String,
  pub tasks: Vec<Task>,
  pub rows: Vec<Vec<String>>,
  pub state: TableState,
  pub columns: Vec<String>,
  pub labels: Vec<String>,
  pub date_time_vague_precise: bool,
  pub virtual_tags: Vec<String>,
  pub description_width: usize,
  pub current_selection: usize,
  pub current_selection_id: Option<u64>,
  pub current_selection_uuid: Option<Uuid>,
}

impl TaskReport {
  pub fn new() -> Self {
    Self::default()
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
    for task in self.tasks.iter() {
      if self.columns.is_empty() {
        break;
      }
      let mut item = vec![];
      for name in &self.columns {
        let s = self.get_string_attribute(name, &task, &self.tasks);
        item.push(s);
      }
      self.rows.push(item);
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

  pub fn task_export(&mut self) -> Result<()> {
    let mut task = std::process::Command::new("task");

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
        self.tasks = imported?;
        log::debug!("Imported {} tasks", self.tasks.len());
        self.send_action(Action::ShowTaskReport)?;
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
    let mut widths = self.labels.iter().map(String::len).collect::<Vec<usize>>();
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
    while (widths.iter().sum::<usize>() as u16) >= maximum_available_width - (self.labels.len()) as u16 {
      let index = widths.iter().position(|i| i == widths.iter().max().unwrap_or(&0)).unwrap_or_default();
      if widths[index] == 1 {
        break;
      }
      widths[index] -= 1;
    }
    widths
  }

  fn style_for_task(&self, task: &Task) -> Style {
    let virtual_tag_names_in_precedence = &self.config.taskwarrior.rule_precedence_color;

    let mut style = Style::default();

    for tag_name in virtual_tag_names_in_precedence.iter().rev() {
      if tag_name == "uda." || tag_name == "priority" {
        if let Some(p) = task.priority() {
          let s = self.config.taskwarrior.color.uda_priority.get(p).copied().unwrap_or_default();
          style = style.patch(s);
        }
      } else if tag_name == "tag." {
        if let Some(tags) = task.tags() {
          for t in tags {
            let s = self.config.taskwarrior.color.tag.get(t).copied().unwrap_or_default();
            style = style.patch(s);
          }
        }
      } else if tag_name == "project." {
        if let Some(p) = task.project() {
          let s = self.config.taskwarrior.color.project.get(p).copied().unwrap_or_default();
          style = style.patch(s);
        }
      } else if task.tags().unwrap_or(&vec![]).contains(&tag_name.to_string().replace('.', "").to_uppercase()) {
        let s = self.config.taskwarrior.color.tag.get(tag_name).copied().unwrap_or_default();
        style = style.patch(s);
      }
    }

    style
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
      Action::MoveDown => self.next(),
      Action::MoveUp => self.previous(),
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
      Row::new(row.clone())
    });
    let table = Table::new(rows)
      .header(Row::new(self.labels.iter().map(|l| Cell::from(l.clone()).underlined())))
      .widths(&constraints)
      .highlight_symbol(&self.config.task_report.selection_indicator)
      .highlight_spacing(HighlightSpacing::Always)
      .column_spacing(column_spacing);
    f.render_stateful_widget(table, rect, &mut self.state);
    Ok(())
  }
}

pub fn format_date_time(dt: NaiveDateTime) -> String {
  let dt = Local.from_local_datetime(&dt).unwrap();
  dt.format("%Y-%m-%d %H:%M:%S").to_string()
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
