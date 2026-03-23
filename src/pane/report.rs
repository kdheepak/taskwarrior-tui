use std::collections::HashSet;

use crate::table::TaskwarriorTuiTableState;

const NAME: &str = "Name";
const DESCRIPTION: &str = "Description";
const ACTIVE: &str = "Active";

#[derive(Debug, Clone, Default)]
pub struct ReportDetails {
  pub name: String,
  pub description: String,
  pub active: String,
}

impl ReportDetails {
  pub fn new(name: String, description: String, active: String) -> Self {
    Self { name, description, active }
  }
}

pub struct ReportsState {
  pub table_state: TaskwarriorTuiTableState,
  pub columns: Vec<String>,
  pub rows: Vec<ReportDetails>,
  /// Current search query typed by the user inside the popup.
  pub search: String,
}

impl ReportsState {
  pub(crate) fn new() -> Self {
    Self {
      table_state: TaskwarriorTuiTableState::default(),
      columns: vec![NAME.to_string(), DESCRIPTION.to_string(), ACTIVE.to_string()],
      rows: vec![],
      search: String::new(),
    }
  }

  pub fn len(&self) -> usize {
    self.rows.len()
  }

  pub fn is_empty(&self) -> bool {
    self.rows.is_empty()
  }

  /// Returns the indices into `self.rows` that match the current search query.
  /// An empty query matches everything (original order is preserved).
  /// Matching is case-insensitive substring on name or description.
  pub fn filtered_indices(&self) -> Vec<usize> {
    let query = self.search.to_lowercase();
    self
      .rows
      .iter()
      .enumerate()
      .filter(|(_, r)| query.is_empty() || r.name.to_lowercase().contains(&query) || r.description.to_lowercase().contains(&query))
      .map(|(i, _)| i)
      .collect()
  }

  /// Parse the `task show` output for all reports that have columns defined
  /// (i.e. reports that produce tabular task output). Collects their name,
  /// description, and whether they are the currently active report.
  pub fn update_data(&mut self, current_report: &str, data: &str) {
    self.rows.clear();

    // Single pass: collect unique report names (those with a `.columns` entry)
    // and build a description map at the same time.
    let mut seen: HashSet<String> = HashSet::new();
    let mut report_names: Vec<String> = vec![];
    let mut descriptions: std::collections::HashMap<String, String> = std::collections::HashMap::new();

    for line in data.split('\n') {
      if let Some(rest) = line.strip_prefix("report.") {
        // Key has format: "report.<name>.<property>  <value>"
        // We split off the first whitespace to isolate the full key.
        let (key, value) = match rest.split_once(char::is_whitespace) {
          Some(pair) => pair,
          None => continue,
        };
        let value = value.trim();

        if let Some((name, property)) = key.split_once('.') {
          if property == "columns" {
            if seen.insert(name.to_string()) {
              report_names.push(name.to_string());
            }
          } else if property == "description" && !value.is_empty() {
            descriptions.entry(name.to_string()).or_insert_with(|| value.to_string());
          }
        }
      }
    }

    report_names.sort();

    for name in report_names {
      let description = descriptions.remove(&name).unwrap_or_default();
      let active = if name == current_report { "yes" } else { "no" };
      self.rows.push(ReportDetails::new(name, description, active.to_string()));
    }
  }
}
