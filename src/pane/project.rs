use anyhow::Context as AnyhowContext;
use anyhow::{anyhow, Result};
use std::fmt;

const COL_WIDTH: usize = 21;
const PROJECT_HEADER: &str = "Name";
const REMAINING_TASK_HEADER: &str = "Remaining";
const AVG_AGE_HEADER: &str = "Avg age";
const COMPLETE_HEADER: &str = "Complete";

use chrono::{Datelike, Duration, Local, Month, NaiveDate, NaiveDateTime, TimeZone};

use tui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    symbols,
    widgets::{Block, Widget},
};

use crate::action::Action;
use crate::app::{Mode, TaskwarriorTui};
use crate::event::Key;
use crate::pane::Pane;
use crate::table::TableState;
use itertools::Itertools;
use std::cmp::min;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::process::{Command, Output};
use task_hookrs::project::Project;
use uuid::Uuid;

pub struct ProjectsState {
    pub(crate) list: Vec<Project>,
    pub table_state: TableState,
    pub current_selection: usize,
    pub marked: HashSet<Project>,
    pub report_height: u16,
    pub columns: Vec<String>,
    pub rows: Vec<ProjectDetails>,
}

#[derive(Debug, Clone, Default)]
pub struct ProjectDetails {
    name: Project,
    remaining: usize,
    avg_age: String,
    complete: String,
}

impl ProjectsState {
    pub(crate) fn new() -> Self {
        Self {
            list: Vec::default(),
            table_state: TableState::default(),
            current_selection: 0,
            marked: HashSet::default(),
            report_height: 0,
            columns: vec![
                PROJECT_HEADER.to_string(),
                REMAINING_TASK_HEADER.to_string(),
                AVG_AGE_HEADER.to_string(),
                COMPLETE_HEADER.to_string(),
            ],
            rows: vec![],
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

    pub fn simplified_view(&mut self) -> (Vec<Vec<String>>, Vec<String>) {
        let rows = self
            .rows
            .iter()
            .map(|c| {
                vec![
                    c.name.clone(),
                    c.remaining.to_string(),
                    c.avg_age.to_string(),
                    c.complete.clone(),
                ]
            })
            .collect();
        let headers = self.columns.clone();
        (rows, headers)
    }

    pub fn update_data(&mut self) -> Result<()> {
        self.list.clear();
        self.rows.clear();
        let output = Command::new("task")
            .arg("summary")
            .output()
            .context("Unable to run `task summary`")
            .unwrap();
        let data = String::from_utf8_lossy(&output.stdout);

        let lines = data.split('\n').into_iter().skip(1).collect::<Vec<&str>>();

        let header = lines.first().unwrap();

        let contains_avg_age = header.contains("Avg age");

        if contains_avg_age {
            let name_index = header.find("Remaining").unwrap();
            let remaining_index = header.find("Remaining").unwrap() + "Remaining".len();
            let average_age_index = header.find("Avg age").unwrap() + "Avg age".len();
            let complete_index = header.find("Complete").unwrap() + "Complete".len();

            for line in lines.into_iter().skip(2) {
                if line.is_empty() {
                    break;
                }

                let line = line.to_string();
                let name = line[0..name_index].trim().to_string();
                let remaining = line[name_index..remaining_index].trim().parse();
                let remaining = if let Ok(v) = remaining { v } else { 0 };
                let avg_age = line[remaining_index..average_age_index].trim().to_string();
                let complete = line[average_age_index..complete_index].trim().to_string();

                self.rows.push(ProjectDetails {
                    name,
                    remaining,
                    avg_age,
                    complete,
                });
            }
        }

        self.list = self.rows.iter().map(|x| x.name.clone()).collect_vec();
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
    fn handle_input(app: &mut TaskwarriorTui, input: Key) -> Result<()> {
        if input == app.keyconfig.quit || input == Key::Ctrl('c') {
            app.should_quit = true;
        } else if input == app.keyconfig.next_tab {
            Self::change_focus_to_right_pane(app);
        } else if input == app.keyconfig.previous_tab {
            Self::change_focus_to_left_pane(app);
        } else if input == Key::Down || input == app.keyconfig.down {
            self::focus_on_next_project(app);
        } else if input == Key::Up || input == app.keyconfig.up {
            self::focus_on_previous_project(app);
        } else if input == app.keyconfig.select {
            self::update_task_filter_by_selection(app)?;
        }
        app.projects.update_table_state();
        Ok(())
    }
}

fn focus_on_next_project(app: &mut TaskwarriorTui) {
    if app.projects.current_selection < app.projects.list.len() - 1 {
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
    app.filter.update(filter.as_str(), filter.len());
    app.update(true)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    fn test_project_summary() {
        let mut app = TaskwarriorTui::new("next").unwrap();

        app.update(true).unwrap();

        dbg!(&app.projects.rows);
        dbg!(&app.projects.list);
    }
}
