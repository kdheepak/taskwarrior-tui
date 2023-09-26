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
use uuid::Uuid;

use super::{Component, Frame};
use crate::{
  command::Command,
  config::{Config, KeyBindings},
};

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

#[derive(Default)]
pub struct TaskReport {
  pub config: Config,
  pub command_tx: Option<UnboundedSender<Command>>,
  pub last_export: Option<std::time::SystemTime>,
  pub report: String,
  pub filter: String,
  pub current_context_filter: String,
  pub tasks: Vec<Task>,
  pub rows: Vec<Vec<String>>,
  pub headers: Vec<String>,
  pub state: TableState,
  pub columns: Vec<String>,
  pub labels: Vec<String>,
  pub date_time_vague_precise: bool,
  pub virtual_tags: Vec<String>,
  pub description_width: usize,
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

  pub fn send_command(&self, command: Command) -> Result<()> {
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
    let num_labels = self.labels.len();
    let num_columns = self.columns.len();
    assert!(num_labels == num_columns, "Must have the same number of labels (currently {}) and columns (currently {}). Compare their values as shown by \"task show report.{}.\" and fix your taskwarrior config.", num_labels, num_columns, &self.report);

    Ok(())
  }

  pub fn generate_rows(&mut self) -> Result<()> {
    self.rows = vec![];

    // get all tasks as their string representation
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

    log::info!("Running `{:?}`", task);
    let output = task.output()?;
    let data = String::from_utf8_lossy(&output.stdout);
    let error = String::from_utf8_lossy(&output.stderr);

    if output.status.success() {
      let imported = import(data.as_bytes());
      if imported.is_ok() {
        self.tasks = imported?;
        log::info!("Imported {} tasks", self.tasks.len());
        self.send_command(Command::ShowTaskReport)?;
      } else {
        imported?;
      }
    } else {
      self.send_command(Command::Error(format!("Unable to parse output of `{:?}`:\n`{:?}`", task, data)))?;
    }

    Ok(())
  }
}

impl Component for TaskReport {
  fn register_command_handler(&mut self, tx: UnboundedSender<Command>) -> Result<()> {
    self.command_tx = Some(tx);
    Ok(())
  }

  fn register_config_handler(&mut self, config: Config) -> Result<()> {
    self.config = config;
    Ok(())
  }

  fn update(&mut self, command: Command) -> Result<Option<Command>> {
    match command {
      Command::Tick => {
        self.task_export()?;
        self.export_headers()?;
        self.generate_rows()?;
      },
      _ => {},
    }
    Ok(None)
  }

  fn draw(&mut self, f: &mut Frame<'_>, rect: Rect) -> Result<()> {
    let mut constraints = vec![];
    for i in 0..self.rows[0].len() {
      constraints.push(Constraint::Percentage(100 / self.rows[0].len() as u16));
    }
    let rows = self.rows.iter().map(|row| Row::new(row.clone()));
    let table = Table::new(rows).header(Row::new(self.headers.clone())).widths(&constraints);
    f.render_stateful_widget(table, rect, &mut self.state);
    Ok(())
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
