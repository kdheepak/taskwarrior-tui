//! Column visibility menu.
//!
//! A popup that lists the columns of the current report and lets the user
//! toggle each one on or off, so the report can be decluttered without
//! editing the taskwarrior report definition. Hidden columns are remembered
//! per report and persisted across sessions in the taskwarrior-tui data
//! directory (`hidden-columns` file, one `report<TAB>col1,col2` line each).

use std::{
  collections::{HashMap, HashSet},
  fs,
  path::PathBuf,
};

use ratatui::{
  Frame,
  layout::Rect,
  style::{Modifier, Style},
  text::{Line, Span},
  widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState},
};

pub struct ColumnItem {
  pub column: String,
  pub label: String,
  pub visible: bool,
}

pub struct ColumnMenu {
  pub items: Vec<ColumnItem>,
  pub selection: usize,
  /// report name -> set of hidden column names
  hidden: HashMap<String, HashSet<String>>,
  path: Option<PathBuf>,
}

impl ColumnMenu {
  pub fn new() -> Self {
    let path = Self::storage_path();
    let hidden = path.as_deref().map(Self::load).unwrap_or_default();
    Self {
      items: vec![],
      selection: 0,
      hidden,
      path,
    }
  }

  fn storage_path() -> Option<PathBuf> {
    let dir = if let Ok(s) = std::env::var("TASKWARRIOR_TUI_DATA") {
      PathBuf::from(s)
    } else {
      dirs::data_local_dir()?.join("taskwarrior-tui")
    };
    fs::create_dir_all(&dir).ok()?;
    Some(dir.join("hidden-columns"))
  }

  fn load(path: &std::path::Path) -> HashMap<String, HashSet<String>> {
    let mut hidden = HashMap::new();
    if let Ok(data) = fs::read_to_string(path) {
      for line in data.lines() {
        if let Some((report, cols)) = line.split_once('\t') {
          let set: HashSet<String> = cols.split(',').filter(|s| !s.is_empty()).map(str::to_string).collect();
          if !set.is_empty() {
            hidden.insert(report.to_string(), set);
          }
        }
      }
    }
    hidden
  }

  fn save(&self) {
    if let Some(path) = &self.path {
      let mut lines: Vec<String> = self
        .hidden
        .iter()
        .filter(|(_, set)| !set.is_empty())
        .map(|(report, set)| {
          let mut cols: Vec<&str> = set.iter().map(String::as_str).collect();
          cols.sort_unstable();
          format!("{}\t{}", report, cols.join(","))
        })
        .collect();
      lines.sort_unstable();
      let _ = fs::write(path, lines.join("\n") + "\n");
    }
  }

  /// Refresh the menu entries from the current report definition, keeping
  /// the persisted hidden state. Call when the menu is opened.
  pub fn sync(&mut self, report: &str, columns: &[String], labels: &[String]) {
    let hidden = self.hidden.get(report).cloned().unwrap_or_default();
    self.items = columns
      .iter()
      .enumerate()
      .map(|(i, c)| ColumnItem {
        column: c.clone(),
        label: labels.get(i).cloned().unwrap_or_else(|| c.clone()),
        visible: !hidden.contains(c),
      })
      .collect();
    if self.selection >= self.items.len() {
      self.selection = self.items.len().saturating_sub(1);
    }
  }

  pub fn hidden_for(&self, report: &str) -> HashSet<String> {
    self.hidden.get(report).cloned().unwrap_or_default()
  }

  pub fn next(&mut self) {
    if !self.items.is_empty() {
      self.selection = (self.selection + 1) % self.items.len();
    }
  }

  pub fn previous(&mut self) {
    if !self.items.is_empty() {
      self.selection = self.selection.checked_sub(1).unwrap_or(self.items.len() - 1);
    }
  }

  /// Toggle the selected column for the given report and persist the change.
  pub fn toggle_current(&mut self, report: &str) {
    if let Some(item) = self.items.get_mut(self.selection) {
      item.visible = !item.visible;
      let set = self.hidden.entry(report.to_string()).or_default();
      if item.visible {
        set.remove(&item.column);
      } else {
        set.insert(item.column.clone());
      }
      if set.is_empty() {
        self.hidden.remove(report);
      }
      self.save();
    }
  }

  /// Show every column of the given report again and persist the change.
  pub fn reset(&mut self, report: &str) {
    for item in &mut self.items {
      item.visible = true;
    }
    self.hidden.remove(report);
    self.save();
  }

  pub fn draw(&self, f: &mut Frame, area: Rect, highlight_style: Style) {
    f.render_widget(Clear, area);
    let items: Vec<ListItem> = self
      .items
      .iter()
      .map(|item| {
        let marker = if item.visible { "[x]" } else { "[ ]" };
        let mut style = Style::default();
        if !item.visible {
          style = style.add_modifier(Modifier::DIM);
        }
        ListItem::new(Line::from(vec![
          Span::styled(format!(" {} ", marker), style),
          Span::styled(item.label.clone(), style),
          Span::styled(format!("  ({})", item.column), style.add_modifier(Modifier::DIM)),
        ]))
      })
      .collect();

    let mut state = ListState::default();
    state.select(Some(self.selection));

    let list = List::new(items)
      .block(
        Block::default()
          .borders(Borders::ALL)
          .border_type(BorderType::Rounded)
          .title(Span::styled(
            "Columns  (space/enter toggle, a show all, esc close)",
            Style::default().add_modifier(Modifier::BOLD),
          )),
      )
      .highlight_style(highlight_style.add_modifier(Modifier::BOLD));

    f.render_stateful_widget(list, area, &mut state);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn menu_with(columns: &[&str]) -> ColumnMenu {
    let mut m = ColumnMenu {
      items: vec![],
      selection: 0,
      hidden: HashMap::new(),
      path: None,
    };
    let cols: Vec<String> = columns.iter().map(|s| s.to_string()).collect();
    let labels = cols.clone();
    m.sync("next", &cols, &labels);
    m
  }

  #[test]
  fn toggle_hides_and_restores() {
    let mut m = menu_with(&["id", "urgency", "description"]);
    m.selection = 1;
    m.toggle_current("next");
    assert!(m.hidden_for("next").contains("urgency"));
    m.toggle_current("next");
    assert!(m.hidden_for("next").is_empty());
  }

  #[test]
  fn hidden_state_is_per_report() {
    let mut m = menu_with(&["id", "urgency"]);
    m.selection = 1;
    m.toggle_current("next");
    assert!(m.hidden_for("list").is_empty());
  }

  #[test]
  fn reset_shows_all() {
    let mut m = menu_with(&["id", "urgency"]);
    m.toggle_current("next");
    m.reset("next");
    assert!(m.hidden_for("next").is_empty());
    assert!(m.items.iter().all(|i| i.visible));
  }

  #[test]
  fn selection_wraps() {
    let mut m = menu_with(&["a", "b"]);
    m.previous();
    assert_eq!(m.selection, 1);
    m.next();
    assert_eq!(m.selection, 0);
  }
}
