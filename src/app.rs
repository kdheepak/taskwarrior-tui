use serde::{Deserialize, Serialize};
use shlex;
use std::cmp::Ordering;
use std::convert::TryInto;
use std::error::Error;
use std::process::{Command, Stdio};
use std::result::Result;

use task_hookrs::date::Date;
use task_hookrs::import::import;
use task_hookrs::task::Task;
use task_hookrs::uda::UDAValue;
use unicode_width::UnicodeWidthStr;

use chrono::{DateTime, Duration, Local, NaiveDateTime, TimeZone};

use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    terminal::Frame,
    text::{Span, Spans, Text},
    widgets::{BarChart, Block, Borders, Clear, Paragraph, Row, Table, TableState, Wrap},
    Terminal,
};

pub fn cmp(t1: &Task, t2: &Task) -> Ordering {
    let urgency1 = match &t1.uda()["urgency"] {
        UDAValue::Str(_) => 0.0,
        UDAValue::U64(u) => *u as f64,
        UDAValue::F64(f) => *f,
    };
    let urgency2 = match &t2.uda()["urgency"] {
        UDAValue::Str(_) => 0.0,
        UDAValue::U64(u) => *u as f64,
        UDAValue::F64(f) => *f,
    };
    urgency2.partial_cmp(&urgency1).unwrap()
}

pub fn vague_format_date_time(dt: &Date) -> String {
    let now = Local::now().naive_utc();
    let seconds = (now - NaiveDateTime::new(dt.date(), dt.time())).num_seconds();

    if seconds >= 60 * 60 * 24 * 365 {
        return format!("{}y", seconds / 86400 / 365);
    } else if seconds >= 60 * 60 * 24 * 90 {
        return format!("{}mo", seconds / 60 / 60 / 24 / 30);
    } else if seconds >= 60 * 60 * 24 * 14 {
        return format!("{}w", seconds / 60 / 60 / 24 / 7);
    } else if seconds >= 60 * 60 * 24 {
        return format!("{}d", seconds / 60 / 60 / 24);
    } else if seconds >= 60 * 60 {
        return format!("{}h", seconds / 60 / 60);
    } else if seconds >= 60 {
        return format!("{}min", seconds / 60);
    } else {
        return format!("{}s", seconds);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

pub enum AppMode {
    Report,
    Filter,
    AddTask,
    LogTask,
    ModifyTask,
    HelpPopup,
    TaskError,
}

pub struct TaskwarriorTUIApp {
    pub should_quit: bool,
    pub state: TableState,
    pub cursor_location: usize,
    pub filter: String,
    pub command: String,
    pub error: String,
    pub modify: String,
    pub tasks: Vec<Task>,
    pub task_report_labels: Vec<String>,
    pub task_report_columns: Vec<String>,
    pub mode: AppMode,
}

impl TaskwarriorTUIApp {
    pub fn new() -> TaskwarriorTUIApp {
        let mut app = TaskwarriorTUIApp {
            should_quit: false,
            state: TableState::default(),
            tasks: vec![],
            task_report_labels: vec![],
            task_report_columns: vec![],
            filter: "status:pending ".to_string(),
            cursor_location: 0,
            command: "".to_string(),
            modify: "".to_string(),
            error: "".to_string(),
            mode: AppMode::Report,
        };
        app.update();
        app
    }

    pub fn draw(&mut self, f: &mut Frame<impl Backend>) {
        while self.tasks.len() != 0 && self.state.selected().unwrap_or_default() >= self.tasks.len()
        {
            self.previous();
        }
        let rects = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
            .split(f.size());
        let task_rects = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(rects[0]);
        self.draw_task_report(f, task_rects[0]);
        self.draw_task_details(f, task_rects[1]);
        match self.mode {
            AppMode::Report => self.draw_command(f, rects[1], &self.filter[..], "Filter Tasks"),
            AppMode::Filter => {
                f.render_widget(Clear, rects[1]);
                f.set_cursor(rects[1].x + self.cursor_location as u16 + 1, rects[1].y + 1);
                self.draw_command(f, rects[1], &self.filter[..], "Filter Tasks");
            }
            AppMode::ModifyTask => {
                f.set_cursor(rects[1].x + self.cursor_location as u16 + 1, rects[1].y + 1);
                f.render_widget(Clear, rects[1]);
                self.draw_command(f, rects[1], &self.modify[..], "Modify Task");
            }
            AppMode::LogTask => {
                f.set_cursor(rects[1].x + self.cursor_location as u16 + 1, rects[1].y + 1);
                f.render_widget(Clear, rects[1]);
                self.draw_command(f, rects[1], &self.command[..], "Log Task");
            }
            AppMode::AddTask => {
                f.set_cursor(rects[1].x + self.cursor_location as u16 + 1, rects[1].y + 1);
                f.render_widget(Clear, rects[1]);
                self.draw_command(f, rects[1], &self.command[..], "Add Task");
            }
            AppMode::TaskError => {
                f.render_widget(Clear, rects[1]);
                self.draw_command(f, rects[1], &self.error[..], "Error");
            }
            AppMode::HelpPopup => {
                self.draw_command(f, rects[1], &self.filter[..], "Filter Tasks");
                self.draw_help_popup(f, f.size());
            }
        }
    }

    fn draw_help_popup(&self, f: &mut Frame<impl Backend>, rect: Rect) {
        let text = vec![
            Spans::from(""),
            Spans::from(vec![
                Span::from("    /"),
                Span::from("    "),
                Span::from(Span::styled(
                    "task {string}              ",
                    Style::default().add_modifier(Modifier::BOLD),
                )),
                Span::from("    "),
                Span::from("- Filter task report"),
            ]),
            Spans::from(""),
            Spans::from(vec![
                Span::from("    a"),
                Span::from("    "),
                Span::from(Span::styled(
                    "task add {string}          ",
                    Style::default().add_modifier(Modifier::BOLD),
                )),
                Span::from("    "),
                Span::from("- Add new task"),
            ]),
            Spans::from(""),
            Spans::from(vec![
                Span::from("    d"),
                Span::from("    "),
                Span::from(Span::styled(
                    "task done {selected}       ",
                    Style::default().add_modifier(Modifier::BOLD),
                )),
                Span::from("    "),
                Span::from("- Mark task as done"),
            ]),
            Spans::from(""),
            Spans::from(vec![
                Span::from("    e"),
                Span::from("    "),
                Span::from(Span::styled(
                    "task edit {selected}       ",
                    Style::default().add_modifier(Modifier::BOLD),
                )),
                Span::from("    "),
                Span::from("- Open selected task in editor"),
            ]),
            Spans::from(""),
            Spans::from(vec![
                Span::from("    j"),
                Span::from("    "),
                Span::from(Span::styled(
                    "{selected+=1}              ",
                    Style::default().add_modifier(Modifier::BOLD),
                )),
                Span::from("    "),
                Span::from("- Move down in task report"),
            ]),
            Spans::from(""),
            Spans::from(vec![
                Span::from("    k"),
                Span::from("    "),
                Span::from(Span::styled(
                    "{selected-=1}              ",
                    Style::default().add_modifier(Modifier::BOLD),
                )),
                Span::from("    "),
                Span::from("- Move up in task report"),
            ]),
            Spans::from(""),
            Spans::from(vec![
                Span::from("    l"),
                Span::from("    "),
                Span::from(Span::styled(
                    "task log {string}          ",
                    Style::default().add_modifier(Modifier::BOLD),
                )),
                Span::from("    "),
                Span::from("- Log new task"),
            ]),
            Spans::from(""),
            Spans::from(vec![
                Span::from("    m"),
                Span::from("    "),
                Span::from(Span::styled(
                    "task modify {string}       ",
                    Style::default().add_modifier(Modifier::BOLD),
                )),
                Span::from("    "),
                Span::from("- Modify selected task"),
            ]),
            Spans::from(""),
            Spans::from(vec![
                Span::from("    q"),
                Span::from("    "),
                Span::from(Span::styled(
                    "exit                       ",
                    Style::default().add_modifier(Modifier::BOLD),
                )),
                Span::from("    "),
                Span::from("- Quit"),
            ]),
            Spans::from(""),
            Spans::from(vec![
                Span::from("    s"),
                Span::from("    "),
                Span::from(Span::styled(
                    "task start/stop {selected} ",
                    Style::default().add_modifier(Modifier::BOLD),
                )),
                Span::from("    "),
                Span::from("- Toggle start and stop"),
            ]),
            Spans::from(""),
            Spans::from(vec![
                Span::from("    u"),
                Span::from("    "),
                Span::from(Span::styled(
                    "task undo                  ",
                    Style::default().add_modifier(Modifier::BOLD),
                )),
                Span::from("    "),
                Span::from("- Undo"),
            ]),
            Spans::from(""),
            Spans::from(vec![
                Span::from("    ?"),
                Span::from("    "),
                Span::from(Span::styled(
                    "help                       ",
                    Style::default().add_modifier(Modifier::BOLD),
                )),
                Span::from("    "),
                Span::from("- Show this help menu"),
            ]),
            Spans::from(""),
        ];
        let paragraph = Paragraph::new(text)
            .block(Block::default().title("Help").borders(Borders::ALL))
            .alignment(Alignment::Left);
        let area = centered_rect(80, 90, rect);
        f.render_widget(Clear, area);
        f.render_widget(paragraph, area);
    }

    fn draw_command(&self, f: &mut Frame<impl Backend>, rect: Rect, text: &str, title: &str) {
        let p = Paragraph::new(Text::from(text))
            .block(Block::default().borders(Borders::ALL).title(title));
        f.render_widget(p, rect);
    }

    fn draw_task_details(&mut self, f: &mut Frame<impl Backend>, rect: Rect) {
        if self.tasks.len() == 0 {
            f.render_widget(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Task not found"),
                rect,
            );
            return ();
        }
        let selected = self.state.selected().unwrap_or_default();
        let task_id = self.tasks[selected].id().unwrap_or_default();
        let output = Command::new("task").arg(format!("{}", task_id)).output();
        match output {
            Ok(output) => {
                let data = String::from_utf8(output.stdout).unwrap();
                let p = Paragraph::new(Text::from(&data[..])).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(format!("Task {}", task_id)),
                );
                f.render_widget(p, rect);
            }
            Err(_) => (),
        }
    }

    fn draw_task_report(&mut self, f: &mut Frame<impl Backend>, rect: Rect) {
        let active_style = Style::default()
            .fg(Color::Blue)
            .add_modifier(Modifier::BOLD);
        let normal_style = Style::default();

        let (tasks, headers, widths) = self.task_report();
        if tasks.len() == 0 {
            f.render_widget(
                Block::default().borders(Borders::ALL).title("Task next"),
                rect,
            );
            return ();
        }
        let header = headers.iter();
        let rows = tasks
            .iter()
            .map(|i| Row::StyledData(i.iter(), normal_style));
        let constraints: Vec<Constraint> = widths
            .iter()
            .map(|i| {
                Constraint::Percentage(std::cmp::min(70, std::cmp::max(*i, 5)).try_into().unwrap())
            })
            .collect();

        let t = Table::new(header, rows)
            .block(Block::default().borders(Borders::ALL).title("Task next"))
            .highlight_style(normal_style.add_modifier(Modifier::BOLD))
            .highlight_symbol("• ")
            .widths(&constraints);
        f.render_stateful_widget(t, rect, &mut self.state);
    }

    pub fn get_string_attribute(&self, attribute: &str, task: &Task) -> String {
        let s = match attribute {
            "id" => task.id().unwrap_or_default().to_string(),
            // "entry" => task.entry().unwrap().to_string(),
            "entry" => vague_format_date_time(task.entry()),
            "start" => match task.start() {
                Some(v) => vague_format_date_time(v),
                None => "".to_string(),
            },
            "description" => task.description().to_string(),
            "urgency" => match &task.uda()["urgency"] {
                UDAValue::Str(s) => "0.0".to_string(),
                UDAValue::U64(u) => (*u as f64).to_string(),
                UDAValue::F64(f) => (*f).to_string(),
            },
            _ => "".to_string(),
        };
        return s;
    }

    pub fn task_report(&mut self) -> (Vec<Vec<String>>, Vec<String>, Vec<i16>) {
        let mut alltasks = vec![];
        // get all tasks as their string representation
        for task in &self.tasks {
            let mut item = vec![];
            for name in &self.task_report_columns {
                let attribute = name.split(".").collect::<Vec<&str>>()[0];
                let s = self.get_string_attribute(attribute, task);
                item.push(s);
            }
            alltasks.push(item)
        }

        // find which columns are empty
        let null_columns_len;
        if alltasks.len() > 0 {
            null_columns_len = alltasks[0].len();
        } else {
            return (vec![], vec![], vec![]);
        }

        let mut null_columns = vec![0; null_columns_len];
        for task in &alltasks {
            for (i, s) in task.iter().enumerate() {
                null_columns[i] += s.len();
            }
        }

        // filter out columns where everything is empty
        let mut tasks = vec![];
        for task in &alltasks {
            let t = task.clone();
            let t: Vec<String> = t
                .iter()
                .enumerate()
                .filter(|&(i, _)| null_columns[i] != 0)
                .map(|(_, e)| e.to_owned())
                .collect();
            tasks.push(t);
        }

        // filter out header where all columns are empty
        let headers: Vec<String> = self
            .task_report_labels
            .iter()
            .enumerate()
            .filter(|&(i, _)| null_columns[i] != 0)
            .map(|(_, e)| e.to_owned())
            .collect();

        // set widths proportional to the content
        let mut widths: Vec<i16> = vec![0; tasks[0].len()];
        for task in &tasks {
            for (i, attr) in task.iter().enumerate() {
                widths[i] = (attr.len() as i16 * 100
                    / task.iter().map(|s| s.len() as i16).sum::<i16>())
                .try_into()
                .unwrap();
            }
        }

        return (tasks, headers, widths);
    }

    pub fn update(&mut self) {
        self.export_tasks();
        self.export_headers();
    }
    pub fn next(&mut self) {
        if self.tasks.len() == 0 {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.tasks.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
    pub fn previous(&mut self) {
        if self.tasks.len() == 0 {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.tasks.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn export_headers(&mut self) {
        self.task_report_columns = vec![];
        self.task_report_labels = vec![];

        let output = Command::new("task")
            .arg("show")
            .arg("report.next.columns")
            .output()
            .expect("Unable to run `task show report.next.columns`. Check documentation for more information");
        let data = String::from_utf8(output.stdout).unwrap();

        for line in data.split("\n") {
            if line.starts_with("report.next.columns") {
                let column_names: &str = line.split(" ").collect::<Vec<&str>>()[1];
                for column in column_names.split(",") {
                    self.task_report_columns.push(column.to_string());
                }
            }
        }

        let output = Command::new("task")
            .arg("show")
            .arg("report.next.labels")
            .output()
            .expect("Unable to run `task show report.next.labels`. Check documentation for more information");
        let data = String::from_utf8(output.stdout).unwrap();

        for line in data.split("\n") {
            if line.starts_with("report.next.labels") {
                let label_names: &str = line.split(" ").collect::<Vec<&str>>()[1];
                for label in label_names.split(",") {
                    self.task_report_labels.push(label.to_string());
                }
            }
        }
    }

    pub fn export_tasks(&mut self) {
        let mut task = Command::new("task");

        task.arg("rc.json.array=on");
        task.arg("export");

        match shlex::split(&self.filter) {
            Some(cmd) => {
                for s in cmd {
                    task.arg(&s);
                }
            }
            None => {
                task.arg("");
            }
        }

        let output = task
            .output()
            .expect("Unable to run `task export`. Check documentation for more information");
        let data = String::from_utf8(output.stdout).unwrap();
        let imported = import(data.as_bytes());
        match imported {
            Ok(i) => {
                self.tasks = i;
                self.tasks.sort_by(cmp);
            }
            _ => (),
        }
    }

    pub fn handle_tick(&mut self) {
        self.update();
    }

    pub fn task_log(&mut self) -> Result<(), String> {
        if self.tasks.len() == 0 {
            return Ok(());
        }

        let mut command = Command::new("task");

        command.arg("log");

        match shlex::split(&self.command) {
            Some(cmd) => {
                for s in cmd {
                    command.arg(&s);
                }
                let output = command
                    .output()
                    .expect("Cannot run `task log`. Check documentation for more information");

                self.command = "".to_string();
                return Ok(());
            }
            None => {
                return Err(format!("Unable to run `task log` with `{}`", &self.command));
            }
        }
    }

    pub fn task_modify(&mut self) -> Result<(), String> {
        if self.tasks.len() == 0 {
            return Ok(());
        }
        let selected = self.state.selected().unwrap_or_default();
        let task_id = self.tasks[selected].id().unwrap_or_default();
        let mut command = Command::new("task");
        command.arg(format!("{}", task_id)).arg("modify");

        match shlex::split(&self.modify) {
            Some(cmd) => {
                for s in cmd {
                    command.arg(&s);
                }
                let output = command
                    .output()
                    .expect("Cannot run `task modify`. Check documentation for more information");

                self.modify = "".to_string();
                return Ok(());
            }
            None => {
                return Err(format!(
                    "Unable to run `task modify` with `{}` on task {}",
                    &self.modify, &task_id
                ));
            }
        }
    }

    pub fn task_add(&mut self) -> Result<(), String> {
        if self.tasks.len() == 0 {
            return Ok(());
        }

        let mut command = Command::new("task");
        command.arg("add");

        match shlex::split(&self.command) {
            Some(cmd) => {
                for s in cmd {
                    command.arg(&s);
                }
                let output = command
                    .output()
                    .expect("Cannot run `task add`. Check documentation for more information");

                self.command = "".to_string();
                return Ok(());
            }
            None => {
                return Err(format!("Unable to run `task add` with `{}`", &self.command));
            }
        }
    }

    pub fn task_virtual_tags(&self) -> Result<String, String> {
        let selected = self.state.selected().unwrap_or_default();
        let task_id = self.tasks[selected].id().unwrap_or_default();
        let output = Command::new("task").arg(format!("{}", task_id)).output();

        match output {
            Ok(output) => {
                let data = String::from_utf8(output.stdout).unwrap();
                for line in data.split("\n") {
                    if line.starts_with("Virtual tags") {
                        let line = line.to_string();
                        let line = line.replace("Virtual tags", "");
                        return Ok(line);
                    }
                }
                return Err(format!(
                    "Cannot find any tags for `task {}`. Check documentation for more information",
                    task_id
                ));
            }
            Err(_) => Err(format!(
                "Cannot run `task {}`. Check documentation for more information",
                task_id
            )),
        }
    }

    pub fn task_start_or_stop(&mut self) -> Result<(), String> {
        if self.tasks.len() == 0 {
            return Ok(());
        }
        let selected = self.state.selected().unwrap_or_default();
        let task_id = self.tasks[selected].id().unwrap_or_default();
        let mut command = "start";
        for tag in self.task_virtual_tags()?.split(" ") {
            if tag == "ACTIVE" {
                command = "stop"
            }
        }

        let output = Command::new("task")
            .arg(format!("{}", task_id))
            .arg(command)
            .output();
        match output {
            Ok(_) => Ok(()),
            Err(_) => Err(format!(
                "Cannot run `task done` for task `{}`. Check documentation for more information",
                task_id
            )),
        }
    }

    pub fn task_delete(&self) -> Result<(), String> {
        if self.tasks.len() == 0 {
            return Ok(());
        }
        let selected = self.state.selected().unwrap_or_default();
        let task_id = self.tasks[selected].id().unwrap_or_default();

        let output = Command::new("task")
            .arg("rc.confirmation=off")
            .arg(format!("{}", task_id))
            .arg("delete")
            .output();
        match output {
            Ok(_) => Ok(()),
            Err(_) => Err(format!(
                "Cannot run `task delete` for task `{}`. Check documentation for more information",
                task_id
            )),
        }
    }

    pub fn task_done(&mut self) -> Result<(), String> {
        if self.tasks.len() == 0 {
            return Ok(());
        }
        let selected = self.state.selected().unwrap_or_default();
        let task_id = self.tasks[selected].id().unwrap_or_default();
        let output = Command::new("task")
            .arg(format!("{}", task_id))
            .arg("done")
            .output();
        match output {
            Ok(_) => Ok(()),
            Err(_) => Err(format!(
                "Cannot run `task done` for task `{}`. Check documentation for more information",
                task_id
            )),
        }
    }

    pub fn task_undo(&self) -> Result<(), String> {
        if self.tasks.len() == 0 {
            return Ok(());
        }
        let output = Command::new("task")
            .arg("rc.confirmation=off")
            .arg("undo")
            .output();

        match output {
            Ok(_) => Ok(()),
            Err(_) => {
                Err("Cannot run `task undo`. Check documentation for more information".to_string())
            }
        }
    }

    pub fn task_edit(&self) -> Result<(), String> {
        if self.tasks.len() == 0 {
            return Ok(());
        }
        let selected = self.state.selected().unwrap_or_default();
        let task_id = self.tasks[selected].id().unwrap_or_default();
        let r = Command::new("task")
            .arg(format!("{}", task_id))
            .arg("edit")
            .spawn();

        match r {
            Ok(child) => {
                let output = child.wait_with_output();
                match output {
                    Ok(output) => {
                        if !output.status.success() {
                            return Err(format!(
                                "`task edit` for task `{}` failed. {}{}",
                                task_id,
                                String::from_utf8(output.stdout).unwrap(),
                                String::from_utf8(output.stderr).unwrap()
                            ));
                        } else {
                            return Ok(());
                        }
                    }
                    Err(err) => Err(format!(
                        "Cannot run `task edit` for task `{}`. {}",
                        task_id, err
                    )),
                }
            }
            _ => Err(format!(
                "Cannot start `task edit` for task `{}`. Check documentation for more information",
                task_id
            )),
        }
    }

    pub fn task_current(&self) -> Option<Task> {
        if self.tasks.len() == 0 {
            return None;
        }
        let selected = self.state.selected().unwrap_or_default();
        return Some(self.tasks[selected].clone());
    }
}

#[cfg(test)]
mod tests {
    use crate::app::TaskwarriorTUIApp;
    use crate::util::setup_terminal;
    use std::io::stdin;

    use task_hookrs::import::import;
    use task_hookrs::task::Task;

    #[test]
    fn test_app() {
        let mut app = TaskwarriorTUIApp::new();
        app.update();

        //println!("{:?}", app.tasks);

        //println!("{:?}", app.task_report_columns);
        //println!("{:?}", app.task_report_labels);

        let (t, h, c) = app.task_report();
        app.next();
        app.next();
        app.modify = "Cannot add this string ' because it has a single quote".to_string();
        println!("{}", app.modify);
        // if let Ok(tasks) = import(stdin()) {
        //     for task in tasks {
        //         println!("Task: {}, entered {:?} is {} -> {}",
        //                   task.uuid(),
        //                   task.entry(),
        //                   task.status(),
        //                   task.description());
        //     }
        // }
    }
}
