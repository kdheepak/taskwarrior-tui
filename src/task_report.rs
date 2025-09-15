use std::{error::Error, process::Command};

use anyhow::Result;
use chrono::{DateTime, Datelike, Local, NaiveDate, NaiveDateTime, TimeZone};
use itertools::join;
use task_hookrs::{task::Task, uda::UDAValue};
use unicode_truncate::UnicodeTruncateStr;
use unicode_width::UnicodeWidthStr;

pub fn format_date_time(dt: NaiveDateTime) -> String {
  let dt = Local.from_local_datetime(&dt).unwrap();
  dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn format_date(dt: NaiveDateTime) -> String {
  let offset = Local.offset_from_utc_datetime(&dt);
  let dt = DateTime::<Local>::from_naive_utc_and_offset(dt, offset);
  dt.format("%Y-%m-%d").to_string()
}

pub fn format_date_formatted(dt: NaiveDateTime, format: String) -> String {
  let offset = Local.offset_from_utc_datetime(&dt);
  let dt = DateTime::<Local>::from_naive_utc_and_offset(dt, offset);
  dt.format(&format).to_string()
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

  const YEAR: i64 =  60 * 60 * 24 * 365;
  const MONTH: i64 = 60 * 60 * 24 * 30;
  const WEEK: i64 = 60 * 60 * 24 * 7;
  const DAY: i64 = 60 * 60 * 24;
  const HOUR: i64 = 60 * 60;
  const MINUTE: i64 = 60;

  if seconds >= YEAR {
    return if with_remainder {
      format!("{}{}y{}mo", minus, seconds / YEAR, (seconds - YEAR * (seconds / YEAR)) / MONTH)
    } else {
      format!("{}{}y", minus, seconds / YEAR)
    };
  }
  if seconds >= MONTH * 3 {
    return if with_remainder {
      format!("{}{}mo{}w", minus, seconds / MONTH, (seconds - MONTH * (seconds / MONTH)) / WEEK)
    } else {
      format!("{}{}mo", minus, seconds / MONTH)
    };
  }
  if seconds >= WEEK * 2 {
    return if with_remainder {
      format!("{}{}w{}d", minus, seconds / WEEK, (seconds - WEEK * (seconds / WEEK)) / DAY)
    } else {
      format!("{}{}w", minus, seconds / WEEK)
    };
  }
  if seconds >= DAY {
    return if with_remainder {
      format!("{}{}d{}h", minus, seconds / DAY, (seconds - DAY * (seconds / DAY)) / HOUR)
    } else {
      format!("{}{}d", minus, seconds / DAY)
    };
  }
  if seconds >= HOUR {
    return if with_remainder {
      format!("{}{}h{}min", minus, seconds / HOUR, (seconds - HOUR * (seconds / HOUR)) / MINUTE)
    } else {
      format!("{}{}h", minus, seconds / HOUR)
    };
  }
  if seconds >= MINUTE {
    return if with_remainder {
      format!("{}{}min{}s", minus, seconds / MINUTE, (seconds - MINUTE * (seconds / MINUTE)))
    } else {
      format!("{}{}min", minus, seconds / MINUTE)
    };
  }
  format!("{}{}s", minus, seconds)
}

pub struct TaskReportTable {
  pub labels: Vec<String>,
  pub columns: Vec<String>,
  pub tasks: Vec<Vec<String>>,
  pub virtual_tags: Vec<String>,
  pub description_width: usize,
  pub date_time_vague_precise: bool,
  pub date_format: String,
}

impl TaskReportTable {
  pub fn new(data: &str, report: &str) -> Result<Self> {
    let virtual_tags = vec![
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
    let mut task_report_table = Self {
      labels: vec![],
      columns: vec![],
      tasks: vec![vec![]],
      virtual_tags: virtual_tags.iter().map(ToString::to_string).collect::<Vec<_>>(),
      description_width: 100,
      date_time_vague_precise: false,
      date_format: "%Y-%m-%d".to_string(),
    };
    task_report_table.export_headers(Some(data), report)?;
    Ok(task_report_table)
  }

  pub fn export_headers(&mut self, data: Option<&str>, report: &str) -> Result<()> {
    self.columns = vec![];
    self.labels = vec![];

    let data = if let Some(s) = data {
      s.to_string()
    } else {
      let output = Command::new("task")
        .arg("show")
        .arg("rc.defaultwidth=0")
        .arg(format!("report.{}.columns", report))
        .output()?;
      String::from_utf8_lossy(&output.stdout).into_owned()
    };

    for line in data.split('\n') {
      if line.starts_with(format!("report.{}.columns", report).as_str()) {
        let column_names = line.split_once(' ').unwrap().1;
        for column in column_names.split(',') {
          self.columns.push(column.to_string());
        }
      }
    }

    let output = Command::new("task")
      .arg("show")
      .arg("rc.defaultwidth=0")
      .arg(format!("report.{}", report))
      .output()?;
    let data = String::from_utf8_lossy(&output.stdout);

    for line in data.split('\n') {
      if line.starts_with(format!("report.{}.labels", report).as_str()) {
        let label_names = line.split_once(' ').unwrap().1;
        for label in label_names.split(',') {
          self.labels.push(label.to_string());
        }
      }
      else if line.starts_with(format!("report.{}.dateformat", report).as_str()) {
        let taskwarrior_dateformat = line.split_once(' ').unwrap().1.to_string();
        self.date_format = taskwarrior_dateformat.chars().map(|c| {
          if c.is_alphabetic() {
            format!("%{}", c)
          } else {
            c.to_string()
          }
        }).collect::<String>();
        println!("found date format: {}", self.date_format);
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
    assert!(num_labels == num_columns, "Must have the same number of labels (currently {}) and columns (currently {}). Compare their values as shown by \"task show report.{}.\" and fix your taskwarrior config.", num_labels, num_columns, report);

    Ok(())
  }

  pub fn generate_table(&mut self, tasks: &[Task]) {
    self.tasks = vec![];

    // get all tasks as their string representation
    for task in tasks {
      if self.columns.is_empty() {
        break;
      }
      let mut item = vec![];
      for name in &self.columns {
        let s = self.get_string_attribute(name, task, tasks);
        item.push(s);
      }
      self.tasks.push(item);
    }
  }

  pub fn simplify_table(&mut self) -> (Vec<Vec<String>>, Vec<String>) {
    // find which columns are empty
    if self.tasks.is_empty() {
      return (vec![], vec![]);
    }

    let mut null_columns = vec![0; self.tasks[0].len()];

    for task in &self.tasks {
      for (i, s) in task.iter().enumerate() {
        null_columns[i] += s.len();
      }
    }

    // filter out columns where everything is empty
    let mut tasks = vec![];
    for task in &self.tasks {
      let t = task.clone();
      let t: Vec<String> = t
        .iter()
        .enumerate()
        .filter(|&(i, _)| null_columns[i] != 0)
        .map(|(_, e)| e.clone())
        .collect();
      tasks.push(t);
    }

    // filter out header where all columns are empty
    let headers: Vec<String> = self
      .labels
      .iter()
      .enumerate()
      .filter(|&(i, _)| null_columns[i] != 0)
      .map(|(_, e)| e.clone())
      .collect();

    (tasks, headers)
  }

  pub fn get_string_attribute(&self, attribute: &str, task: &Task, tasks: &[Task]) -> String {
    match attribute {
      "id" => task.id().unwrap_or_default().to_string(),
      "scheduled.relative" => match task.scheduled() {
        Some(v) => vague_format_date_time(
          Local::now().naive_utc(),
          NaiveDateTime::new(v.date(), v.time()),
          self.date_time_vague_precise,
        ),
        None => "".to_string(),
      },
      "scheduled.countdown" => match task.scheduled() {
        Some(v) => vague_format_date_time(
          Local::now().naive_utc(),
          NaiveDateTime::new(v.date(), v.time()),
          self.date_time_vague_precise,
        ),
        None => "".to_string(),
      },
      "scheduled" => match task.scheduled() {
        Some(v) => format_date(NaiveDateTime::new(v.date(), v.time())),
        None => "".to_string(),
      },
      "due.relative" => match task.due() {
        Some(v) => vague_format_date_time(
          Local::now().naive_utc(),
          NaiveDateTime::new(v.date(), v.time()),
          self.date_time_vague_precise,
        ),
        None => "".to_string(),
      },
      "due" => match task.due() {
        Some(v) => format_date_formatted(NaiveDateTime::new(v.date(), v.time()), self.date_format.clone()),
        None => "".to_string(),
      },
      "until.remaining" => match task.until() {
        Some(v) => vague_format_date_time(
          Local::now().naive_utc(),
          NaiveDateTime::new(v.date(), v.time()),
          self.date_time_vague_precise,
        ),
        None => "".to_string(),
      },
      "until" => match task.until() {
        Some(v) => format_date(NaiveDateTime::new(v.date(), v.time())),
        None => "".to_string(),
      },
      "entry.age" => vague_format_date_time(
        NaiveDateTime::new(task.entry().date(), task.entry().time()),
        Local::now().naive_utc(),
        self.date_time_vague_precise,
      ),
      "entry" => format_date(NaiveDateTime::new(task.entry().date(), task.entry().time())),
      "start.age" => match task.start() {
        Some(v) => vague_format_date_time(
          NaiveDateTime::new(v.date(), v.time()),
          Local::now().naive_utc(),
          self.date_time_vague_precise,
        ),
        None => "".to_string(),
      },
      "start" => match task.start() {
        Some(v) => format_date(NaiveDateTime::new(v.date(), v.time())),
        None => "".to_string(),
      },
      "end.age" => match task.end() {
        Some(v) => vague_format_date_time(
          NaiveDateTime::new(v.date(), v.time()),
          Local::now().naive_utc(),
          self.date_time_vague_precise,
        ),
        None => "".to_string(),
      },
      "end" => match task.end() {
        Some(v) => format_date(NaiveDateTime::new(v.date(), v.time())),
        None => "".to_string(),
      },
      "status.short" => task.status().to_string().chars().next().unwrap().to_string(),
      "status" => task.status().to_string(),
      "priority" => match task.priority() {
        Some(p) => p.clone(),
        None => "".to_string(),
      },
      "project" => match task.project() {
        Some(p) => p.to_string(),
        None => "".to_string(),
      },
      "depends.count" => match task.depends() {
        Some(v) => {
          if v.is_empty() {
            "".to_string()
          } else {
            format!("{}", v.len())
          }
        }
        None => "".to_string(),
      },
      "depends" => match task.depends() {
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
            join(dt.iter().map(ToString::to_string), " ")
          }
        }
        None => "".to_string(),
      },
      "tags.count" => match task.tags() {
        Some(v) => {
          let t = v.iter().filter(|t| !self.virtual_tags.contains(t)).count();
          if t == 0 {
            "".to_string()
          } else {
            t.to_string()
          }
        }
        None => "".to_string(),
      },
      "tags" => match task.tags() {
        Some(v) => v.iter().filter(|t| !self.virtual_tags.contains(t)).cloned().collect::<Vec<_>>().join(","),
        None => "".to_string(),
      },
      "recur" => match task.recur() {
        Some(v) => v.clone(),
        None => "".to_string(),
      },
      "wait" => match task.wait() {
        Some(v) => vague_format_date_time(
          NaiveDateTime::new(v.date(), v.time()),
          Local::now().naive_utc(),
          self.date_time_vague_precise,
        ),
        None => "".to_string(),
      },
      "wait.remaining" => match task.wait() {
        Some(v) => vague_format_date_time(
          Local::now().naive_utc(),
          NaiveDateTime::new(v.date(), v.time()),
          self.date_time_vague_precise,
        ),
        None => "".to_string(),
      },
      "description.count" => {
        let c = if let Some(a) = task.annotations() {
          format!("[{}]", a.len())
        } else {
          Default::default()
        };
        format!("{} {}", task.description(), c)
      }
      "description.truncated_count" => {
        let c = if let Some(a) = task.annotations() {
          format!("[{}]", a.len())
        } else {
          Default::default()
        };
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
      }
      "description.truncated" => {
        let d = task.description().to_string();
        let available_width = self.description_width;
        let (d, _) = d.unicode_truncate(available_width);
        let mut d = d.to_string();
        if d != *task.description() {
          d = format!("{}\u{2026}", d);
        }
        d
      }
      "description.desc" | "description" => task.description().to_string(),
      "urgency" => match &task.urgency() {
        Some(f) => format!("{:.2}", *f),
        None => "0.00".to_string(),
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
      }
    }
  }
}
