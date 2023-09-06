use std::fmt;

use color_eyre::eyre::{anyhow, Context as AnyhowContext, Result};

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
  pane::Pane,
  table::TableState,
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
  pub table_state: TableState,
  pub report_height: u16,
  pub columns: Vec<String>,
  pub rows: Vec<ContextDetails>,
}

impl ContextsState {
  pub(crate) fn new() -> Self {
    Self {
      table_state: TableState::default(),
      report_height: 0,
      columns: vec![
        NAME.to_string(),
        TYPE.to_string(),
        DEFINITION.to_string(),
        ACTIVE.to_string(),
      ],
      rows: vec![],
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

  pub fn update_data(&mut self) -> Result<()> {
    let output = Command::new("task").arg("context").output()?;
    let data = String::from_utf8_lossy(&output.stdout);

    self.rows = vec![];
    for (i, line) in data.trim().split('\n').enumerate() {
      if line.starts_with("  ") && line.trim().starts_with("write") {
        continue;
      }
      if line.starts_with("  ") && !(line.trim().ends_with("yes") || line.trim().ends_with("no")) {
        let definition = line.trim();
        if let Some(c) = self.rows.last_mut() {
          c.definition = format!("{} {}", c.definition, definition);
        }
        continue;
      }
      let line = line.trim();
      if line.is_empty() || line == "Use 'task context none' to unset the current context." {
        continue;
      }
      if i == 0 || i == 1 {
        continue;
      }
      let mut s = line.split_whitespace();
      let name = s.next().unwrap_or_default();
      let typ = s.next().unwrap_or_default();
      let active = s.last().unwrap_or_default();
      let definition = line.replacen(name, "", 1);
      let definition = definition.replacen(typ, "", 1);
      let definition = definition.strip_suffix(active).unwrap_or_default();
      let context = ContextDetails::new(
        name.to_string(),
        definition.trim().to_string(),
        active.to_string(),
        typ.to_string(),
      );
      self.rows.push(context);
    }
    if self.rows.iter().any(|r| r.active != "no") {
      self.rows.insert(
        0,
        ContextDetails::new("none".to_string(), "".to_string(), "no".to_string(), "read".to_string()),
      );
    } else {
      self.rows.insert(
        0,
        ContextDetails::new(
          "none".to_string(),
          "".to_string(),
          "yes".to_string(),
          "read".to_string(),
        ),
      );
    }
    Ok(())
  }
}
