use crate::config::TConfig;

use std::cmp::Ordering;
use std::convert::TryInto;
use std::process::Command;
use std::result::Result;

use task_hookrs::date::Date;
use task_hookrs::import::import;
use task_hookrs::status::TaskStatus;
use task_hookrs::task::Task;
use task_hookrs::uda::UDAValue;
use uuid::Uuid;

use chrono::{Local, NaiveDateTime, TimeZone, Datelike};

use crate::calendar::Calendar;
use std::sync::{Arc, Mutex};
use std::{sync::mpsc, thread, time::Duration};
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    terminal::Frame,
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::table::{Row, Table, TableState};

use rustyline::error::ReadlineError;
use rustyline::line_buffer::LineBuffer;
use rustyline::At;
use rustyline::Editor;
use rustyline::Word;

use crate::util::Events;
use crate::util::Key;

use std::io::{self};
use tui::{backend::CrosstermBackend, Terminal};

const MAX_LINE: usize = 4096;

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

pub enum DateState {
    BeforeToday,
    EarlierToday,
    LaterToday,
    AfterToday,
}

pub fn get_date_state(reference: &Date) -> DateState {
    let now = Local::now();
    let reference = TimeZone::from_utc_datetime(now.offset(), reference);
    let now = TimeZone::from_utc_datetime(now.offset(), &now.naive_utc());

    if reference.date() < now.date() {
        return DateState::BeforeToday;
    }

    if reference.date() == now.date() {
        if reference.time() < now.time() {
            return DateState::EarlierToday;
        } else {
            return DateState::LaterToday;
        }
    }
    DateState::AfterToday
}

pub fn vague_format_date_time(from_dt: NaiveDateTime, to_dt: NaiveDateTime) -> String {
    let seconds = (to_dt - from_dt).num_seconds();

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
    TaskReport,
    TaskFilter,
    TaskAdd,
    TaskAnnotate,
    TaskSubprocess,
    TaskLog,
    TaskModify,
    TaskHelpPopup,
    TaskError,
    Calendar,
}

pub struct TaskReportTable {
    pub labels: Vec<String>,
    pub columns: Vec<String>,
    pub tasks: Vec<Vec<String>>,
}

impl TaskReportTable {
    pub fn new() -> Self {
        let mut task_report_table = Self {
            labels: vec![],
            columns: vec![],
            tasks: vec![vec![]],
        };
        task_report_table.export_headers();
        task_report_table
    }

    pub fn export_headers(&mut self) {
        self.columns = vec![];
        self.labels = vec![];

        let output = Command::new("task")
            .arg("show")
            .arg("report.next.columns")
            .output()
            .expect("Unable to run `task show report.next.columns`. Check documentation for more information");
        let data = String::from_utf8(output.stdout).expect("Unable to decode utf8 from stdout of `task show report.next.columns`");

        for line in data.split('\n') {
            if line.starts_with("report.next.columns") {
                let column_names = line.split(' ').collect::<Vec<_>>()[1];
                for column in column_names.split(',') {
                    self.columns.push(column.to_string());
                }
            }
        }

        let output = Command::new("task")
            .arg("show")
            .arg("report.next.labels")
            .output()
            .expect("Unable to run `task show report.next.labels`. Check documentation for more information");
        let data = String::from_utf8(output.stdout).expect("Unable to decode utf8 from stdout of `task show report.next.labels`");

        for line in data.split('\n') {
            if line.starts_with("report.next.labels") {
                let label_names = line.split(' ').collect::<Vec<_>>()[1];
                for label in label_names.split(',') {
                    self.labels.push(label.to_string());
                }
            }
        }
    }

    pub fn generate_table(&mut self, tasks: &[Task]) {
        self.tasks = vec![];

        // get all tasks as their string representation
        for task in tasks {
            let mut item = vec![];
            for name in &self.columns {
                let s = self.get_string_attribute(name, &task);
                item.push(s);
            }
            self.tasks.push(item)
        }
    }

    pub fn simplify_table(&mut self) -> (Vec<Vec<String>>, Vec<String>) {
        // find which columns are empty
        let null_columns_len;
        if !self.tasks.is_empty() {
            null_columns_len = self.tasks[0].len();
        } else {
            return (vec![], vec![]);
        }

        let mut null_columns = vec![0; null_columns_len];
        for task in &self.tasks {
            for (i, s) in task.iter().enumerate() {
                null_columns[i] += s.len();
            }
        }

        // filter out columns where everything is empty
        let mut tasks = vec![];
        for task in &self.tasks {
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
            .labels
            .iter()
            .enumerate()
            .filter(|&(i, _)| null_columns[i] != 0)
            .map(|(_, e)| e.to_owned())
            .collect();

        (tasks, headers)
    }

    pub fn get_string_attribute(&self, attribute: &str, task: &Task) -> String {
        let tags = vec![
            "PROJECT",
            "BLOCKED",
            "UNBLOCKED",
            "BLOCKING",
            "DUE",
            "DUETODAY",
            "TODAY",
            "OVERDUE",
            "WEEK",
            "MONTH",
            "QUARTER",
            "YEAR",
            "ACTIVE",
            "SCHEDULED",
            "PARENT",
            "CHILD",
            "UNTIL",
            "WAITING",
            "ANNOTATED",
            "READY",
            "YESTERDAY",
            "TOMORROW",
            "TAGGED",
            "PENDING",
            "COMPLETED",
            "DELETED",
            "UDA",
            "ORPHAN",
            "PRIORITY",
            "PROJECT",
            "LATEST",
        ];

        match attribute {
            "id" => task.id().unwrap_or_default().to_string(),
            "due.relative" => match task.due() {
                Some(v) => vague_format_date_time(Local::now().naive_utc(), NaiveDateTime::new(v.date(), v.time())),
                None => "".to_string(),
            },
            "entry.age" => vague_format_date_time(
                NaiveDateTime::new(task.entry().date(), task.entry().time()),
                Local::now().naive_utc(),
            ),
            "start.age" => match task.start() {
                Some(v) => vague_format_date_time(NaiveDateTime::new(v.date(), v.time()), Local::now().naive_utc()),
                None => "".to_string(),
            },
            "project" => match task.project() {
                Some(p) => p.to_string(),
                None => "".to_string(),
            },
            "depends.count" => match task.depends() {
                Some(v) => {
                    if v.is_empty() {
                        "".to_string()
                    } else {
                        format!("{}", v.len())
                    }
                }
                None => "".to_string(),
            },
            "tags.count" => match task.tags() {
                Some(v) => {
                    let t = v.iter().filter(|t| !tags.contains(&t.as_str())).cloned().count();
                    if t == 0 {
                        "".to_string()
                    } else {
                        t.to_string()
                    }
                }
                None => "".to_string(),
            },
            "tags" => match task.tags() {
                Some(v) => v
                    .iter()
                    .filter(|t| !tags.contains(&t.as_str()))
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(","),
                None => "".to_string(),
            },
            "description.count" => task.description().to_string(),
            "description" => task.description().to_string(),
            "urgency" => match &task.uda()["urgency"] {
                UDAValue::Str(_) => "0.00".to_string(),
                UDAValue::U64(u) => format!("{:.2}", *u as f64),
                UDAValue::F64(f) => format!("{:.2}", *f),
            },
            _ => "".to_string(),
        }
    }
}

pub struct TTApp {
    pub should_quit: bool,
    pub state: TableState,
    pub context_filter: String,
    pub context_name: String,
    pub command: LineBuffer,
    pub filter: LineBuffer,
    pub modify: LineBuffer,
    pub error: String,
    pub tasks: Arc<Mutex<Vec<Task>>>,
    pub task_report_table: TaskReportTable,
    pub mode: AppMode,
    pub config: TConfig,
}

impl TTApp {
    pub fn new() -> Self {
        let mut app = Self {
            should_quit: false,
            state: TableState::default(),
            tasks: Arc::new(Mutex::new(vec![])),
            context_filter: "".to_string(),
            context_name: "".to_string(),
            command: LineBuffer::with_capacity(MAX_LINE),
            filter: LineBuffer::with_capacity(MAX_LINE),
            modify: LineBuffer::with_capacity(MAX_LINE),
            error: "".to_string(),
            mode: AppMode::TaskReport,
            config: TConfig::default(),
            task_report_table: TaskReportTable::new(),
        };
        for c in "status:pending ".chars() {
            app.filter.insert(c, 1);
        }
        app.get_context();
        app.update();
        app
    }

    pub fn get_context(&mut self) {
        self.context_name = String::from_utf8(
            Command::new("task")
                .arg("_get")
                .arg("rc.context")
                .output()
                .expect("Unable to run `task _get rc.context`")
                .stdout,
        )
        .expect("Unable to decode utf8 from stdout of `task _get rc.context`");
        self.context_name = self.context_name.strip_suffix('\n').unwrap_or("").to_string();

        self.context_filter = String::from_utf8(
            Command::new("task")
                .arg("_get")
                .arg(format!("rc.context.{}", self.context_name))
                .output()
                .expect(format!("Unable to run `task _get rc.context.{}`", self.context_name).as_str())
                .stdout,
        )
        .expect(format!("Unable to decode utf8 from stdout of `task _get rc.context.{}`", self.context_name).as_str());
        self.context_filter = self.context_filter.strip_suffix('\n').unwrap_or("").to_string();
    }

    pub fn draw(&mut self, f: &mut Frame<impl Backend>) {
        match self.mode {
            AppMode::TaskReport
            | AppMode::TaskFilter
            | AppMode::TaskAdd
            | AppMode::TaskAnnotate
            | AppMode::TaskError
            | AppMode::TaskHelpPopup
            | AppMode::TaskSubprocess
            | AppMode::TaskLog
            | AppMode::TaskModify => self.draw_task(f),
            AppMode::Calendar => self.draw_calendar(f),
        }
    }

    pub fn draw_calendar(&mut self, f: &mut Frame<impl Backend>) {
        let rects = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0)].as_ref())
            .split(f.size());
        let today = Local::today();
        let c = &Calendar::new(today.year().into()).to_string()[..];
        let p = Paragraph::new(Text::from(c))
            .alignment(Alignment::Left)
            .block(Block::default().borders(Borders::ALL).title("Calendar"));
        f.render_widget(p, rects[0]);
    }

    pub fn draw_task(&mut self, f: &mut Frame<impl Backend>) {
        let tasks_is_empty = self.tasks.lock().unwrap().is_empty();
        let tasks_len = self.tasks.lock().unwrap().len();
        while !tasks_is_empty && self.state.selected().unwrap_or_default() >= tasks_len {
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
        let selected = self.state.selected().unwrap_or_default();
        let task_id = if tasks_len == 0 {
            0
        } else {
            self.tasks.lock().unwrap()[selected].id().unwrap_or_default()
        };
        match self.mode {
            AppMode::TaskReport => self.draw_command(f, rects[1], self.filter.as_str(), "Filter Tasks"),
            AppMode::TaskFilter => {
                f.render_widget(Clear, rects[1]);
                f.set_cursor(rects[1].x + self.filter.pos() as u16 + 1, rects[1].y + 1);
                self.draw_command(f, rects[1], self.filter.as_str(), "Filter Tasks");
            }
            AppMode::TaskModify => {
                f.set_cursor(rects[1].x + self.modify.pos() as u16 + 1, rects[1].y + 1);
                f.render_widget(Clear, rects[1]);
                self.draw_command(
                    f,
                    rects[1],
                    self.modify.as_str(),
                    format!("Modify Task {}", task_id).as_str(),
                );
            }
            AppMode::TaskLog => {
                f.set_cursor(rects[1].x + self.command.pos() as u16 + 1, rects[1].y + 1);
                f.render_widget(Clear, rects[1]);
                self.draw_command(f, rects[1], self.command.as_str(), "Log Task");
            }
            AppMode::TaskSubprocess => {
                f.set_cursor(rects[1].x + self.command.pos() as u16 + 1, rects[1].y + 1);
                f.render_widget(Clear, rects[1]);
                self.draw_command(f, rects[1], self.command.as_str(), "Shell Command");
            }
            AppMode::TaskAnnotate => {
                f.set_cursor(rects[1].x + self.command.pos() as u16 + 1, rects[1].y + 1);
                f.render_widget(Clear, rects[1]);
                self.draw_command(
                    f,
                    rects[1],
                    self.command.as_str(),
                    format!("Annotate Task {}", task_id).as_str(),
                );
            }
            AppMode::TaskAdd => {
                f.set_cursor(rects[1].x + self.command.pos() as u16 + 1, rects[1].y + 1);
                f.render_widget(Clear, rects[1]);
                self.draw_command(f, rects[1], self.command.as_str(), "Add Task");
            }
            AppMode::TaskError => {
                f.render_widget(Clear, rects[1]);
                self.draw_command(f, rects[1], self.error.as_str(), "Error");
            }
            AppMode::TaskHelpPopup => {
                self.draw_command(f, rects[1], self.filter.as_str(), "Filter Tasks");
                self.draw_help_popup(f, f.size());
            }
            AppMode::Calendar => {
                panic!("Reached unreachable code. Something went wrong");
            }
        }
    }

    fn draw_help_popup(&self, f: &mut Frame<impl Backend>, rect: Rect) {
        let text = vec![
            Spans::from(""),
            Spans::from(vec![
                Span::from("    /"),
                Span::from("    "),
                Span::styled(
                    "task {string}              ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::from("    "),
                Span::from("- Filter task report"),
            ]),
            Spans::from(""),
            Spans::from(vec![
                Span::from("    a"),
                Span::from("    "),
                Span::styled(
                    "task add {string}          ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::from("    "),
                Span::from("- Add new task"),
            ]),
            Spans::from(""),
            Spans::from(vec![
                Span::from("    A"),
                Span::from("    "),
                Span::styled(
                    "task annotate {string}     ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::from("    "),
                Span::from("- Annotate task"),
            ]),
            Spans::from(""),
            Spans::from(vec![
                Span::from("    d"),
                Span::from("    "),
                Span::styled(
                    "task done {selected}       ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::from("    "),
                Span::from("- Mark task as done"),
            ]),
            Spans::from(""),
            Spans::from(vec![
                Span::from("    e"),
                Span::from("    "),
                Span::styled(
                    "task edit {selected}       ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::from("    "),
                Span::from("- Open selected task in editor"),
            ]),
            Spans::from(""),
            Spans::from(vec![
                Span::from("    j"),
                Span::from("    "),
                Span::styled(
                    "{selected+=1}              ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::from("    "),
                Span::from("- Move down in task report"),
            ]),
            Spans::from(""),
            Spans::from(vec![
                Span::from("    k"),
                Span::from("    "),
                Span::styled(
                    "{selected-=1}              ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::from("    "),
                Span::from("- Move up in task report"),
            ]),
            Spans::from(""),
            Spans::from(vec![
                Span::from("    l"),
                Span::from("    "),
                Span::styled(
                    "task log {string}          ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::from("    "),
                Span::from("- Log new task"),
            ]),
            Spans::from(""),
            Spans::from(vec![
                Span::from("    m"),
                Span::from("    "),
                Span::styled(
                    "task modify {string}       ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::from("    "),
                Span::from("- Modify selected task"),
            ]),
            Spans::from(""),
            Spans::from(vec![
                Span::from("    q"),
                Span::from("    "),
                Span::styled(
                    "exit                       ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::from("    "),
                Span::from("- Quit"),
            ]),
            Spans::from(""),
            Spans::from(vec![
                Span::from("    s"),
                Span::from("    "),
                Span::styled(
                    "task start/stop {selected} ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::from("    "),
                Span::from("- Toggle start and stop"),
            ]),
            Spans::from(""),
            Spans::from(vec![
                Span::from("    u"),
                Span::from("    "),
                Span::styled(
                    "task undo                  ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::from("    "),
                Span::from("- Undo"),
            ]),
            Spans::from(""),
            Spans::from(vec![
                Span::from("    x"),
                Span::from("    "),
                Span::styled(
                    "task delete                ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::from("    "),
                Span::from("- Delete task"),
            ]),
            Spans::from(""),
            Spans::from(vec![
                Span::from("    ?"),
                Span::from("    "),
                Span::styled(
                    "help                       ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::from("    "),
                Span::from("- Show this help menu"),
            ]),
            Spans::from(""),
            Spans::from(vec![
                Span::from("    !"),
                Span::from("    "),
                Span::styled(
                    "shell                      ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::from("    "),
                Span::from("- Custom shell command"),
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
        let p = Paragraph::new(Text::from(text)).block(Block::default().borders(Borders::ALL).title(title));
        f.render_widget(p, rect);
    }

    fn draw_task_details(&mut self, f: &mut Frame<impl Backend>, rect: Rect) {
        if self.tasks.lock().unwrap().is_empty() {
            f.render_widget(Block::default().borders(Borders::ALL).title("Task not found"), rect);
            return;
        }
        let selected = self.state.selected().unwrap_or_default();
        let task_id = self.tasks.lock().unwrap()[selected].id().unwrap_or_default();
        let output = Command::new("task").arg(format!("{}", task_id)).output();
        if let Ok(output) = output {
            let data = String::from_utf8(output.stdout).unwrap_or(format!(
                "Unable to get description of task with id: {}. Please report as an issue on github.",
                task_id
            ));
            let p = Paragraph::new(Text::from(&data[..])).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("Task {}", task_id)),
            );
            f.render_widget(p, rect);
        }
    }

    fn task_by_index(&self, i: usize) -> Option<Task> {
        let tasks = &self.tasks.lock().unwrap();
        if i > tasks.len() {
            None
        } else {
            Some(tasks[i].clone())
        }
    }

    fn task_by_uuid(&self, uuid: Uuid) -> Option<Task> {
        let tasks = &self.tasks.lock().unwrap();
        let m = tasks.iter().find(|t| *t.uuid() == uuid);
        match m {
            Some(v) => Some(v.clone()),
            None => None,
        }
    }

    fn style_for_task(&self, task: &Task) -> Style {

        let tag_names_in_precedence = &self.config.rule_precedence_color;

        let mut style = Style::default();

        for tag_name in tag_names_in_precedence {
            if task.tags().unwrap_or(&vec![]).contains(&tag_name.to_string().replace(".", "").to_uppercase()) {
                let color_tag_name = format!("color.{}", tag_name);
                let c = self.config.color.get(&color_tag_name).cloned().unwrap_or_default();
                style = style.fg(c.fg).bg(c.bg);
                for modifier in c.modifiers {
                    style = style.add_modifier(modifier);
                }
                break
            }
        }

        style
    }

    fn draw_task_report(&mut self, f: &mut Frame<impl Backend>, rect: Rect) {
        let (tasks, headers) = self.task_report();
        if tasks.is_empty() {
            f.render_widget(Block::default().borders(Borders::ALL).title("Task next"), rect);
            return;
        }

        let maximum_column_width = rect.width as i16 / tasks[0].len() as i16;
        let mut description_column_width = rect.width as i16 / tasks[0].len() as i16;
        let mut description_column_index = 0;

        // set widths proportional to the content
        let mut widths: Vec<i16> = vec![0; tasks[0].len()];

        for (i, header) in headers.iter().enumerate() {
            widths[i] = header.len() as i16 + 2;
            if header != "Description" {
                description_column_width += std::cmp::max(0, maximum_column_width - widths[i]);
            } else {
                description_column_index = i;
            }
        }
        widths[0] += 1;

        let sum_of_remaining_widths: i16 = widths
            .iter()
            .enumerate()
            .filter(|(i, w)| i != &description_column_index)
            .map(|(i, w)| w)
            .sum();

        if description_column_width + sum_of_remaining_widths >= rect.width as i16 - 4 {
            description_column_width = rect.width as i16 - sum_of_remaining_widths - 10;
        }

        for task in &tasks {
            for (i, attr) in task.iter().enumerate() {
                if i == description_column_index {
                    widths[i] = std::cmp::max(
                        widths[i],
                        std::cmp::min(attr.len() as i16 + 2, description_column_width),
                    );
                } else {
                    widths[i] = std::cmp::max(widths[i], std::cmp::min(attr.len() as i16, maximum_column_width));
                }
            }
        }

        let selected = self.state.selected().unwrap_or_default();
        let header = headers.iter();
        let mut rows = vec![];
        let mut highlight_style = Style::default();
        for (i, task) in tasks.iter().enumerate() {
            let style = self.style_for_task(&self.tasks.lock().unwrap()[i]);
            if i == selected {
                highlight_style = style.add_modifier(Modifier::BOLD);
            }
            rows.push(Row::StyledData(task.iter(), style));
        }

        let constraints: Vec<Constraint> = widths
            .iter()
            .map(|i| Constraint::Min((*i).try_into().unwrap_or(10)))
            .collect();

        let t = Table::new(header, rows.into_iter())
            .block(Block::default().borders(Borders::ALL).title("Task next"))
            .highlight_style(highlight_style)
            .highlight_symbol(&self.config.uda_current_task_indicator)
            .widths(&constraints);

        f.render_stateful_widget(t, rect, &mut self.state);
    }

    pub fn task_report(&mut self) -> (Vec<Vec<String>>, Vec<String>) {
        let alltasks = &*(self.tasks.lock().unwrap());

        self.task_report_table.generate_table(alltasks);

        let (tasks, headers) = self.task_report_table.simplify_table();

        (tasks, headers)
    }

    pub fn update(&mut self) {
        self.export_tasks();
        self.update_tags();
    }

    pub fn next(&mut self) {
        if self.tasks.lock().unwrap().is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.tasks.lock().unwrap().len() - 1 {
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
        if self.tasks.lock().unwrap().is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.tasks.lock().unwrap().len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn export_tasks(&self) {
        let mut task = Command::new("task");

        task.arg("rc.json.array=on");
        task.arg("export");

        let filter = if self.context_filter != *"" {
            let t = format!("{} {}", self.filter.as_str(), self.context_filter);
            t
        } else {
            self.filter.as_str().into()
        };
        match shlex::split(&filter) {
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
            .expect("Unable to run `task export`. Check documentation for more information.");
        let data = String::from_utf8(output.stdout).unwrap();
        let imported = import(data.as_bytes());
        {
            if let Ok(i) = imported {
                *(self.tasks.lock().unwrap()) = i;
                self.tasks.lock().unwrap().sort_by(cmp);
            }
        }
    }

    pub fn task_subprocess(&mut self) -> Result<(), String> {
        if self.tasks.lock().unwrap().is_empty() {
            return Ok(());
        }

        match shlex::split(&self.command) {
            Some(cmd) => {
                let mut command = Command::new(&cmd[0]);
                for (i, s) in cmd.iter().enumerate() {
                    if i == 0 {
                        continue;
                    }
                    command.arg(&s);
                }
                let output = command.output();
                match output {
                    Ok(_) => {
                        self.command.update("", 0);
                        Ok(())
                    }
                    Err(_) => Err(format!(
                        "Shell command `{}` exited with non-zero output",
                        self.command.as_str()
                    )),
                }
            }
            None => Err(format!("Unable to split `{}`", self.command.as_str())),
        }
    }

    pub fn task_log(&mut self) -> Result<(), String> {
        if self.tasks.lock().unwrap().is_empty() {
            return Ok(());
        }

        let mut command = Command::new("task");

        command.arg("log");

        match shlex::split(&self.command) {
            Some(cmd) => {
                for s in cmd {
                    command.arg(&s);
                }
                let output = command.output();
                match output {
                    Ok(_) => {
                        self.command.update("", 0);
                        Ok(())
                    }
                    Err(_) => {
                        Err("Cannot run `task log` for task `{}`. Check documentation for more information".to_string())
                    }
                }
            }
            None => Err(format!("Unable to run `task log` with `{}`", self.command.as_str())),
        }
    }

    pub fn task_modify(&mut self) -> Result<(), String> {
        if self.tasks.lock().unwrap().is_empty() {
            return Ok(());
        }
        let selected = self.state.selected().unwrap_or_default();
        let task_id = self.tasks.lock().unwrap()[selected].id().unwrap_or_default();
        let mut command = Command::new("task");
        command.arg(format!("{}", task_id)).arg("modify");

        match shlex::split(&self.modify) {
            Some(cmd) => {
                for s in cmd {
                    command.arg(&s);
                }
                let output = command.output();
                match output {
                    Ok(_) => {
                        self.modify.update("", 0);
                        Ok(())
                    }
                    Err(_) => Err(format!(
                        "Cannot run `task modify` for task `{}`. Check documentation for more information",
                        task_id
                    )),
                }
            }
            None => Err(format!(
                "Unable to run `task modify` with `{}` on task {}",
                self.modify.as_str(),
                &task_id
            )),
        }
    }

    pub fn task_annotate(&mut self) -> Result<(), String> {
        if self.tasks.lock().unwrap().is_empty() {
            return Ok(());
        }
        let selected = self.state.selected().unwrap_or_default();
        let task_id = self.tasks.lock().unwrap()[selected].id().unwrap_or_default();
        let mut command = Command::new("task");
        command.arg(format!("{}", task_id)).arg("annotate");

        match shlex::split(&self.command) {
            Some(cmd) => {
                for s in cmd {
                    command.arg(&s);
                }
                let output = command.output();
                match output {
                    Ok(_) => {
                        self.command.update("", 0);
                        Ok(())
                    }
                    Err(_) => Err("Cannot run `task annotate`. Check documentation for more information".to_string()),
                }
            }
            None => Err(format!("Unable to run `task add` with `{}`", self.command.as_str())),
        }
    }

    pub fn task_add(&mut self) -> Result<(), String> {
        let mut command = Command::new("task");
        command.arg("add");

        match shlex::split(&self.command) {
            Some(cmd) => {
                for s in cmd {
                    command.arg(&s);
                }
                let output = command.output();
                match output {
                    Ok(_) => {
                        self.command.update("", 0);
                        Ok(())
                    }
                    Err(_) => Err("Cannot run `task add`. Check documentation for more information".to_string()),
                }
            }
            None => Err(format!("Unable to run `task add` with `{}`", self.command.as_str())),
        }
    }

    pub fn task_virtual_tags(task_id: u64) -> Result<String, String> {
        let output = Command::new("task").arg(format!("{}", task_id)).output();

        match output {
            Ok(output) => {
                let data = String::from_utf8(output.stdout).unwrap();
                for line in data.split('\n') {
                    if line.starts_with("Virtual tags") {
                        let line = line.to_string();
                        let line = line.replace("Virtual tags", "");
                        return Ok(line);
                    }
                }
                Err(format!(
                    "Cannot find any tags for `task {}`. Check documentation for more information",
                    task_id
                ))
            }
            Err(_) => Err(format!(
                "Cannot run `task {}`. Check documentation for more information",
                task_id
            )),
        }
    }

    pub fn task_start_or_stop(&mut self) -> Result<(), String> {
        if self.tasks.lock().unwrap().is_empty() {
            return Ok(());
        }
        let selected = self.state.selected().unwrap_or_default();
        let task_id = self.tasks.lock().unwrap()[selected].id().unwrap_or_default();
        let mut command = "start";
        for tag in TTApp::task_virtual_tags(task_id)?.split(' ') {
            if tag == "ACTIVE" {
                command = "stop"
            }
        }

        let output = Command::new("task").arg(format!("{}", task_id)).arg(command).output();
        match output {
            Ok(_) => Ok(()),
            Err(_) => Err(format!(
                "Cannot run `task {}` for task `{}`. Check documentation for more information",
                command, task_id,
            )),
        }
    }

    pub fn task_delete(&self) -> Result<(), String> {
        if self.tasks.lock().unwrap().is_empty() {
            return Ok(());
        }
        let selected = self.state.selected().unwrap_or_default();
        let task_id = self.tasks.lock().unwrap()[selected].id().unwrap_or_default();

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
        if self.tasks.lock().unwrap().is_empty() {
            return Ok(());
        }
        let selected = self.state.selected().unwrap_or_default();
        let task_id = self.tasks.lock().unwrap()[selected].id().unwrap_or_default();
        let output = Command::new("task").arg(format!("{}", task_id)).arg("done").output();
        match output {
            Ok(_) => Ok(()),
            Err(_) => Err(format!(
                "Cannot run `task done` for task `{}`. Check documentation for more information",
                task_id
            )),
        }
    }

    pub fn task_undo(&self) -> Result<(), String> {
        if self.tasks.lock().unwrap().is_empty() {
            return Ok(());
        }
        let output = Command::new("task").arg("rc.confirmation=off").arg("undo").output();

        match output {
            Ok(_) => Ok(()),
            Err(_) => Err("Cannot run `task undo`. Check documentation for more information".to_string()),
        }
    }

    pub fn task_edit(&self) -> Result<(), String> {
        if self.tasks.lock().unwrap().is_empty() {
            return Ok(());
        }
        let selected = self.state.selected().unwrap_or_default();
        let task_id = self.tasks.lock().unwrap()[selected].id().unwrap_or_default();
        let r = Command::new("task").arg(format!("{}", task_id)).arg("edit").spawn();

        match r {
            Ok(child) => {
                let output = child.wait_with_output();
                match output {
                    Ok(output) => {
                        if !output.status.success() {
                            Err(format!(
                                "`task edit` for task `{}` failed. {}{}",
                                task_id,
                                String::from_utf8(output.stdout).unwrap(),
                                String::from_utf8(output.stderr).unwrap()
                            ))
                        } else {
                            Ok(())
                        }
                    }
                    Err(err) => Err(format!("Cannot run `task edit` for task `{}`. {}", task_id, err)),
                }
            }
            _ => Err(format!(
                "Cannot start `task edit` for task `{}`. Check documentation for more information",
                task_id
            )),
        }
    }

    pub fn task_current(&self) -> Option<Task> {
        if self.tasks.lock().unwrap().is_empty() {
            return None;
        }
        let selected = self.state.selected().unwrap_or_default();
        Some(self.tasks.lock().unwrap()[selected].clone())
    }

    pub fn update_tags(&mut self) {
        let tasks = &mut *self.tasks.lock().unwrap();

        // dependency scan
        for l_i in 0..tasks.len() {
            let default_deps = vec![];
            let deps = tasks[l_i].depends().unwrap_or(&default_deps).clone();
            for dep in deps {
                for r_i in 0..tasks.len() {
                    if tasks[r_i].uuid() == &dep {
                        let lstatus = tasks[l_i].status();
                        let rstatus = tasks[r_i].status();
                        if lstatus != &TaskStatus::Completed
                            && lstatus != &TaskStatus::Deleted
                            && rstatus != &TaskStatus::Completed
                            && rstatus != &TaskStatus::Deleted
                        {
                            add_tag(&mut tasks[l_i], "BLOCKED".to_string());
                            add_tag(&mut tasks[r_i], "BLOCKING".to_string());
                        }
                        break;
                    }
                }
            }
        }

        // other virtual tags
        // TODO: support all virtual tags that taskwarrior supports
        for mut task in tasks.iter_mut() {
            match task.status() {
                TaskStatus::Waiting => add_tag(&mut task, "WAITING".to_string()),
                TaskStatus::Completed => add_tag(&mut task, "COMPLETED".to_string()),
                TaskStatus::Pending => add_tag(&mut task, "PENDING".to_string()),
                TaskStatus::Deleted => add_tag(&mut task, "DELETED".to_string()),
                TaskStatus::Recurring => (),
            }
            if task.start().is_some() {
                add_tag(&mut task, "ACTIVE".to_string());
            }
            if task.scheduled().is_some() {
                add_tag(&mut task, "SCHEDULED".to_string());
            }
            if task.parent().is_some() {
                add_tag(&mut task, "INSTANCE".to_string());
            }
            if task.until().is_some() {
                add_tag(&mut task, "UNTIL".to_string());
            }
            if task.annotations().is_some() {
                add_tag(&mut task, "ANNOTATED".to_string());
            }
            if task.tags().is_some() {
                add_tag(&mut task, "TAGGED".to_string());
            }
            if task.mask().is_some() {
                add_tag(&mut task, "TEMPLATE".to_string());
            }
            if task.project().is_some() {
                add_tag(&mut task, "PROJECT".to_string());
            }
            if task.priority().is_some() {
                add_tag(&mut task, "PROJECT".to_string());
            }
            if task.due().is_some() {
                add_tag(&mut task, "DUE".to_string());
            }
            if let Some(d) = task.due() {
                let status = task.status();
                // due today
                if status != &TaskStatus::Completed && status != &TaskStatus::Deleted {
                    let today = Local::now().naive_utc().date();
                    match get_date_state(d) {
                        DateState::EarlierToday | DateState::LaterToday => add_tag(&mut task, "TODAY".to_string()),
                        _ => (),
                    }
                }
            }
            if let Some(d) = task.due() {
                let status = task.status();
                // overdue
                if status != &TaskStatus::Completed
                    && status != &TaskStatus::Deleted
                    && status != &TaskStatus::Recurring
                {
                    let now = Local::now().naive_utc();
                    let d = NaiveDateTime::new(d.date(), d.time());
                    if d < now {
                        add_tag(&mut task, "OVERDUE".to_string());
                    }
                }
            }
        }
    }

    pub fn handle_input(&mut self, input: Key, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, events: &Events) {
        match self.mode {
            AppMode::TaskReport => match input {
                Key::Ctrl('c') | Key::Char('q') => self.should_quit = true,
                Key::Char(']') => {
                    self.mode = AppMode::Calendar;
                }
                Key::Char('r') => self.update(),
                Key::Down | Key::Char('j') => self.next(),
                Key::Up | Key::Char('k') => self.previous(),
                Key::Char('d') => match self.task_done() {
                    Ok(_) => self.update(),
                    Err(e) => {
                        self.mode = AppMode::TaskError;
                        self.error = e;
                    }
                },
                Key::Char('x') => match self.task_delete() {
                    Ok(_) => self.update(),
                    Err(e) => {
                        self.mode = AppMode::TaskError;
                        self.error = e;
                    }
                },
                Key::Char('s') => match self.task_start_or_stop() {
                    Ok(_) => self.update(),
                    Err(e) => {
                        self.mode = AppMode::TaskError;
                        self.error = e;
                    }
                },
                Key::Char('u') => match self.task_undo() {
                    Ok(_) => self.update(),
                    Err(e) => {
                        self.mode = AppMode::TaskError;
                        self.error = e;
                    }
                },
                Key::Char('e') => {
                    events.pause_event_loop(terminal);
                    let r = self.task_edit();
                    events.resume_event_loop(terminal);
                    match r {
                        Ok(_) => self.update(),
                        Err(e) => {
                            self.mode = AppMode::TaskError;
                            self.error = e;
                        }
                    }
                }
                Key::Char('m') => {
                    self.mode = AppMode::TaskModify;
                    match self.task_current() {
                        Some(t) => self.modify.update(t.description(), t.description().len()),
                        None => self.modify.update("", 0),
                    }
                }
                Key::Char('!') => {
                    self.mode = AppMode::TaskSubprocess;
                }
                Key::Char('l') => {
                    self.mode = AppMode::TaskLog;
                }
                Key::Char('a') => {
                    self.mode = AppMode::TaskAdd;
                }
                Key::Char('A') => {
                    self.mode = AppMode::TaskAnnotate;
                }
                Key::Char('?') => {
                    self.mode = AppMode::TaskHelpPopup;
                }
                Key::Char('/') => {
                    self.mode = AppMode::TaskFilter;
                }
                _ => {}
            },
            AppMode::TaskHelpPopup => match input {
                Key::Esc => {
                    self.mode = AppMode::TaskReport;
                }
                _ => {}
            },
            AppMode::TaskModify => match input {
                Key::Char('\n') => match self.task_modify() {
                    Ok(_) => {
                        self.mode = AppMode::TaskReport;
                        self.update();
                    }
                    Err(e) => {
                        self.mode = AppMode::TaskError;
                        self.error = e;
                    }
                },
                Key::Esc => {
                    self.modify.update("", 0);
                    self.mode = AppMode::TaskReport;
                }
                _ => handle_movement(&mut self.modify, input),
            },
            AppMode::TaskSubprocess => match input {
                Key::Char('\n') => match self.task_subprocess() {
                    Ok(_) => {
                        self.mode = AppMode::TaskReport;
                        self.update();
                    }
                    Err(e) => {
                        self.mode = AppMode::TaskError;
                        self.error = e;
                    }
                },
                Key::Esc => {
                    self.command.update("", 0);
                    self.mode = AppMode::TaskReport;
                }
                _ => handle_movement(&mut self.command, input),
            },
            AppMode::TaskLog => match input {
                Key::Char('\n') => match self.task_log() {
                    Ok(_) => {
                        self.mode = AppMode::TaskReport;
                        self.update();
                    }
                    Err(e) => {
                        self.mode = AppMode::TaskError;
                        self.error = e;
                    }
                },
                Key::Esc => {
                    self.command.update("", 0);
                    self.mode = AppMode::TaskReport;
                }
                _ => handle_movement(&mut self.command, input),
            },
            AppMode::TaskAnnotate => match input {
                Key::Char('\n') => match self.task_annotate() {
                    Ok(_) => {
                        self.mode = AppMode::TaskReport;
                        self.update();
                    }
                    Err(e) => {
                        self.mode = AppMode::TaskError;
                        self.error = e;
                    }
                },
                Key::Esc => {
                    self.command.update("", 0);
                    self.mode = AppMode::TaskReport;
                }
                _ => handle_movement(&mut self.command, input),
            },
            AppMode::TaskAdd => match input {
                Key::Char('\n') => match self.task_add() {
                    Ok(_) => {
                        self.mode = AppMode::TaskReport;
                        self.update();
                    }
                    Err(e) => {
                        self.mode = AppMode::TaskError;
                        self.error = e;
                    }
                },
                Key::Esc => {
                    self.command.update("", 0);
                    self.mode = AppMode::TaskReport;
                }
                _ => handle_movement(&mut self.command, input),
            },
            AppMode::TaskFilter => match input {
                Key::Char('\n') | Key::Esc => {
                    self.mode = AppMode::TaskReport;
                    self.update();
                }
                _ => handle_movement(&mut self.command, input),
            },
            AppMode::TaskError => self.mode = AppMode::TaskReport,
            AppMode::Calendar => match input {
                Key::Char('[') => {
                    self.mode = AppMode::TaskReport;
                }
                Key::Ctrl('c') | Key::Char('q') => self.should_quit = true,
                _ => {}
            },
        }
    }
}

pub fn handle_movement(linebuffer: &mut LineBuffer, input: Key) {
    match input {
        Key::Ctrl('f') | Key::Right => {
            linebuffer.move_forward(1);
        }
        Key::Ctrl('b') | Key::Left => {
            linebuffer.move_backward(1);
        }
        Key::Char(c) => {
            linebuffer.insert(c, 1);
        }
        Key::Ctrl('h') | Key::Backspace => {
            linebuffer.backspace(1);
        }
        Key::Ctrl('d') | Key::Delete => {
            linebuffer.delete(1);
        }
        Key::Ctrl('a') | Key::Home => {
            linebuffer.move_home();
        }
        Key::Ctrl('e') | Key::End => {
            linebuffer.move_end();
        }
        Key::Ctrl('k') => {
            linebuffer.kill_line();
        }
        Key::Ctrl('u') => {
            linebuffer.discard_line();
        }
        Key::Ctrl('w') => {
            linebuffer.delete_prev_word(Word::Emacs, 1);
        }
        Key::Alt('d') => {
            linebuffer.delete_word(At::AfterEnd, Word::Emacs, 1);
        }
        Key::Alt('f') => {
            linebuffer.move_to_next_word(At::AfterEnd, Word::Emacs, 1);
        }
        Key::Alt('b') => {
            linebuffer.move_to_prev_word(Word::Emacs, 1);
        }
        Key::Alt('t') => {
            linebuffer.transpose_words(1);
        }
        _ => {}
    }
}

pub fn add_tag(task: &mut Task, tag: String) {
    match task.tags_mut() {
        Some(t) => t.push(tag),
        None => task.set_tags(Some(vec![tag])),
    }
}

#[cfg(test)]
mod tests {
    use crate::app::TTApp;
    use crate::util::setup_terminal;
    use std::io::stdin;

    use std::{sync::mpsc, thread, time::Duration};
    use task_hookrs::import::import;
    use task_hookrs::task::Task;

    #[test]
    fn test_app() {
        let app = TTApp::new();
        assert_eq!(app.context_name, "".to_string());
        println!("{:?}", app.tasks.lock().unwrap()[0]);

        println!("{:?}", app.style_for_task(&app.task_current().unwrap()));
        //println!("{:?}", app.task_report_columns);
        //println!("{:?}", app.task_report_labels);

        // let (t, h, c) = app.task_report();
        // app.next();
        // app.next();
        // app.modify = "Cannot add this string ' because it has a single quote".to_string();
        // println!("{}", app.modify);
        // // if let Ok(tasks) = import(stdin()) {
        // //     for task in tasks {
        // //         println!("Task: {}, entered {:?} is {} -> {}",
        // //                   task.uuid(),
        // //                   task.entry(),
        // //                   task.status(),
        // //                   task.description());
        // //     }
        // // }
    }
}
