// Based on https://gist.github.com/diwic/5c20a283ca3a03752e1a27b0f3ebfa30
// See https://old.reddit.com/r/rust/comments/4xneq5/the_calendar_example_challenge_ii_why_eddyb_all/

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

use crate::app::{Action, Mode, TaskwarriorTui};
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
                if idx > 0 {
                    project_pattern = format!("{} or project:{}", project_pattern, input);
                } else {
                    project_pattern = format!("\'(project:{}", input);
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
        for line in data.split('\n').into_iter().skip(3) {
            if line.is_empty() {
                break;
            }
            let row: Vec<String> = line
                .split(' ')
                .map(str::trim)
                .map(str::trim_start)
                .filter(|x| !x.is_empty())
                .map(ToString::to_string)
                .collect();

            self.rows.push(ProjectDetails {
                name: (&row[0]).parse()?,
                remaining: (&row[1]).parse()?,
                avg_age: (&row[2]).parse()?,
                complete: (&row[3]).parse()?,
            });
        }
        self.list = self.rows.iter().map(|x| x.name.clone()).collect_vec();
        Ok(())
    }
    fn update_table_state(&mut self) {
        self.table_state.select(Some(self.current_selection));
        if self.marked.is_empty() {
            self.table_state.single_selection();
        }
        self.table_state.clear();
        for project in &self.marked {
            let index = self.list.iter().position(|x| x == project);
            self.table_state.mark(index);
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
    app.filter_history_context.add(current_filter);

    let mut filter = current_filter.replace(&last_project_pattern, "");
    filter = format!("{}{}", filter, new_project_pattern);
    app.filter.update(filter.as_str(), filter.len());
    app.update(true)?;
    Ok(())
}
