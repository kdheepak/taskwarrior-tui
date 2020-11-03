use crate::calendar::Calendar;
use crate::config::Config;
use crate::context::Context;
use crate::help::Help;
use crate::table::{Row, Table, TableState};
use crate::task_report::TaskReportTable;
use crate::util::{Events, Event};
use crate::util::Key;

use std::cmp::Ordering;
use std::convert::TryInto;
use std::error::Error;
use std::process::Command;
use std::result::Result;

use task_hookrs::date::Date;
use task_hookrs::import::import;
use task_hookrs::status::TaskStatus;
use task_hookrs::task::Task;
use task_hookrs::uda::UDAValue;
use uuid::Uuid;

use chrono::{Datelike, Local, NaiveDate, NaiveDateTime, TimeZone};

use std::sync::{Arc, Mutex};
use std::{sync::mpsc, thread, time::Duration};
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect, Margin},
    style::{Color, Modifier, Style},
    terminal::Frame,
    text::{Span, Spans, Text},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};

use rustyline::error::ReadlineError;
use rustyline::line_buffer::LineBuffer;
use rustyline::At;
use rustyline::Editor;
use rustyline::Word;

use std::io::{self};
use tui::{backend::CrosstermBackend, Terminal};

const MAX_LINE: usize = 4096;

pub fn cmp(t1: &Task, t2: &Task) -> Ordering {
    let urgency1 = match t1.urgency() {
        Some(f) => *f,
        None => 0.0,
    };
    let urgency2 = match t2.urgency() {
        Some(f) => *f,
        None => 0.0,
    };
    urgency2.partial_cmp(&urgency1).unwrap_or(Ordering::Less)
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
    TaskContextMenu,
    Calendar,
}

pub struct TTApp {
    pub should_quit: bool,
    pub task_table_state: TableState,
    pub context_table_state: TableState,
    pub current_context_filter: String,
    pub current_context: String,
    pub command: LineBuffer,
    pub filter: LineBuffer,
    pub modify: LineBuffer,
    pub error: String,
    pub tasks: Arc<Mutex<Vec<Task>>>,
    pub task_report_table: TaskReportTable,
    pub calendar_year: i32,
    pub mode: AppMode,
    pub config: Config,
    pub task_report_show_info: bool,
    pub task_report_height: u16,
    pub help_popup: Help,
    pub contexts: Vec<Context>,
}

impl TTApp {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let c = Config::default()?;
        let mut app = Self {
            should_quit: false,
            task_table_state: TableState::default(),
            context_table_state: TableState::default(),
            tasks: Arc::new(Mutex::new(vec![])),
            current_context_filter: "".to_string(),
            current_context: "".to_string(),
            command: LineBuffer::with_capacity(MAX_LINE),
            filter: LineBuffer::with_capacity(MAX_LINE),
            modify: LineBuffer::with_capacity(MAX_LINE),
            error: "".to_string(),
            mode: AppMode::TaskReport,
            task_report_height: 0,
            task_report_show_info: c.uda_task_report_show_info,
            config: c,
            task_report_table: TaskReportTable::new()?,
            calendar_year: Local::today().year(),
            help_popup: Help::new(),
            contexts: vec![],
        };
        for c in "status:pending ".chars() {
            app.filter.insert(c, 1);
        }
        app.get_context()?;
        app.update()?;
        Ok(app)
    }

    pub fn get_context(&mut self) -> Result<(), Box<dyn Error>> {
        let output = Command::new("task").arg("_get").arg("rc.context").output()?;
        self.current_context = String::from_utf8_lossy(&output.stdout).to_string();
        self.current_context = self.current_context.strip_suffix('\n').unwrap_or("").to_string();

        let output = Command::new("task")
            .arg("_get")
            .arg(format!("rc.context.{}", self.current_context))
            .output()?;
        self.current_context_filter = String::from_utf8_lossy(&output.stdout).to_string();
        self.current_context_filter = self.current_context_filter.strip_suffix('\n').unwrap_or("").to_string();
        Ok(())
    }

    pub fn draw(&mut self, f: &mut Frame<impl Backend>) {
        match self.mode {
            AppMode::TaskReport
            | AppMode::TaskFilter
            | AppMode::TaskAdd
            | AppMode::TaskAnnotate
            | AppMode::TaskContextMenu
            | AppMode::TaskError
            | AppMode::TaskHelpPopup
            | AppMode::TaskSubprocess
            | AppMode::TaskLog
            | AppMode::TaskModify => self.draw_task(f),
            AppMode::Calendar => self.draw_calendar(f),
        }
    }

    pub fn draw_calendar(&mut self, f: &mut Frame<impl Backend>) {
        let dates_with_styles = self.get_dates_with_styles();
        let rects = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0)].as_ref())
            .split(f.size());
        let today = Local::today();
        let mut c = Calendar::default()
            .block(
                Block::default()
                    .title(Spans::from(vec![
                        Span::styled("Task", Style::default().add_modifier(Modifier::DIM)),
                        Span::from("|"),
                        Span::styled("Calendar", Style::default().add_modifier(Modifier::BOLD)),
                    ]))
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .year(self.calendar_year)
            .date_style(dates_with_styles)
            .months_per_row(self.config.uda_calendar_months_per_row);
        c.title_background_color = self.config.uda_style_calendar_title.bg;
        f.render_widget(c, rects[0]);
    }

    pub fn get_dates_with_styles(&self) -> Vec<(NaiveDate, Style)> {
        let mut tasks_with_styles = vec![];

        let tasks_is_empty = self.tasks.lock().unwrap().is_empty();
        let tasks_len = self.tasks.lock().unwrap().len();

        if !tasks_is_empty {
            let tasks = &self.tasks.lock().unwrap();
            let tasks_with_due_dates = tasks.iter().filter(|t| t.due().is_some());

            tasks_with_styles
                .extend(tasks_with_due_dates.map(|t| (t.due().unwrap().clone().date(), self.style_for_task(t))))
        }
        tasks_with_styles
    }

    pub fn draw_task(&mut self, f: &mut Frame<impl Backend>) {
        let tasks_is_empty = self.tasks.lock().unwrap().is_empty();
        let tasks_len = self.tasks.lock().unwrap().len();
        while !tasks_is_empty && self.task_table_state.selected().unwrap_or_default() >= tasks_len {
            self.task_report_previous();
        }
        let rects = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
            .split(f.size());

        if !self.task_report_show_info {
            let full_table_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(100)].as_ref())
                .split(rects[0]);

            self.task_report_height = full_table_layout[0].height;
            self.draw_task_report(f, full_table_layout[0]);
        } else {
            let split_task_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(rects[0]);

            self.task_report_height = split_task_layout[0].height;
            self.draw_task_report(f, split_task_layout[0]);
            self.draw_task_details(f, split_task_layout[1]);
        }
        let selected = self.task_table_state.selected().unwrap_or_default();
        let task_id = if tasks_len == 0 {
            0
        } else {
            self.tasks.lock().unwrap()[selected].id().unwrap_or_default()
        };
        match self.mode {
            AppMode::TaskReport => self.draw_command(f, rects[1], self.filter.as_str(), "Filter Tasks"),
            AppMode::TaskFilter => {
                f.set_cursor(rects[1].x + self.filter.pos() as u16 + 1, rects[1].y + 1);
                f.render_widget(Clear, rects[1]);
                self.draw_command(
                    f,
                    rects[1],
                    self.filter.as_str(),
                    Span::styled("Filter Tasks", Style::default().add_modifier(Modifier::BOLD)),
                );
            }
            AppMode::TaskModify => {
                f.set_cursor(rects[1].x + self.modify.pos() as u16 + 1, rects[1].y + 1);
                f.render_widget(Clear, rects[1]);
                self.draw_command(
                    f,
                    rects[1],
                    self.modify.as_str(),
                    Span::styled(
                        format!("Modify Task {}", task_id).as_str(),
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                );
            }
            AppMode::TaskLog => {
                f.set_cursor(rects[1].x + self.command.pos() as u16 + 1, rects[1].y + 1);
                f.render_widget(Clear, rects[1]);
                self.draw_command(
                    f,
                    rects[1],
                    self.command.as_str(),
                    Span::styled("Log Tasks", Style::default().add_modifier(Modifier::BOLD)),
                );
            }
            AppMode::TaskSubprocess => {
                f.set_cursor(rects[1].x + self.command.pos() as u16 + 1, rects[1].y + 1);
                f.render_widget(Clear, rects[1]);
                self.draw_command(
                    f,
                    rects[1],
                    self.command.as_str(),
                    Span::styled("Shell Command", Style::default().add_modifier(Modifier::BOLD)),
                );
            }
            AppMode::TaskAnnotate => {
                f.set_cursor(rects[1].x + self.command.pos() as u16 + 1, rects[1].y + 1);
                f.render_widget(Clear, rects[1]);
                self.draw_command(
                    f,
                    rects[1],
                    self.command.as_str(),
                    Span::styled(
                        format!("Annotate Task {}", task_id).as_str(),
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                );
            }
            AppMode::TaskAdd => {
                f.set_cursor(rects[1].x + self.command.pos() as u16 + 1, rects[1].y + 1);
                f.render_widget(Clear, rects[1]);
                self.draw_command(
                    f,
                    rects[1],
                    self.command.as_str(),
                    Span::styled("Add Task", Style::default().add_modifier(Modifier::BOLD)),
                );
            }
            AppMode::TaskError => {
                f.render_widget(Clear, rects[1]);
                self.draw_command(
                    f,
                    rects[1],
                    self.error.as_str(),
                    Span::styled("Error", Style::default().add_modifier(Modifier::BOLD)),
                );
            }
            AppMode::TaskHelpPopup => {
                self.draw_command(f, rects[1], self.filter.as_str(), "Filter Tasks");
                self.draw_help_popup(f);
            }
            AppMode::TaskContextMenu => {
                self.draw_command(f, rects[1], self.filter.as_str(), "Filter Tasks");
                self.draw_context_menu(f);
            }
            _ => {
                panic!("Reached unreachable code. Something went wrong");
            }
        }
    }

    fn draw_help_popup(&self, f: &mut Frame<impl Backend>) {
        let area = centered_rect(80, 90, f.size());
        f.render_widget(Clear, area);
        f.render_widget(&self.help_popup, area);
    }

    fn draw_context_menu(&mut self, f: &mut Frame<impl Backend>) {
        let rects = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0)].as_ref())
            .split(f.size());

        let area = centered_rect(80, 50, f.size());

        f.render_widget(Clear, area.inner(&Margin {
            vertical: 0, horizontal: 0,
        }));

        let maximum_column_width = area.width;

        let (contexts, headers) = self.get_all_contexts();
        let widths = self.calculate_widths(&contexts, &headers, maximum_column_width);

        let selected = self.context_table_state.selected().unwrap_or_default();
        let header = headers.iter();
        let mut rows = vec![];
        let mut highlight_style = Style::default();
        for (i, context) in contexts.iter().enumerate() {
            let mut style = Style::default();
            if &self.contexts[i].active == "yes" {
                style = style
                    .fg(self.config.uda_style_context_active.fg)
                    .bg(self.config.uda_style_context_active.bg)
            }
            rows.push(Row::StyledData(context.iter(), style));
            if i == self.context_table_state.selected().unwrap_or_default() {
                highlight_style = style;
            }
        }

        let constraints: Vec<Constraint> = widths
            .iter()
            .map(|i| Constraint::Length((*i).try_into().unwrap_or(maximum_column_width as u16)))
            .collect();

        let highlight_style = highlight_style.add_modifier(Modifier::BOLD);
        let t = Table::new(header, rows.into_iter())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(Spans::from(vec![
                        Span::styled("Context", Style::default().add_modifier(Modifier::BOLD)),
                    ])),
            )
            .highlight_style(highlight_style)
            .highlight_symbol(&self.config.uda_selection_indicator)
            .widths(&constraints);

        f.render_stateful_widget(t, area, &mut self.context_table_state);
    }

    fn draw_command<'a, T>(&self, f: &mut Frame<impl Backend>, rect: Rect, text: &str, title: T)
    where
        T: Into<Spans<'a>>,
    {
        let p = Paragraph::new(Text::from(text)).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(title.into()),
        );
        f.render_widget(p, rect);
    }

    fn draw_task_details(&mut self, f: &mut Frame<impl Backend>, rect: Rect) {
        if self.tasks.lock().unwrap().is_empty() {
            f.render_widget(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title("Task not found"),
                rect,
            );
            return;
        }
        let selected = self.task_table_state.selected().unwrap_or_default();
        let task_id = self.tasks.lock().unwrap()[selected].id().unwrap_or_default();
        let output = Command::new("task").arg(format!("{}", task_id)).output();
        if let Ok(output) = output {
            let data = String::from_utf8_lossy(&output.stdout);
            let p = Paragraph::new(Text::from(&data[..])).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
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
        let virtual_tag_names_in_precedence = &self.config.rule_precedence_color;

        let mut style = Style::default();

        for tag_name in virtual_tag_names_in_precedence {
            if task
                .tags()
                .unwrap_or(&vec![])
                .contains(&tag_name.to_string().replace(".", "").to_uppercase())
            {
                let color_tag_name = format!("color.{}", tag_name);
                let c = self.config.color.get(&color_tag_name).cloned().unwrap_or_default();
                style = style.fg(c.fg).bg(c.bg);
                for modifier in c.modifiers {
                    style = style.add_modifier(modifier);
                }
                break;
            }
        }

        style
    }

    pub fn calculate_widths(&self, tasks: &Vec<Vec<String>>, headers: &Vec<String>, maximum_column_width: u16) -> Vec<usize> {

        // naive implementation of calculate widths
        let mut widths = headers.iter().map(|s| s.len()).collect::<Vec<usize>>();

        for row in tasks.iter() {
            for (i, cell) in row.iter().enumerate() {
                widths[i] = std::cmp::max(cell.len() + 2, widths[i]);
            }
        }

        for (i, header) in headers.iter().enumerate() {
            if header == "Description" || header == "Definition" {
                // always give description or definition the most room to breath
                widths[i] = maximum_column_width as usize;
                break
            }
        }

        // now start trimming
        while (widths.iter().sum::<usize>() as u16) >= maximum_column_width - 5 {
            let index = widths.iter().position(|i| i == widths.iter().max().unwrap()).unwrap();
            widths[index] -= 1;
        }

        return widths
    }

    fn draw_task_report(&mut self, f: &mut Frame<impl Backend>, rect: Rect) {
        let (tasks, headers) = self.get_task_report();
        if tasks.is_empty() {
            let mut style = Style::default();
            match self.mode {
                AppMode::TaskReport => style = style.add_modifier(Modifier::BOLD),
                _ => style = style.add_modifier(Modifier::DIM),
            }
            f.render_widget(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(Spans::from(vec![
                        Span::styled("Task", style),
                        Span::from("|"),
                        Span::styled("Calendar", Style::default().add_modifier(Modifier::DIM)),
                    ])),
                rect,
            );
            return;
        }

        let maximum_column_width = rect.width;
        let widths = self.calculate_widths(&tasks, &headers, maximum_column_width);

        let selected = self.task_table_state.selected().unwrap_or_default();
        let header = headers.iter();
        let mut rows = vec![];
        let mut highlight_style = Style::default();
        for (i, task) in tasks.iter().enumerate() {
            let style = self.style_for_task(&self.tasks.lock().unwrap()[i]);
            if i == selected {
                highlight_style = style;
                if self.config.uda_selection_bold {
                    highlight_style = highlight_style.add_modifier(Modifier::BOLD);
                }
                if self.config.uda_selection_italic {
                    highlight_style = highlight_style.add_modifier(Modifier::ITALIC);
                }
                if self.config.uda_selection_dim {
                    highlight_style = highlight_style.add_modifier(Modifier::DIM);
                }
                if self.config.uda_selection_blink {
                    highlight_style = highlight_style.add_modifier(Modifier::SLOW_BLINK);
                }
            }
            rows.push(Row::StyledData(task.iter(), style));
        }

        let constraints: Vec<Constraint> = widths
            .iter()
            .map(|i| Constraint::Length((*i).try_into().unwrap_or(maximum_column_width as u16)))
            .collect();

        let mut style = Style::default();
        match self.mode {
            AppMode::TaskReport => style = style.add_modifier(Modifier::BOLD),
            _ => style = style.add_modifier(Modifier::DIM),
        }
        let t = Table::new(header, rows.into_iter())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(Spans::from(vec![
                        Span::styled("Task", style),
                        Span::from("|"),
                        Span::styled("Calendar", Style::default().add_modifier(Modifier::DIM)),
                    ])),
            )
            .highlight_style(highlight_style)
            .highlight_symbol(&self.config.uda_selection_indicator)
            .widths(&constraints);

        f.render_stateful_widget(t, rect, &mut self.task_table_state);
    }

    pub fn get_all_contexts(&self) -> (Vec<Vec<String>>, Vec<String>) {
        let contexts = self.contexts.iter().map(|c| vec![c.name.clone(), c.description.clone(), c.active.clone()]).collect();
        let headers = vec!["Name".to_string(), "Description".to_string(), "Active".to_string()];
        (contexts, headers)
    }

    pub fn get_task_report(&mut self) -> (Vec<Vec<String>>, Vec<String>) {
        let alltasks = &*(self.tasks.lock().unwrap());

        self.task_report_table.generate_table(alltasks);

        let (tasks, headers) = self.task_report_table.simplify_table();

        (tasks, headers)
    }

    pub fn update(&mut self) -> Result<(), Box<dyn Error>> {
        self.task_report_table.export_headers()?;
        self.export_tasks()?;
        self.export_contexts()?;
        self.update_tags();
        Ok(())
    }

    pub fn context_next(&mut self) {
        let i = match self.context_table_state.selected() {
            Some(i) => {
                if i >= self.contexts.len() - 1 {
                    0
                } else {
                    i + 1
                }
            },
            None => 0,
        };
        self.context_table_state.select(Some(i));
    }

    pub fn context_previous(&mut self) {
        let i = match self.context_table_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.contexts.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.context_table_state.select(Some(i));
    }

    pub fn context_select(&mut self) {
        let i = self.context_table_state.selected().unwrap();
        let output = Command::new("task")
            .arg("context")
            .arg(&self.contexts[i].name)
            .output();
    }

    pub fn task_report_top(&mut self) {
        if self.tasks.lock().unwrap().is_empty() {
            return;
        }
        self.task_table_state.select(Some(0));
    }

    pub fn task_report_bottom(&mut self) {
        if self.tasks.lock().unwrap().is_empty() {
            return;
        }
        self.task_table_state.select(Some(self.tasks.lock().unwrap().len() - 1));
    }

    pub fn task_report_next(&mut self) {
        if self.tasks.lock().unwrap().is_empty() {
            return;
        }
        let i = match self.task_table_state.selected() {
            Some(i) => {
                if i >= self.tasks.lock().unwrap().len() - 1 {
                    if self.config.uda_task_report_looping {
                        0
                    } else {
                        i
                    }
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.task_table_state.select(Some(i));
    }

    pub fn task_report_previous(&mut self) {
        if self.tasks.lock().unwrap().is_empty() {
            return;
        }
        let i = match self.task_table_state.selected() {
            Some(i) => {
                if i == 0 {
                    if self.config.uda_task_report_looping {
                        self.tasks.lock().unwrap().len() - 1
                    } else {
                        0
                    }
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.task_table_state.select(Some(i));
    }

    pub fn task_report_next_page(&mut self) {
        if self.tasks.lock().unwrap().is_empty() {
            return;
        }
        let i = match self.task_table_state.selected() {
            Some(i) => {
                if i >= self.tasks.lock().unwrap().len() - 1 {
                    if self.config.uda_task_report_looping {
                        0
                    } else {
                        i
                    }
                } else {
                    i.checked_add(self.task_report_height as usize)
                        .unwrap_or(self.tasks.lock().unwrap().len())
                }
            }
            None => 0,
        };
        self.task_table_state.select(Some(i));
    }

    pub fn task_report_previous_page(&mut self) {
        if self.tasks.lock().unwrap().is_empty() {
            return;
        }
        let i = match self.task_table_state.selected() {
            Some(i) => {
                if i == 0 {
                    if self.config.uda_task_report_looping {
                        self.tasks.lock().unwrap().len() - 1
                    } else {
                        0
                    }
                } else {
                    i.checked_sub(self.task_report_height as usize).unwrap_or(0)
                }
            }
            None => 0,
        };
        self.task_table_state.select(Some(i));
    }

    pub fn export_contexts(&mut self) -> Result<(), Box<dyn Error>> {
        let output = Command::new("task").arg("context").output()?;
        let data = String::from_utf8_lossy(&output.stdout);

        self.contexts = vec![];

        for (i, line) in data.trim().split('\n').enumerate() {
            let line = line.trim();
            let mut s = line.split(" ");
            let name = s.next().unwrap_or_default();
            let active = s.last().unwrap_or_default();
            let definition = line.replacen(name, "", 1);
            let definition = definition.strip_suffix(active).unwrap();
            if i == 0 || i == 1 {
                continue
            } else {
                let context = Context::new(name.to_string(), definition.trim().to_string(), active.to_string());
                self.contexts.push(context);
            }
        }
        if self.contexts.iter().any(|r| r.active != "no") {
            self.contexts.insert(0, Context::new("none".to_string(), "".to_string(), "no".to_string()))
        } else {
            self.contexts.insert(0, Context::new("none".to_string(), "".to_string(), "yes".to_string()))
        }

        Ok(())
    }

    pub fn export_tasks(&mut self) -> Result<(), Box<dyn Error>> {
        let mut task = Command::new("task");

        task.arg("rc.json.array=on");
        task.arg("rc.confirmation=off");
        task.arg("export");

        let filter = if self.current_context_filter != "" {
            let t = format!("{} {}", self.filter.as_str(), self.current_context_filter);
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

        let output = task.output()?;
        let data = String::from_utf8_lossy(&output.stdout);
        let error = String::from_utf8_lossy(&output.stderr);
        if !error.contains("The expression could not be evaluated.") {
            let imported = import(data.as_bytes())?;
            *(self.tasks.lock().unwrap()) = imported;
            self.tasks.lock().unwrap().sort_by(cmp);
        }
        Ok(())
    }

    pub fn task_subprocess(&mut self) -> Result<(), String> {
        if self.tasks.lock().unwrap().is_empty() {
            return Ok(());
        }

        let shell = self.command.as_str().replace("'", "\\'");

        match shlex::split(&shell) {
            Some(cmd) => {
                // first argument must be a binary
                let mut command = Command::new(&cmd[0]);
                // remaining arguments are args
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
                    Err(_) => Err(format!("Shell command `{}` exited with non-zero output", shell,)),
                }
            }
            None => Err(format!("Cannot run subprocess. Unable to shlex split `{}`", shell)),
        }
    }

    pub fn task_log(&mut self) -> Result<(), String> {
        if self.tasks.lock().unwrap().is_empty() {
            return Ok(());
        }

        let mut command = Command::new("task");

        command.arg("log");

        let shell = self.command.as_str().replace("'", "\\'");

        match shlex::split(&shell) {
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
                    Err(_) => Err(format!(
                        "Cannot run `task log {}`. Check documentation for more information",
                        shell
                    )),
                }
            }
            None => Err(format!(
                "Unable to run `task log`. Cannot shlex split `{}`",
                shell.as_str()
            )),
        }
    }

    pub fn task_modify(&mut self) -> Result<(), String> {
        if self.tasks.lock().unwrap().is_empty() {
            return Ok(());
        }
        let selected = self.task_table_state.selected().unwrap_or_default();
        let task_id = self.tasks.lock().unwrap()[selected].id().unwrap_or_default();
        let mut command = Command::new("task");
        command.arg(format!("{}", task_id)).arg("modify");

        let shell = self.modify.as_str().replace("'", "\\'");

        match shlex::split(&shell) {
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
                        "Cannot run `task {} modify {}`. Check documentation for more information",
                        task_id, shell,
                    )),
                }
            }
            None => Err(format!(
                "Unable to run `task {} modify`. Cannot shlex split `{}`",
                task_id, shell,
            )),
        }
    }

    pub fn task_annotate(&mut self) -> Result<(), String> {
        if self.tasks.lock().unwrap().is_empty() {
            return Ok(());
        }
        let selected = self.task_table_state.selected().unwrap_or_default();
        let task_id = self.tasks.lock().unwrap()[selected].id().unwrap_or_default();
        let mut command = Command::new("task");
        command.arg(format!("{}", task_id)).arg("annotate");

        let shell = self.command.as_str().replace("'", "\\'");

        match shlex::split(&shell) {
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
                    Err(_) => Err(format!(
                        "Cannot run `task {} annotate {}`. Check documentation for more information",
                        task_id, shell
                    )),
                }
            }
            None => Err(format!(
                "Unable to run `task {} annotate`. Cannot shlex split `{}`",
                task_id, shell
            )),
        }
    }

    pub fn task_add(&mut self) -> Result<(), String> {
        let mut command = Command::new("task");
        command.arg("add");

        let shell = self.command.as_str().replace("'", "\\'");

        match shlex::split(&shell) {
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
                    Err(_) => Err(format!(
                        "Cannot run `task add {}`. Check documentation for more information",
                        shell
                    )),
                }
            }
            None => Err(format!("Unable to run `task add`. Cannot shlex split `{}`", shell)),
        }
    }

    pub fn task_virtual_tags(task_id: u64) -> Result<String, String> {
        let output = Command::new("task").arg(format!("{}", task_id)).output();

        match output {
            Ok(output) => {
                let data = String::from_utf8_lossy(&output.stdout);
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
        let selected = self.task_table_state.selected().unwrap_or_default();
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
        let selected = self.task_table_state.selected().unwrap_or_default();
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
        let selected = self.task_table_state.selected().unwrap_or_default();
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
        let selected = self.task_table_state.selected().unwrap_or_default();
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
                                String::from_utf8_lossy(&output.stdout),
                                String::from_utf8_lossy(&output.stderr),
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
        let selected = self.task_table_state.selected().unwrap_or_default();
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
                if !task
                    .tags()
                    .unwrap()
                    .iter()
                    .filter(|s| !self.task_report_table.virtual_tags.contains(s))
                    .next()
                    .is_none()
                {
                    add_tag(&mut task, "TAGGED".to_string());
                }
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
                        DateState::EarlierToday | DateState::LaterToday => {
                            add_tag(&mut task, "TODAY".to_string());
                            add_tag(&mut task, "DUETODAY".to_string());
                        }
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

    pub fn handle_input(
        &mut self,
        input: Key,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        events: &Events,
    ) -> Result<(), Box<dyn Error>> {
        match self.mode {
            AppMode::TaskReport => match input {
                Key::Ctrl('c') | Key::Char('q') => self.should_quit = true,
                Key::Char('r') => self.update()?,
                Key::Home | Key::Char('g') => self.task_report_top(),
                Key::End | Key::Char('G') => self.task_report_bottom(),
                Key::Down | Key::Char('j') => self.task_report_next(),
                Key::Up | Key::Char('k') => self.task_report_previous(),
                Key::PageDown | Key::Char('J') => self.task_report_next_page(),
                Key::PageUp | Key::Char('K') => self.task_report_previous_page(),
                Key::Char('d') => match self.task_done() {
                    Ok(_) => self.update()?,
                    Err(e) => {
                        self.mode = AppMode::TaskError;
                        self.error = e;
                    }
                },
                Key::Char('x') => match self.task_delete() {
                    Ok(_) => self.update()?,
                    Err(e) => {
                        self.mode = AppMode::TaskError;
                        self.error = e;
                    }
                },
                Key::Char('s') => match self.task_start_or_stop() {
                    Ok(_) => self.update()?,
                    Err(e) => {
                        self.mode = AppMode::TaskError;
                        self.error = e;
                    }
                },
                Key::Char('u') => match self.task_undo() {
                    Ok(_) => self.update()?,
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
                        Ok(_) => self.update()?,
                        Err(e) => {
                            self.mode = AppMode::TaskError;
                            self.error = e;
                        }
                    }
                }
                Key::Char('m') => {
                    self.mode = AppMode::TaskModify;
                    match self.task_current() {
                        Some(t) => {
                            let s = format!("{} ", t.description());
                            self.modify.update(&s, s.len())
                        }
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
                Key::Char('z') => {
                    self.task_report_show_info = !self.task_report_show_info;
                }
                Key::Char('c') => {
                    self.mode = AppMode::TaskContextMenu;
                }
                Key::Char(']') => {
                    self.mode = AppMode::Calendar;
                }
                _ => {}
            },
            AppMode::TaskContextMenu => match input {
                Key::Esc | Key::Char('q') => {
                    self.mode = AppMode::TaskReport;
                },
                Key::Down | Key::Char('j') => self.context_next(),
                Key::Up | Key::Char('k') => self.context_previous(),
                Key::Char('\n') => {
                    self.context_select();
                    self.get_context()?;
                }
                _ => {}
            },
            AppMode::TaskHelpPopup => match input {
                Key::Esc | Key::Char('q') => {
                    self.mode = AppMode::TaskReport;
                }
                Key::Char('j') => {
                    self.help_popup.scroll = self.help_popup.scroll.checked_add(1).unwrap_or(0);
                    let th = (self.help_popup.text_height as u16).checked_sub(1).unwrap_or(0);
                    if self.help_popup.scroll > th {
                        self.help_popup.scroll = th
                    }
                }
                Key::Char('k') => {
                    self.help_popup.scroll = self.help_popup.scroll.checked_sub(1).unwrap_or(0);
                }
                _ => {}
            },
            AppMode::TaskModify => match input {
                Key::Char('\n') => match self.task_modify() {
                    Ok(_) => {
                        self.mode = AppMode::TaskReport;
                        self.update()?;
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
                        self.update()?;
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
                        self.update()?;
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
                        self.update()?;
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
                        self.update()?;
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
                    self.update()?;
                }
                _ => handle_movement(&mut self.filter, input),
            },
            AppMode::TaskError => self.mode = AppMode::TaskReport,
            AppMode::Calendar => match input {
                Key::Ctrl('c') | Key::Char('q') => self.should_quit = true,
                Key::Char('[') => {
                    self.mode = AppMode::TaskReport;
                }
                Key::Up | Key::Char('k') => {
                    if self.calendar_year > 0 {
                        self.calendar_year -= 1
                    }
                }
                Key::Down | Key::Char('j') => self.calendar_year += 1,
                Key::PageUp | Key::Char('K') => {
                    if self.calendar_year > 0 {
                        self.calendar_year -= 10
                    }
                }
                Key::PageDown | Key::Char('J') => self.calendar_year += 10,
                _ => {}
            },
        }
        Ok(())
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
        let mut app = TTApp::new().unwrap();

        let (contexts, headers) = app.get_all_contexts();
        dbg!(contexts);
        dbg!(headers);

        //println!("{:?}", app.task_report_columns);
        //println!("{:?}", app.task_report_labels);

        // let (t, h, c) = app.get_task_report();
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
