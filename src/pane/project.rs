use anyhow::{Context as AnyhowContext, Result};

use std::{collections::HashSet, process::Command};

use task_hookrs::project::Project;

use crate::{app::TaskwarriorTui, event::KeyCode, pane::Pane, table::TaskwarriorTuiTableState, utils::Changeset};

pub struct ProjectsState {
  pub(crate) list: Vec<Project>,
  pub table_state: TaskwarriorTuiTableState,
  pub current_selection: usize,
  pub marked: HashSet<Project>,
  pub data: String,
}

impl ProjectsState {
  pub(crate) fn new() -> Self {
    Self {
      list: Vec::default(),
      table_state: TaskwarriorTuiTableState::default(),
      current_selection: 0,
      marked: HashSet::default(),
      data: Default::default(),
    }
  }

  fn pattern_by_marked(app: &mut TaskwarriorTui) -> String {
    let mut project_pattern = String::new();
    if !app.projects.marked.is_empty() {
      for (idx, project) in app.projects.marked.clone().iter().enumerate() {
        let mut input: String = String::from(project);
        if input.as_str() == "(none)" {
          input = " ".to_string();
        }
        if idx == 0 {
          project_pattern = format!("\'(project:{}", input);
        } else {
          project_pattern = format!("{} or project:{}", project_pattern, input);
        }
      }
      project_pattern = format!("{})\'", project_pattern);
    }
    project_pattern
  }

  pub fn toggle_mark(&mut self) {
    if !self.list.is_empty() {
      let selected = self.current_selection;
      if !self.marked.insert(self.list[selected].clone()) {
        self.marked.remove(self.list[selected].as_str());
      }
    }
  }

  pub fn update_data(&mut self, task_exe: &str) -> Result<()> {
    self.list.clear();
    let output = Command::new(task_exe)
      .arg("summary")
      .output()
      .context("Unable to run `task summary`")
      .unwrap();
    let data = String::from_utf8_lossy(&output.stdout);
    self.data = data.into();
    Ok(())
  }

  fn update_table_state(&mut self) {
    self.table_state.select(Some(self.current_selection));
    if self.marked.is_empty() {
      self.table_state.single_selection();
    } else {
      self.table_state.multiple_selection();
      self.table_state.clear();
      for project in &self.marked {
        let index = self.list.iter().position(|x| x == project);
        self.table_state.mark(index);
      }
    }
  }
}

impl Pane for ProjectsState {
  fn handle_input(app: &mut TaskwarriorTui, input: KeyCode) -> Result<()> {
    if input == app.keyconfig.quit || input == KeyCode::Ctrl('c') {
      app.should_quit = true;
    } else if input == app.keyconfig.next_tab {
      Self::change_focus_to_right_pane(app);
    } else if input == app.keyconfig.previous_tab {
      Self::change_focus_to_left_pane(app);
    } else if input == KeyCode::Down || input == app.keyconfig.down {
      self::focus_on_next_project(app);
    } else if input == KeyCode::Up || input == app.keyconfig.up {
      self::focus_on_previous_project(app);
    } else if input == app.keyconfig.select {
      self::update_task_filter_by_selection(app)?;
    }
    app.projects.update_table_state();
    Ok(())
  }
}

fn focus_on_next_project(app: &mut TaskwarriorTui) {
  if app.projects.current_selection < app.projects.list.len().saturating_sub(1) {
    app.projects.current_selection += 1;
    app.projects.table_state.select(Some(app.projects.current_selection));
  }
}

fn focus_on_previous_project(app: &mut TaskwarriorTui) {
  if app.projects.current_selection >= 1 {
    app.projects.current_selection -= 1;
    app.projects.table_state.select(Some(app.projects.current_selection));
  }
}

fn update_task_filter_by_selection(app: &mut TaskwarriorTui) -> Result<()> {
  app.projects.table_state.multiple_selection();
  let last_project_pattern = ProjectsState::pattern_by_marked(app);
  app.projects.toggle_mark();
  let new_project_pattern = ProjectsState::pattern_by_marked(app);
  let current_filter = app.filter.as_str();
  app.filter_history.add(current_filter);

  let mut filter = current_filter.replace(&last_project_pattern, "");
  filter = format!("{}{}", filter, new_project_pattern);
  app.filter.update(filter.as_str(), filter.len(), &mut Changeset::default());
  Ok(())
}
