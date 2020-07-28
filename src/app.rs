use serde::{Deserialize, Serialize};
use serde_json::Result;
use shlex::split;
use std::cmp::Ordering;
use std::convert::TryInto;
use std::process::Command;

use task_hookrs::date::Date;
use task_hookrs::import::import;
use task_hookrs::task::Task;
use task_hookrs::uda::UDAValue;
use unicode_width::UnicodeWidthStr;

use chrono::{DateTime, Duration, Local, NaiveDateTime, TimeZone};

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    terminal::Frame,
    text::Text,
    widgets::{BarChart, Clear, Block, Borders, Paragraph, Row, Table, TableState},
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
}

pub struct App {
    pub should_quit: bool,
    pub state: TableState,
    pub filter: String,
    pub command: String,
    pub tasks: Vec<Task>,
    pub task_report_labels: Vec<String>,
    pub task_report_columns: Vec<String>,
    pub mode: AppMode,
}

impl App {
    pub fn new() -> App {
        let mut app = App {
            should_quit: false,
            state: TableState::default(),
            tasks: vec![],
            task_report_labels: vec![],
            task_report_columns: vec![],
            filter: "status:pending ".to_string(),
            command: "".to_string(),
            mode: AppMode::Report,
        };
        app.update();
        app
    }

    pub fn draw(&mut self, f: &mut Frame<impl Backend>) {
        while self.tasks.len() != 0 && self.state.selected().unwrap_or_default() >= self.tasks.len() {
            self.previous();
        }
        let rects = Layout::default()
            .constraints(
                [
                    Constraint::Percentage(48),
                    Constraint::Percentage(48),
                    Constraint::Max(3),
                ]
                .as_ref(),
            )
            .split(f.size());
        self.draw_task_report(f, rects[0]);
        self.draw_task_details(f, rects[1]);
        match self.mode {
            AppMode::Report => self.draw_command(f, rects[2], &self.filter[..], "Filter"),
            AppMode::Filter => {
                f.render_widget(Clear, rects[2]);
                f.set_cursor(
                    rects[2].x + self.filter.width() as u16 + 1,
                    rects[2].y + 1,
                );
                self.draw_command(f, rects[2], &self.filter[..], "Filter");
            },
            AppMode::AddTask => {
                f.set_cursor(
                    // Put cursor past the end of the input text
                    rects[2].x + self.command.width() as u16 + 1,
                    // Move one line down, from the border to the input line
                    rects[2].y + 1,
                );
                f.render_widget(Clear, rects[2]);
                self.draw_command(f, rects[2], &self.command[..], "Add Task");
            },
        }
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
        let output = Command::new("task")
            .arg(format!("{}", task_id))
            .output()
            .expect(
                &format!(
                "Unable to show details for `task {}`. Check documentation for more information",
                task_id
                )[..],
            );
        let data = String::from_utf8(output.stdout).unwrap();
        let p = Paragraph::new(Text::from(&data[..])).block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Task {}", task_id)),
        );
        f.render_widget(p, rect);
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
                Constraint::Percentage(std::cmp::min(50, std::cmp::max(*i, 5)).try_into().unwrap())
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

        match split(&self.filter) {
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

    pub fn task_add(&mut self) {
        if self.tasks.len() == 0 {
            return
        }

        let output = Command::new("task")
            .arg("add")
            .arg(format!("{}", self.command))
            .output()
            .expect("Cannot run `task add`. Check documentation for more information");

        self.command = "".to_string();
    }

    pub fn task_virtual_tags(& self) -> String {
        let selected = self.state.selected().unwrap_or_default();
        let task_id = self.tasks[selected].id().unwrap_or_default();
        let output = Command::new("task")
            .arg(format!("{}", task_id))
            .output()
            .expect(
                &format!(
                "Cannot run `task {}`. Check documentation for more information",
                task_id
                )[..],
            );
        let data = String::from_utf8(output.stdout).unwrap();
        for line in data.split("\n") {
            if line.starts_with("Virtual tags") {
                let line = line.to_string();
                let line = line.replace("Virtual tags", "");
                return line;
            }
        }
        "".to_string()
    }

    pub fn task_start_or_stop(&mut self) {
        if self.tasks.len() == 0 {
            return
        }
        let selected = self.state.selected().unwrap_or_default();
        let task_id = self.tasks[selected].id().unwrap_or_default();
        let mut command = "start";
        for tag in self.task_virtual_tags().split(" ") {
            if tag == "ACTIVE" {
                command = "stop"
            }
        }

        let output = Command::new("task")
            .arg(command)
            .arg(format!("{}", task_id))
            .output()
            .expect(
                &format!(
                "Cannot run `task done` for task `{}`. Check documentation for more information",
                task_id
                )[..],
            );
    }

    pub fn task_delete(&self) {
        if self.tasks.len() == 0 {
            return
        }
        let selected = self.state.selected().unwrap_or_default();
        let task_id = self.tasks[selected].id().unwrap_or_default();

        let output = Command::new("task")
            .arg("rc.confirmation=off")
            .arg("delete")
            .arg(format!("{}", task_id))
            .output()
            .expect(
                &format!(
                "Cannot run `task delete` for task `{}`. Check documentation for more information",
                task_id
                )[..],
            );
    }

    pub fn task_done(&mut self) {
        if self.tasks.len() == 0 {
            return
        }
        let selected = self.state.selected().unwrap_or_default();
        let task_id = self.tasks[selected].id().unwrap_or_default();
        let output = Command::new("task")
            .arg("done")
            .arg(format!("{}", task_id))
            .output()
            .expect(
                &format!(
                "Cannot run `task done` for task `{}`. Check documentation for more information",
                task_id
                )[..],
            );
    }

    pub fn task_undo(&self) {
        if self.tasks.len() == 0 {
            return
        }
        let output = Command::new("task")
            .arg("rc.confirmation=off")
            .arg("undo")
            .output()
            .expect("Cannot run `task undo`. Check documentation for more information");
    }

    pub fn task_edit(&self) {
        if self.tasks.len() == 0 {
            return
        }
        let selected = self.state.selected().unwrap_or_default();
        let task_id = self.tasks[selected].id().unwrap_or_default();
        let r = Command::new("task")
            .arg("edit")
            .arg(format!("{}", task_id))
            .spawn();

        // TODO: fix vim hanging
        match r {
            Ok(child) => {
                let output = child
                    .wait_with_output()
                    .expect(
                        &format!(
                        "Cannot run `task edit` for task `{}`. Check documentation for more information",
                        task_id
                        )[..],
                    );
                if !output.status.success() {
                    // TODO: show error message here
                }
            }
            _ => {
                println!("Vim failed to start");
            }
        }
    }

}

#[cfg(test)]
mod tests {
    use crate::app::App;
    use crate::util::setup_terminal;
    use std::io::stdin;

    use task_hookrs::import::import;
    use task_hookrs::task::Task;

    #[test]
    fn test_app() {
        let mut app = App::new();
        app.update();

        //println!("{:?}", app.tasks);

        //println!("{:?}", app.task_report_columns);
        //println!("{:?}", app.task_report_labels);

        let (t, h, c) = app.task_report();
        app.next();
        app.next();
        let selected = app.state.selected().unwrap_or_default();
        let task_id = app.tasks[selected].id().unwrap_or_default();
        let mut command = "start";
        for tag in app.tasks[selected].tags().unwrap_or(&vec![]) {
            if tag == "ACTIVE" {
                command = "stop"
            }
        }
        println!("{:?}", app.tasks[selected]);
        println!("{:?}", app.tasks[selected].tags().unwrap_or(&vec![]));
        println!("{}", app.task_virtual_tags());
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
