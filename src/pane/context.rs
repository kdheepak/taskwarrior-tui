use std::fmt;

use anyhow::{Context as AnyhowContext, Result, anyhow};

const NAME: &str = "Name";
const TYPE: &str = "Remaining";
const DEFINITION: &str = "Avg age";
const ACTIVE: &str = "Complete";

use std::{
  cmp,
  cmp::min,
  collections::{HashMap, HashSet},
  error::Error,
  process::{Command, Output},
};

use chrono::{Datelike, Duration, Local, Month, NaiveDate, NaiveDateTime, TimeZone};
use itertools::Itertools;
use ratatui::{
  buffer::Buffer,
  layout::{Alignment, Rect},
  style::{Color, Modifier, Style},
  symbols,
  text::{Line, Span, Text},
  widgets::{Block, BorderType, Borders, Clear, Paragraph, StatefulWidget, Widget},
};
use uuid::Uuid;

use crate::{
  action::Action,
  app::{Mode, TaskwarriorTui},
  event::KeyCode,
  pane::Pane,
  table::TaskwarriorTuiTableState,
};

#[derive(Debug, Clone, Default)]
pub struct ContextDetails {
  pub name: String,
  pub definition: String,
  pub active: String,
  pub type_: String,
}

impl ContextDetails {
  pub fn new(name: String, definition: String, active: String, type_: String) -> Self {
    Self {
      name,
      definition,
      active,
      type_,
    }
  }
}

pub struct ContextsState {
  pub table_state: TaskwarriorTuiTableState,
  pub report_height: u16,
  pub columns: Vec<String>,
  pub rows: Vec<ContextDetails>,
  /// Current search query typed by the user inside the popup.
  pub search: String,
}

impl ContextsState {
  pub(crate) fn new() -> Self {
    Self {
      table_state: TaskwarriorTuiTableState::default(),
      report_height: 0,
      columns: vec![NAME.to_string(), TYPE.to_string(), DEFINITION.to_string(), ACTIVE.to_string()],
      rows: vec![],
      search: String::new(),
    }
  }

  pub fn simplified_view(&mut self) -> (Vec<Vec<String>>, Vec<String>) {
    let rows = self
      .rows
      .iter()
      .map(|c| vec![c.name.clone(), c.type_.clone(), c.definition.clone(), c.active.clone()])
      .collect();
    let headers = self.columns.clone();
    (rows, headers)
  }

  pub fn len(&self) -> usize {
    self.rows.len()
  }

  /// Returns the indices into `self.rows` (after filtering to `type_ == "read"`)
  /// that match the current search query.
  /// An empty query matches everything. Matching is case-insensitive substring
  /// on name or definition.
  pub fn filtered_indices(&self) -> Vec<usize> {
    let query = self.search.to_lowercase();
    self
      .rows
      .iter()
      .enumerate()
      .filter(|(_, r)| {
        r.type_ == "read" && (query.is_empty() || r.name.to_lowercase().contains(&query) || r.definition.to_lowercase().contains(&query))
      })
      .map(|(i, _)| i)
      .collect()
  }

  pub fn update_data(&mut self, task_exe: &str) -> Result<()> {
    let output = Command::new(task_exe).arg("context").output()?;
    let data = String::from_utf8_lossy(&output.stdout);

    self.rows = Self::parse(&data);
    if self.rows.iter().any(|r| r.active != "no") {
      self.rows.insert(
        0,
        ContextDetails::new("none".to_string(), "".to_string(), "no".to_string(), "read".to_string()),
      );
    } else {
      self.rows.insert(
        0,
        ContextDetails::new("none".to_string(), "".to_string(), "yes".to_string(), "read".to_string()),
      );
    }
    Ok(())
  }

  fn parse(data: &str) -> Vec<ContextDetails> {
    let mut rows: Vec<ContextDetails> = vec![];
    for (i, line) in data.trim().split('\n').enumerate() {
      if line.starts_with("  ") && line.trim().starts_with("write") {
        continue;
      }
      if line.starts_with("  ") && !(line.trim().ends_with("yes") || line.trim().ends_with("no")) {
        let definition = line.trim();
        if let Some(c) = rows.last_mut() {
          c.definition = format!("{} {}", c.definition, definition);
        }
        continue;
      }
      let line = line.trim();
      if line.is_empty() || line == "Use 'task context none' to unset the current context." {
        continue;
      }
      // Skip the header row and, on taskwarrior 2.x, the dashed separator row under it.
      // taskwarrior 3.x underlines the header instead, so the first context sits on line 1.
      if i == 0 || line.chars().all(|c| c == '-' || c == ' ') {
        continue;
      }
      let mut s = line.split_whitespace();
      let name = s.next().unwrap_or_default();
      let typ = s.next().unwrap_or_default();
      let active = s.last().unwrap_or_default();
      let definition = line.replacen(name, "", 1);
      let definition = definition.replacen(typ, "", 1);
      let definition = definition.strip_suffix(active).unwrap_or_default();
      rows.push(ContextDetails::new(
        name.to_string(),
        definition.trim().to_string(),
        active.to_string(),
        typ.to_string(),
      ));
    }
    rows
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn names(rows: &[ContextDetails]) -> Vec<&str> {
    rows.iter().map(|r| r.name.as_str()).collect()
  }

  #[test]
  fn parse_taskwarrior_3_output_keeps_first_context() {
    let data = [
      "",
      "\x1b[4mName   \x1b[0m \x1b[4mType \x1b[0m \x1b[4mDefinition   \x1b[0m \x1b[4mActive\x1b[0m",
      "agent  read  project:agent no",
      "       write project:agent no",
      "comms  read  +comms        yes",
      "       write +comms        no",
      "",
      "Use 'task context none' to unset the current context.",
    ]
    .join("\n");
    let rows = ContextsState::parse(&data);
    assert_eq!(names(&rows), ["agent", "comms"]);
    assert_eq!(rows[0].definition, "project:agent");
    assert_eq!(rows[1].active, "yes");
  }

  #[test]
  fn parse_taskwarrior_2_output_skips_separator() {
    let data = [
      "",
      "Name   Type  Definition    Active",
      "-----  ----- ------------- ------",
      "agent  read  project:agent no",
      "       write project:agent no",
    ]
    .join("\n");
    let rows = ContextsState::parse(&data);
    assert_eq!(names(&rows), ["agent"]);
  }
}
