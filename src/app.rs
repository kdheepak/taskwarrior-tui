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
    backend::{Backend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    terminal::Frame,
    text::Text,
    widgets::{BarChart, Block, Borders, Paragraph, Row, Table, TableState},
    Terminal,
};

use crate::util::{Key};

use crossterm::event::KeyCode;

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

pub enum InputMode {
    Normal,
    Command,
}

pub struct App {
    pub should_quit: bool,
    pub state: TableState,
    pub filter: String,
    pub tasks: Vec<Task>,
    pub task_report_labels: Vec<String>,
    pub task_report_columns: Vec<String>,
    pub input_mode: InputMode,
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
            input_mode: InputMode::Normal,
        };
        app.update();
        app
    }

    pub fn draw(&mut self, f: &mut Frame<impl Backend>) {
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
        self.draw_command(f, rects[2]);
        match self.input_mode {
            InputMode::Normal => (),
            InputMode::Command => {
                // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
                f.set_cursor(
                    // Put cursor past the end of the input text
                    rects[2].x + self.filter.width() as u16 + 1,
                    // Move one line down, from the border to the input line
                    rects[2].y + 1,
                )
            }
        }
    }

    fn draw_command(&mut self, f: &mut Frame<impl Backend>, rect: Rect) {
        let p = Paragraph::new(Text::from(&self.filter[..]))
            .block(Block::default().borders(Borders::ALL).title("Command"));
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
            return
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
            return
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

    pub fn handle_input(&mut self, event: Key) {
        match self.input_mode {
            InputMode::Normal => match event {
                Key::Ctrl('c') | Key::Char('q') => self.should_quit = true,
                Key::Char('r') => self.update(),
                Key::Down | Key::Char('j') => self.next(),
                Key::Up | Key::Char('k') => self.previous(),
                Key::Char('/') => {
                    self.input_mode = InputMode::Command;
                }
                _ => {}
            },
            InputMode::Command => match event {
                Key::Char('\n') | Key::Esc => {
                    self.input_mode = InputMode::Normal;
                }
                Key::Char(c) => {
                    self.filter.push(c);
                }
                Key::Backspace => {
                    self.filter.pop();
                }
                _ => {}
            },
        }
    }

}

#[cfg(test)]
mod tests {
    use crate::app::App;
    use std::io::stdin;

    use task_hookrs::import::import;
    use task_hookrs::task::Task;

    #[test]
    fn test_app() {
        let mut app = App::new();
        app.update();

        println!("{:?}", app.tasks);

        println!("{:?}", app.task_report_columns);
        println!("{:?}", app.task_report_labels);

        let (t, h, c) = app.task_report();
        println!("{:?}", t);
        println!("{:?}", t);
        println!("{:?}", t);
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
