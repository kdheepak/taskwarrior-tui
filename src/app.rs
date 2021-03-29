use crate::calendar::Calendar;
use crate::config;
use crate::config::Config;
use crate::context::Context;
use crate::help::Help;
use crate::keyconfig::KeyConfig;
use crate::table::{Row, Table, TableMode, TableState};
use crate::task_report::TaskReportTable;
use crate::util::Key;
use crate::util::{Event, Events};

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::convert::TryInto;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::SystemTime;

use task_hookrs::date::Date;
use task_hookrs::import::import;
use task_hookrs::status::TaskStatus;
use task_hookrs::task::Task;
use uuid::Uuid;

use unicode_segmentation::Graphemes;
use unicode_segmentation::UnicodeSegmentation;

use chrono::{Datelike, Local, NaiveDate, NaiveDateTime, TimeZone};

use anyhow::Result;

use async_std::task;

use std::sync::{Arc, Mutex};
use std::{sync::mpsc, thread, time::Duration};
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
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

use std::io;
use tui::{backend::CrosstermBackend, Terminal};

use regex::Regex;

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

#[derive(Debug)]
pub enum DateState {
    BeforeToday,
    EarlierToday,
    LaterToday,
    AfterToday,
    NotDue,
}

pub fn get_date_state(reference: &Date, due: usize) -> DateState {
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

    if reference <= now + chrono::Duration::days(7) {
        DateState::AfterToday
    } else {
        DateState::NotDue
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
    TaskContextMenu,
    Calendar,
}

pub struct TaskwarriorTuiApp {
    pub should_quit: bool,
    pub task_table_state: TableState,
    pub context_table_state: TableState,
    pub current_context_filter: String,
    pub current_context: String,
    pub command: LineBuffer,
    pub filter: LineBuffer,
    pub modify: LineBuffer,
    pub error: String,
    pub tasks: Vec<Task>,
    pub task_details: HashMap<Uuid, String>,
    pub marked: HashSet<Uuid>,
    // stores index of current task that is highlighted
    pub current_selection: usize,
    pub task_report_table: TaskReportTable,
    pub calendar_year: i32,
    pub mode: AppMode,
    pub config: Config,
    pub task_report_show_info: bool,
    pub task_report_height: u16,
    pub task_details_scroll: u16,
    pub help_popup: Help,
    pub contexts: Vec<Context>,
    pub last_export: Option<SystemTime>,
    pub keyconfig: KeyConfig,
    pub terminal_width: u16,
    pub terminal_height: u16,
}

impl TaskwarriorTuiApp {
    pub fn new() -> Result<Self> {
        let c = Config::default()?;
        let mut kc = KeyConfig::default();
        kc.update()?;
        let mut app = Self {
            should_quit: false,
            task_table_state: TableState::default(),
            context_table_state: TableState::default(),
            tasks: vec![],
            task_details: HashMap::new(),
            marked: HashSet::new(),
            current_selection: 0,
            current_context_filter: "".to_string(),
            current_context: "".to_string(),
            command: LineBuffer::with_capacity(MAX_LINE),
            filter: LineBuffer::with_capacity(MAX_LINE),
            modify: LineBuffer::with_capacity(MAX_LINE),
            error: "".to_string(),
            mode: AppMode::TaskReport,
            task_report_height: 0,
            task_details_scroll: 0,
            task_report_show_info: c.uda_task_report_show_info,
            config: c,
            task_report_table: TaskReportTable::new()?,
            calendar_year: Local::today().year(),
            help_popup: Help::new(),
            contexts: vec![],
            last_export: None,
            keyconfig: kc,
            terminal_width: 0,
            terminal_height: 0,
        };
        for c in app.config.filter.chars() {
            app.filter.insert(c, 1);
        }
        app.get_context()?;
        task::block_on(app.update(true))?;
        Ok(app)
    }

    pub fn get_context(&mut self) -> Result<()> {
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
        let rect = f.size();
        self.terminal_width = rect.width;
        self.terminal_height = rect.height;
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

    pub fn draw_debug(&mut self, f: &mut Frame<impl Backend>) {
        let area = centered_rect(50, 50, f.size());
        f.render_widget(Clear, area);
        let t = format!("{:?}", self.marked);
        let p = Paragraph::new(Text::from(t))
            .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded));
        f.render_widget(p, area);
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
        c.title_background_color = self.config.uda_style_calendar_title.bg.unwrap_or(Color::Black);
        f.render_widget(c, rects[0]);
    }

    pub fn get_dates_with_styles(&self) -> Vec<(NaiveDate, Style)> {
        let mut tasks_with_styles = vec![];

        let tasks_is_empty = self.tasks.is_empty();
        let tasks_len = self.tasks.len();

        if !tasks_is_empty {
            let tasks = &self.tasks;
            let tasks_with_due_dates = tasks.iter().filter(|t| t.due().is_some());

            tasks_with_styles
                .extend(tasks_with_due_dates.map(|t| (t.due().unwrap().clone().date(), self.style_for_task(t))))
        }
        tasks_with_styles
    }

    pub fn get_position(&self, lb: &LineBuffer) -> usize {
        let mut position = lb.as_str().graphemes(true).count();
        for (i, (_i, g)) in lb.as_str().grapheme_indices(true).enumerate() {
            if _i == lb.pos() {
                position = i;
                break;
            }
        }
        position
    }

    pub fn draw_task(&mut self, f: &mut Frame<impl Backend>) {
        let tasks_is_empty = self.tasks.is_empty();
        let tasks_len = self.tasks.len();
        while !tasks_is_empty && self.current_selection >= tasks_len {
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
        let selected = self.current_selection;
        let task_ids = if tasks_len == 0 {
            vec!["0".to_string()]
        } else {
            match self.task_table_state.mode() {
                TableMode::SingleSelection => vec![self.tasks[selected].id().unwrap_or_default().to_string()],
                TableMode::MultipleSelection => {
                    let mut tids = vec![];
                    for uuid in self.marked.iter() {
                        if let Some(t) = self.task_by_uuid(*uuid) {
                            tids.push(t.id().unwrap_or_default().to_string());
                        }
                    }
                    tids
                }
            }
        };
        match self.mode {
            AppMode::TaskReport => self.draw_command(f, rects[1], self.filter.as_str(), "Filter Tasks"),
            AppMode::TaskFilter => {
                let position = self.get_position(&self.filter);
                f.set_cursor(rects[1].x + position as u16 + 1, rects[1].y + 1);
                f.render_widget(Clear, rects[1]);
                self.draw_command(
                    f,
                    rects[1],
                    self.filter.as_str(),
                    Span::styled("Filter Tasks", Style::default().add_modifier(Modifier::BOLD)),
                );
            }
            AppMode::TaskLog => {
                let position = self.get_position(&self.command);
                f.set_cursor(rects[1].x + position as u16 + 1, rects[1].y + 1);
                f.render_widget(Clear, rects[1]);
                self.draw_command(
                    f,
                    rects[1],
                    self.command.as_str(),
                    Span::styled("Log Tasks", Style::default().add_modifier(Modifier::BOLD)),
                );
            }
            AppMode::TaskSubprocess => {
                let position = self.get_position(&self.command);
                f.set_cursor(rects[1].x + position as u16 + 1, rects[1].y + 1);
                f.render_widget(Clear, rects[1]);
                self.draw_command(
                    f,
                    rects[1],
                    self.command.as_str(),
                    Span::styled("Shell Command", Style::default().add_modifier(Modifier::BOLD)),
                );
            }
            AppMode::TaskModify => {
                let position = self.get_position(&self.modify);
                f.set_cursor(rects[1].x + position as u16 + 1, rects[1].y + 1);
                f.render_widget(Clear, rects[1]);
                let label = if task_ids.len() > 1 {
                    format!("Modify Tasks {}", task_ids.join(","))
                } else {
                    format!("Modify Task {}", task_ids.join(","))
                };
                self.draw_command(
                    f,
                    rects[1],
                    self.modify.as_str(),
                    Span::styled(label, Style::default().add_modifier(Modifier::BOLD)),
                );
            }
            AppMode::TaskAnnotate => {
                let position = self.get_position(&self.command);
                f.set_cursor(rects[1].x + position as u16 + 1, rects[1].y + 1);
                f.render_widget(Clear, rects[1]);
                let label = if task_ids.len() > 1 {
                    format!("Annotate Tasks {}", task_ids.join(","))
                } else {
                    format!("Annotate Task {}", task_ids.join(","))
                };
                self.draw_command(
                    f,
                    rects[1],
                    self.command.as_str(),
                    Span::styled(label, Style::default().add_modifier(Modifier::BOLD)),
                );
            }
            AppMode::TaskAdd => {
                let position = self.get_position(&self.command);
                f.set_cursor(rects[1].x + position as u16 + 1, rects[1].y + 1);
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
                self.draw_help_popup(f, 80, 90);
            }
            AppMode::TaskContextMenu => {
                self.draw_command(f, rects[1], self.filter.as_str(), "Filter Tasks");
                self.draw_context_menu(f, 80, 50);
            }
            _ => {
                panic!("Reached unreachable code. Something went wrong");
            }
        }
    }

    fn draw_help_popup(&mut self, f: &mut Frame<impl Backend>, percent_x: u16, percent_y: u16) {
        let area = centered_rect(percent_x, percent_y, f.size());
        f.render_widget(Clear, area);
        self.help_popup.scroll = std::cmp::min(
            self.help_popup.scroll,
            (self.help_popup.text_height as u16).saturating_sub(area.height),
        );
        f.render_widget(&self.help_popup, area);
    }

    fn draw_context_menu(&mut self, f: &mut Frame<impl Backend>, percent_x: u16, percent_y: u16) {
        let rects = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0)].as_ref())
            .split(f.size());

        let area = centered_rect(percent_x, percent_y, f.size());

        f.render_widget(
            Clear,
            area.inner(&Margin {
                vertical: 0,
                horizontal: 0,
            }),
        );

        let (contexts, headers) = self.get_all_contexts();

        let maximum_column_width = area.width;
        let widths = self.calculate_widths(&contexts, &headers, maximum_column_width);

        let selected = self.context_table_state.current_selection().unwrap_or_default();
        let header = headers.iter();
        let mut rows = vec![];
        let mut highlight_style = Style::default();
        for (i, context) in contexts.iter().enumerate() {
            let mut style = Style::default();
            if &self.contexts[i].active == "yes" {
                style = self.config.uda_style_context_active;
            }
            rows.push(Row::StyledData(context.iter(), style));
            if i == self.context_table_state.current_selection().unwrap_or_default() {
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
                    .title(Spans::from(vec![Span::styled(
                        "Context",
                        Style::default().add_modifier(Modifier::BOLD),
                    )])),
            )
            .header_style(
                self.config
                    .color
                    .get("color.label")
                    .cloned()
                    .unwrap_or_default()
                    .add_modifier(Modifier::UNDERLINED),
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
        if self.tasks.is_empty() {
            f.render_widget(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title("Task not found"),
                rect,
            );
            return;
        }
        let selected = self.current_selection;
        let task_id = self.tasks[selected].id().unwrap_or_default();
        let task_uuid = *self.tasks[selected].uuid();

        if !self.task_details.contains_key(&task_uuid) {
            let output = Command::new("task")
                .arg("rc.color=off")
                .arg(format!("rc.defaultwidth={}", self.terminal_width - 2))
                .arg(format!("{}", task_uuid))
                .output();
            let data = if let Ok(output) = output {
                String::from_utf8_lossy(&output.stdout).to_string()
            } else {
                "".to_string()
            };
            let entry = self.task_details.entry(task_uuid).or_insert_with(|| "".to_string());
            *entry = data;
        }
        let data = self.task_details[&task_uuid].clone();

        self.task_details_scroll = std::cmp::min(
            (data.lines().count() as u16).saturating_sub(rect.height),
            self.task_details_scroll,
        );
        let p = Paragraph::new(Text::from(&data[..]))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(format!("Task {}", task_id)),
            )
            .scroll((self.task_details_scroll, 0));
        f.render_widget(p, rect);
    }

    fn task_details_scroll_down(&mut self) {
        self.task_details_scroll = self.task_details_scroll.saturating_sub(1);
    }

    fn task_details_scroll_up(&mut self) {
        self.task_details_scroll = self.task_details_scroll.saturating_add(1);
    }

    fn task_by_index(&self, i: usize) -> Option<Task> {
        let tasks = &self.tasks;
        if i >= tasks.len() {
            None
        } else {
            Some(tasks[i].clone())
        }
    }

    fn task_by_uuid(&self, uuid: Uuid) -> Option<Task> {
        let tasks = &self.tasks;
        let m = tasks.iter().find(|t| *t.uuid() == uuid);
        m.cloned()
    }

    fn task_by_id(&self, id: u64) -> Option<Task> {
        let tasks = &self.tasks;
        let m = tasks.iter().find(|t| t.id().unwrap() == id);
        m.cloned()
    }

    fn task_index_by_uuid(&self, uuid: Uuid) -> Option<usize> {
        let tasks = &self.tasks;
        let m = tasks.iter().position(|t| *t.uuid() == uuid);
        m
    }

    fn style_for_task(&self, task: &Task) -> Style {
        let virtual_tag_names_in_precedence = &self.config.rule_precedence_color;

        let mut style = Style::default();

        for tag_name in virtual_tag_names_in_precedence.iter().rev() {
            if tag_name == "uda." || tag_name == "priority" {
                if let Some(p) = task.priority() {
                    let s = self
                        .config
                        .color
                        .get(&format!("color.uda.priority.{}", p))
                        .cloned()
                        .unwrap_or_default();
                    style = style.patch(s);
                }
            } else if tag_name == "tag." {
                if let Some(tags) = task.tags() {
                    for t in tags {
                        let color_tag_name = format!("color.tag.{}", t);
                        let s = self.config.color.get(&color_tag_name).cloned().unwrap_or_default();
                        style = style.patch(s);
                    }
                }
            } else if tag_name == "project." {
                if let Some(p) = task.project() {
                    let s = self
                        .config
                        .color
                        .get(&format!("color.project.{}", p))
                        .cloned()
                        .unwrap_or_default();
                    style = style.patch(s);
                }
            } else if task
                .tags()
                .unwrap_or(&vec![])
                .contains(&tag_name.to_string().replace(".", "").to_uppercase())
            {
                let color_tag_name = format!("color.{}", tag_name);
                let s = self.config.color.get(&color_tag_name).cloned().unwrap_or_default();
                style = style.patch(s);
            }
        }

        style
    }

    pub fn calculate_widths(&self, tasks: &[Vec<String>], headers: &[String], maximum_column_width: u16) -> Vec<usize> {
        // naive implementation of calculate widths
        let mut widths = headers.iter().map(|s| s.len()).collect::<Vec<usize>>();

        for row in tasks.iter() {
            for (i, cell) in row.iter().enumerate() {
                widths[i] = std::cmp::max(cell.len(), widths[i]);
            }
        }

        for (i, header) in headers.iter().enumerate() {
            if header == "Description" || header == "Definition" {
                // always give description or definition the most room to breath
                widths[i] = maximum_column_width as usize;
                break;
            }
        }
        for (i, header) in headers.iter().enumerate() {
            if header == "ID" || header == "Name" {
                // always give ID a couple of extra for indicator
                widths[i] += self.config.uda_selection_indicator.as_str().graphemes(true).count();
                // if let TableMode::MultipleSelection = self.task_table_state.mode() {
                //     widths[i] += 2
                // };
            }
        }

        // now start trimming
        while (widths.iter().sum::<usize>() as u16) >= maximum_column_width - (headers.len()) as u16 {
            let index = widths.iter().position(|i| i == widths.iter().max().unwrap()).unwrap();
            if widths[index] == 1 {
                break;
            }
            widths[index] -= 1;
        }

        widths
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

        for (i, header) in headers.iter().enumerate() {
            if header == "Description" || header == "Definition" {
                self.task_report_table.description_width = widths[i] - 1;
                break;
            }
        }
        let selected = self.current_selection;
        let header = headers.iter();
        let mut rows = vec![];
        let mut highlight_style = Style::default();
        for (i, task) in tasks.iter().enumerate() {
            let style = self.style_for_task(&self.tasks[i]);
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
            .header_style(
                self.config
                    .color
                    .get("color.label")
                    .cloned()
                    .unwrap_or_default()
                    .add_modifier(Modifier::UNDERLINED),
            )
            .highlight_style(highlight_style)
            .highlight_symbol(&self.config.uda_selection_indicator)
            .mark_symbol(&self.config.uda_mark_indicator)
            .unmark_symbol(&self.config.uda_unmark_indicator)
            .widths(&constraints);

        f.render_stateful_widget(t, rect, &mut self.task_table_state);
    }

    pub fn get_all_contexts(&self) -> (Vec<Vec<String>>, Vec<String>) {
        let contexts = self
            .contexts
            .iter()
            .map(|c| vec![c.name.clone(), c.description.clone(), c.active.clone()])
            .collect();
        let headers = vec!["Name".to_string(), "Description".to_string(), "Active".to_string()];
        (contexts, headers)
    }

    pub fn get_task_report(&mut self) -> (Vec<Vec<String>>, Vec<String>) {
        self.task_report_table.generate_table(&self.tasks);
        let (tasks, headers) = self.task_report_table.simplify_table();
        (tasks, headers)
    }

    pub async fn update(&mut self, force: bool) -> Result<()> {
        if force || self.tasks_changed_since(self.last_export)? {
            self.last_export = Some(std::time::SystemTime::now());
            self.task_report_table.export_headers()?;
            let _ = self.export_tasks().await;
            self.export_contexts().await?;
            self.update_tags();
            self.task_details.clear();
        }
        Ok(())
    }

    pub fn update_task_table_state(&mut self) {
        self.task_table_state.select(Some(self.current_selection));

        for uuid in self.marked.clone() {
            if self.task_by_uuid(uuid).is_none() {
                self.marked.remove(&uuid);
            }
        }

        if self.marked.is_empty() {
            self.task_table_state.single_selection();
        }

        self.task_table_state.clear();

        for uuid in &self.marked {
            self.task_table_state.mark(self.task_index_by_uuid(*uuid))
        }
    }

    pub fn context_next(&mut self) {
        let i = match self.context_table_state.current_selection() {
            Some(i) => {
                if i >= self.contexts.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.context_table_state.select(Some(i));
    }

    pub fn context_previous(&mut self) {
        let i = match self.context_table_state.current_selection() {
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

    pub fn context_select(&mut self) -> Result<(), String> {
        let i = self.context_table_state.current_selection().unwrap();
        let mut command = Command::new("task");
        command.arg("context").arg(&self.contexts[i].name);
        let output = command.output();
        Ok(())
    }

    pub fn task_report_top(&mut self) {
        if self.tasks.is_empty() {
            return;
        }
        self.current_selection = 0;
    }

    pub fn task_report_bottom(&mut self) {
        if self.tasks.is_empty() {
            return;
        }
        self.current_selection = self.tasks.len() - 1;
    }

    pub fn task_report_next(&mut self) {
        if self.tasks.is_empty() {
            return;
        }
        let i = {
            if self.current_selection >= self.tasks.len() - 1 {
                if self.config.uda_task_report_looping {
                    0
                } else {
                    self.current_selection
                }
            } else {
                self.current_selection + 1
            }
        };
        self.current_selection = i;
    }

    pub fn task_report_previous(&mut self) {
        if self.tasks.is_empty() {
            return;
        }
        let i = {
            if self.current_selection == 0 {
                if self.config.uda_task_report_looping {
                    self.tasks.len() - 1
                } else {
                    0
                }
            } else {
                self.current_selection - 1
            }
        };
        self.current_selection = i;
    }

    pub fn task_report_next_page(&mut self) {
        if self.tasks.is_empty() {
            return;
        }
        let i = {
            if self.current_selection >= self.tasks.len() - 1 {
                if self.config.uda_task_report_looping {
                    0
                } else {
                    self.current_selection
                }
            } else {
                self.current_selection
                    .checked_add(self.task_report_height as usize)
                    .unwrap_or_else(|| self.tasks.len())
            }
        };
        self.current_selection = i;
    }

    pub fn task_report_previous_page(&mut self) {
        if self.tasks.is_empty() {
            return;
        }
        let i = {
            if self.current_selection == 0 {
                if self.config.uda_task_report_looping {
                    self.tasks.len() - 1
                } else {
                    0
                }
            } else {
                self.current_selection.saturating_sub(self.task_report_height as usize)
            }
        };
        self.current_selection = i;
    }

    pub async fn export_contexts(&mut self) -> Result<()> {
        let output = async_std::process::Command::new("task").arg("context").output().await?;
        let data = String::from_utf8_lossy(&output.stdout);

        self.contexts = vec![];

        for (i, line) in data.trim().split('\n').enumerate() {
            let line = line.trim();
            if line.is_empty() || line == "Use 'task context none' to unset the current context." {
                continue;
            }
            let mut s = line.split(' ');
            let name = s.next().unwrap_or_default();
            let active = s.last().unwrap_or_default();
            let definition = line.replacen(name, "", 1);
            let definition = definition.strip_suffix(active).unwrap();
            if i == 0 || i == 1 {
                continue;
            } else {
                let context = Context::new(name.to_string(), definition.trim().to_string(), active.to_string());
                self.contexts.push(context);
            }
        }
        if self.contexts.iter().any(|r| r.active != "no") {
            self.contexts
                .insert(0, Context::new("none".to_string(), "".to_string(), "no".to_string()))
        } else {
            self.contexts
                .insert(0, Context::new("none".to_string(), "".to_string(), "yes".to_string()))
        }

        Ok(())
    }

    fn get_task_files_max_mtime(&self) -> Result<SystemTime> {
        let data_dir = shellexpand::tilde(&self.config.data_location).into_owned();
        let mut mtimes = Vec::new();
        for fname in &["backlog.data", "completed.data", "pending.data"] {
            let pending_fp = Path::new(&data_dir).join(fname);
            let mtime = fs::metadata(pending_fp)?.modified()?;
            mtimes.push(mtime);
        }
        Ok(*mtimes.iter().max().unwrap())
    }

    pub fn tasks_changed_since(&mut self, prev: Option<SystemTime>) -> Result<bool> {
        if let Some(prev) = prev {
            match self.get_task_files_max_mtime() {
                Ok(mtime) => {
                    if mtime > prev {
                        Ok(true)
                    } else {
                        // Unfortunately, we can not use std::time::Instant which is guaranteed to be monotonic,
                        // because we need to compare it to a file mtime as SystemTime, so as a safety for unexpected
                        // time shifts, cap maximum wait to 1 min
                        let now = SystemTime::now();
                        let max_delta = Duration::from_secs(60);
                        Ok(now.duration_since(prev)? > max_delta)
                    }
                }
                Err(_) => Ok(true),
            }
        } else {
            Ok(true)
        }
    }

    pub async fn export_tasks(&mut self) -> Result<()> {
        let mut task = async_std::process::Command::new("task");

        task.arg("rc.json.array=on");
        task.arg("rc.confirmation=off");
        task.arg("export");

        let filter = if !self.current_context_filter.is_empty() {
            let t = format!("{} '\\({}\\)'", self.filter.as_str(), self.current_context_filter);
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

        let output = task.output().await?;
        let data = String::from_utf8_lossy(&output.stdout);
        let error = String::from_utf8_lossy(&output.stderr);
        if !error.contains("The expression could not be evaluated.") {
            if let Ok(imported) = import(data.as_bytes()) {
                self.tasks = imported;
                self.tasks.sort_by(cmp);
            }
        }

        Ok(())
    }

    pub fn selected_task_uuids(&self) -> Vec<Uuid> {
        let selected = match self.task_table_state.mode() {
            TableMode::SingleSelection => vec![self.current_selection],
            TableMode::MultipleSelection => self.task_table_state.marked().cloned().collect::<Vec<usize>>(),
        };

        let mut task_uuids = vec![];

        for s in selected {
            let task_id = self.tasks[s].id().unwrap_or_default();
            let task_uuid = *self.tasks[s].uuid();
            task_uuids.push(task_uuid);
        }

        task_uuids
    }

    pub fn task_subprocess(&mut self) -> Result<(), String> {
        if self.tasks.is_empty() {
            return Ok(());
        }

        let shell = self.command.as_str();

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
        if self.tasks.is_empty() {
            return Ok(());
        }

        let mut command = Command::new("task");

        command.arg("log");

        let shell = self.command.as_str();

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
            None => Err(format!("Unable to run `task log`. Cannot shlex split `{}`", shell)),
        }
    }

    pub fn task_shortcut(&mut self, s: usize) -> Result<(), String> {
        if self.tasks.is_empty() {
            return Ok(());
        }

        let task_uuids = self.selected_task_uuids();

        let shell = &self.config.uda_shortcuts[s];

        if shell.is_empty() {
            return Err("Trying to run empty shortcut.".to_string());
        }

        let shell = format!(
            "{} {}",
            shell,
            task_uuids
                .iter()
                .map(|u| u.to_string())
                .collect::<Vec<String>>()
                .join(" ")
        );
        let shell = shellexpand::tilde(&shell).into_owned();
        match shlex::split(&shell) {
            Some(cmd) => {
                let mut command = Command::new(&cmd[0]);
                for s in cmd.iter().skip(1) {
                    command.arg(&s);
                }
                let output = command.output();
                match output {
                    Ok(o) => {
                        if o.status.success() {
                            Ok(())
                        } else {
                            Err(format!(
                                "Unable to run shortcut {}. Failed with status code {}",
                                s,
                                o.status.code().unwrap()
                            ))
                        }
                    }
                    Err(s) => Err(format!("`{}` failed: {}", shell, s,)),
                }
            }
            None => Err(format!("Unable to run `{}`. Cannot shlex split `{}`", shell, shell)),
        }
    }

    pub fn task_modify(&mut self) -> Result<(), String> {
        if self.tasks.is_empty() {
            return Ok(());
        }

        let task_uuids = self.selected_task_uuids();

        let mut command = Command::new("task");
        command.arg("rc.bulk=0");
        command.arg("rc.confirmation=off");
        command.arg("rc.dependency.confirmation=off");
        command.arg("rc.recurrence.confirmation=off");
        for task_uuid in &task_uuids {
            command.arg(task_uuid.to_string());
        }
        command.arg("modify");

        let shell = self.modify.as_str();

        match shlex::split(&shell) {
            Some(cmd) => {
                for s in cmd {
                    command.arg(&s);
                }
                let output = command.output();
                match output {
                    Ok(o) => {
                        if o.status.success() {
                            self.modify.update("", 0);
                            Ok(())
                        } else {
                            Err(format!("Modify failed. {}", String::from_utf8_lossy(&o.stdout),))
                        }
                    }
                    Err(_) => Err(format!(
                        "Cannot run `task {:?} modify {}`. Check documentation for more information",
                        task_uuids, shell,
                    )),
                }
            }
            None => Err(format!("Cannot shlex split `{}`", shell,)),
        }
    }

    pub fn task_annotate(&mut self) -> Result<(), String> {
        if self.tasks.is_empty() {
            return Ok(());
        }

        let task_uuids = self.selected_task_uuids();

        let mut command = Command::new("task");
        command.arg("rc.bulk=0");
        command.arg("rc.confirmation=off");
        command.arg("rc.dependency.confirmation=off");
        command.arg("rc.recurrence.confirmation=off");
        for task_uuid in &task_uuids {
            command.arg(task_uuid.to_string());
        }
        command.arg("annotate");

        let shell = self.command.as_str();

        match shlex::split(&shell) {
            Some(cmd) => {
                for s in cmd {
                    command.arg(&s);
                }
                let output = command.output();
                match output {
                    Ok(o) => {
                        if o.status.success() {
                            self.command.update("", 0);
                            Ok(())
                        } else {
                            Err(format!("Annotate failed. {}", String::from_utf8_lossy(&o.stdout),))
                        }
                    }
                    Err(_) => Err(format!(
                        "Cannot run `task {} annotate {}`. Check documentation for more information",
                        task_uuids
                            .iter()
                            .map(|u| u.to_string())
                            .collect::<Vec<String>>()
                            .join(" "),
                        shell
                    )),
                }
            }
            None => Err(format!("Cannot shlex split `{}`", shell)),
        }
    }

    pub fn task_add(&mut self) -> Result<(), String> {
        let mut command = Command::new("task");
        command.arg("add");

        let shell = self.command.as_str();

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

    pub fn task_virtual_tags(task_uuid: Uuid) -> Result<String, String> {
        let output = Command::new("task").arg(format!("{}", task_uuid)).output();

        match output {
            Ok(output) => {
                let data = String::from_utf8_lossy(&output.stdout);
                for line in data.split('\n') {
                    for prefix in &["Virtual tags", "Virtual"] {
                        if line.starts_with(prefix) {
                            let line = line.to_string();
                            let line = line.replace(prefix, "");
                            return Ok(line);
                        }
                    }
                }
                Err(format!(
                    "Cannot find any tags for `task {}`. Check documentation for more information",
                    task_uuid
                ))
            }
            Err(_) => Err(format!(
                "Cannot run `task {}`. Check documentation for more information",
                task_uuid
            )),
        }
    }

    pub fn task_start_stop(&mut self) -> Result<(), String> {
        if self.tasks.is_empty() {
            return Ok(());
        }
        let selected = self.current_selection;
        let task_id = self.tasks[selected].id().unwrap_or_default();
        let task_uuid = *self.tasks[selected].uuid();

        let task_uuids = self.selected_task_uuids();

        for task_uuid in &task_uuids {
            let mut command = "start";
            for tag in TaskwarriorTuiApp::task_virtual_tags(*task_uuid).unwrap().split(' ') {
                if tag == "ACTIVE" {
                    command = "stop"
                }
            }

            let output = Command::new("task").arg(task_uuid.to_string()).arg(command).output();
            if output.is_err() {
                return Err(format!("Error running `task {}` for task `{}`.", command, task_uuid,));
            }
        }

        Ok(())
    }

    pub fn task_delete(&self) -> Result<(), String> {
        if self.tasks.is_empty() {
            return Ok(());
        }

        let task_uuids = self.selected_task_uuids();

        let mut cmd = Command::new("task");
        cmd.arg("rc.bulk=0")
            .arg("rc.confirmation=off")
            .arg("rc.dependency.confirmation=off")
            .arg("rc.recurrence.confirmation=off");
        for task_uuid in &task_uuids {
            cmd.arg(task_uuid.to_string());
        }
        cmd.arg("delete");
        let output = cmd.output();
        match output {
            Ok(_) => Ok(()),
            Err(_) => Err(format!(
                "Cannot run `task delete` for tasks `{}`. Check documentation for more information",
                task_uuids
                    .iter()
                    .map(|u| u.to_string())
                    .collect::<Vec<String>>()
                    .join(" ")
            )),
        }
    }

    pub fn task_done(&mut self) -> Result<(), String> {
        if self.tasks.is_empty() {
            return Ok(());
        }
        let task_uuids = self.selected_task_uuids();
        let mut cmd = Command::new("task");
        cmd.arg("rc.bulk=0")
            .arg("rc.confirmation=off")
            .arg("rc.dependency.confirmation=off")
            .arg("rc.recurrence.confirmation=off");
        for task_uuid in &task_uuids {
            cmd.arg(task_uuid.to_string());
        }
        cmd.arg("done");
        let output = cmd.output();
        match output {
            Ok(_) => Ok(()),
            Err(_) => Err(format!(
                "Cannot run `task done` for task `{}`. Check documentation for more information",
                task_uuids
                    .iter()
                    .map(|u| u.to_string())
                    .collect::<Vec<String>>()
                    .join(" ")
            )),
        }
    }

    pub fn task_undo(&self) -> Result<(), String> {
        if self.tasks.is_empty() {
            return Ok(());
        }
        let output = Command::new("task").arg("rc.confirmation=off").arg("undo").output();

        match output {
            Ok(_) => Ok(()),
            Err(_) => Err("Cannot run `task undo`. Check documentation for more information".to_string()),
        }
    }

    pub fn task_edit(&self) -> Result<(), String> {
        if self.tasks.is_empty() {
            return Ok(());
        }
        let selected = self.task_table_state.current_selection().unwrap_or_default();
        let task_id = self.tasks[selected].id().unwrap_or_default();
        let task_uuid = *self.tasks[selected].uuid();
        let r = Command::new("task").arg(format!("{}", task_uuid)).arg("edit").spawn();

        match r {
            Ok(child) => {
                let output = child.wait_with_output();
                match output {
                    Ok(output) => {
                        if !output.status.success() {
                            Err(format!(
                                "`task edit` for task `{}` failed. {}{}",
                                task_uuid,
                                String::from_utf8_lossy(&output.stdout),
                                String::from_utf8_lossy(&output.stderr),
                            ))
                        } else {
                            String::from_utf8_lossy(&output.stdout);
                            String::from_utf8_lossy(&output.stderr);
                            Ok(())
                        }
                    }
                    Err(err) => Err(format!("Cannot run `task edit` for task `{}`. {}", task_uuid, err)),
                }
            }
            _ => Err(format!(
                "Cannot start `task edit` for task `{}`. Check documentation for more information",
                task_uuid
            )),
        }
    }

    pub fn task_current(&self) -> Option<Task> {
        if self.tasks.is_empty() {
            return None;
        }
        let selected = self.current_selection;
        Some(self.tasks[selected].clone())
    }

    pub fn update_tags(&mut self) {
        let tasks = &mut self.tasks;

        // dependency scan
        for l_i in 0..tasks.len() {
            let default_deps = vec![];
            let deps = tasks[l_i].depends().unwrap_or(&default_deps).clone();
            add_tag(&mut tasks[l_i], "UNBLOCKED".to_string());
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
                            remove_tag(&mut tasks[l_i], "UNBLOCKED".to_string());
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
            let virtual_tags = self.task_report_table.virtual_tags.clone();
            if task.tags().is_some() && task.tags().unwrap().iter().any(|s| !virtual_tags.contains(s)) {
                add_tag(&mut task, "TAGGED".to_string());
            }
            if !task.uda().is_empty() {
                add_tag(&mut task, "UDA".to_string());
            }
            if task.mask().is_some() {
                add_tag(&mut task, "TEMPLATE".to_string());
            }
            if task.project().is_some() {
                add_tag(&mut task, "PROJECT".to_string());
            }
            if task.priority().is_some() {
                add_tag(&mut task, "PRIORITY".to_string());
            }
            if task.recur().is_some() {
                add_tag(&mut task, "RECURRING".to_string());
                let r = task.recur().unwrap();
            }
            if let Some(d) = task.due() {
                let status = task.status();
                // due today
                if status != &TaskStatus::Completed && status != &TaskStatus::Deleted {
                    let now = Local::now();
                    let reference = TimeZone::from_utc_datetime(now.offset(), d);
                    let now = TimeZone::from_utc_datetime(now.offset(), &now.naive_utc());
                    let d = d.clone();
                    if (reference - chrono::Duration::nanoseconds(1)).month() == now.month() {
                        add_tag(&mut task, "MONTH".to_string());
                    }
                    if (reference - chrono::Duration::nanoseconds(1)).month() % 4 == now.month() % 4 {
                        add_tag(&mut task, "QUARTER".to_string());
                    }
                    if reference.year() == now.year() {
                        add_tag(&mut task, "YEAR".to_string());
                    }
                    match get_date_state(&d, self.config.due) {
                        DateState::EarlierToday | DateState::LaterToday => {
                            add_tag(&mut task, "DUE".to_string());
                            add_tag(&mut task, "TODAY".to_string());
                            add_tag(&mut task, "DUETODAY".to_string());
                        }
                        DateState::AfterToday => {
                            add_tag(&mut task, "DUE".to_string());
                            if reference.date() == (now + chrono::Duration::days(1)).date() {
                                add_tag(&mut task, "TOMORROW".to_string());
                            }
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

    pub fn toggle_mark(&mut self) {
        let selected = self.current_selection;
        let task_id = self.tasks[selected].id().unwrap_or_default();
        let task_uuid = *self.tasks[selected].uuid();

        if !self.marked.insert(task_uuid) {
            self.marked.remove(&task_uuid);
        }
    }

    pub fn toggle_mark_all(&mut self) {
        for task in &self.tasks {
            if !self.marked.insert(*task.uuid()) {
                self.marked.remove(task.uuid());
            }
        }
    }

    pub async fn handle_input(
        &mut self,
        input: Key,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        events: &Events,
    ) -> Result<()> {
        match self.mode {
            AppMode::TaskReport => {
                if input == Key::Esc {
                    self.marked.clear();
                } else if input == self.keyconfig.quit || input == Key::Ctrl('c') {
                    self.should_quit = true;
                } else if input == self.keyconfig.select {
                    self.task_table_state.multiple_selection();
                    self.toggle_mark();
                } else if input == self.keyconfig.select_all {
                    self.task_table_state.multiple_selection();
                    self.toggle_mark_all();
                } else if input == self.keyconfig.refresh {
                    self.update(true).await?;
                } else if input == self.keyconfig.go_to_bottom || input == Key::End {
                    self.task_report_bottom();
                } else if input == self.keyconfig.go_to_top || input == Key::Home {
                    self.task_report_top();
                } else if input == Key::Down || input == self.keyconfig.down {
                    self.task_report_next();
                } else if input == Key::Up || input == self.keyconfig.up {
                    self.task_report_previous();
                } else if input == Key::PageDown || input == self.keyconfig.page_down {
                    self.task_report_next_page();
                } else if input == Key::PageUp || input == self.keyconfig.page_up {
                    self.task_report_previous_page();
                } else if input == Key::Ctrl('e') {
                    self.task_details_scroll_up();
                } else if input == Key::Ctrl('y') {
                    self.task_details_scroll_down();
                } else if input == self.keyconfig.done {
                    match self.task_done() {
                        Ok(_) => self.update(true).await?,
                        Err(e) => {
                            self.mode = AppMode::TaskError;
                            self.error = e;
                        }
                    }
                } else if input == self.keyconfig.delete {
                    match self.task_delete() {
                        Ok(_) => self.update(true).await?,
                        Err(e) => {
                            self.mode = AppMode::TaskError;
                            self.error = e;
                        }
                    }
                } else if input == self.keyconfig.start_stop {
                    match self.task_start_stop() {
                        Ok(_) => self.update(true).await?,
                        Err(e) => {
                            self.mode = AppMode::TaskError;
                            self.error = e;
                        }
                    }
                } else if input == self.keyconfig.undo {
                    match self.task_undo() {
                        Ok(_) => self.update(true).await?,
                        Err(e) => {
                            self.mode = AppMode::TaskError;
                            self.error = e;
                        }
                    }
                } else if input == self.keyconfig.edit {
                    events.pause_key_capture(terminal);
                    let r = self.task_edit();
                    events.resume_key_capture(terminal);
                    match r {
                        Ok(_) => self.update(true).await?,
                        Err(e) => {
                            self.mode = AppMode::TaskError;
                            self.error = e;
                        }
                    }
                } else if input == self.keyconfig.modify {
                    self.mode = AppMode::TaskModify;
                    match self.task_table_state.mode() {
                        TableMode::SingleSelection => match self.task_current() {
                            Some(t) => {
                                let s = format!("{} ", t.description());
                                self.modify.update(&s, s.as_str().len())
                            }
                            None => self.modify.update("", 0),
                        },
                        TableMode::MultipleSelection => self.modify.update("", 0),
                    }
                } else if input == self.keyconfig.shell {
                    self.mode = AppMode::TaskSubprocess;
                } else if input == self.keyconfig.log {
                    self.mode = AppMode::TaskLog;
                } else if input == self.keyconfig.add {
                    self.mode = AppMode::TaskAdd;
                } else if input == self.keyconfig.annotate {
                    self.mode = AppMode::TaskAnnotate;
                } else if input == self.keyconfig.help {
                    self.mode = AppMode::TaskHelpPopup;
                } else if input == self.keyconfig.filter {
                    self.mode = AppMode::TaskFilter;
                } else if input == self.keyconfig.shortcut1 {
                    match self.task_shortcut(1) {
                        Ok(_) => self.update(true).await?,
                        Err(e) => {
                            self.mode = AppMode::TaskError;
                            self.error = e;
                        }
                    }
                } else if input == self.keyconfig.shortcut2 {
                    match self.task_shortcut(2) {
                        Ok(_) => self.update(true).await?,
                        Err(e) => {
                            self.mode = AppMode::TaskError;
                            self.error = e;
                        }
                    }
                } else if input == self.keyconfig.shortcut3 {
                    match self.task_shortcut(3) {
                        Ok(_) => self.update(true).await?,
                        Err(e) => {
                            self.mode = AppMode::TaskError;
                            self.error = e;
                        }
                    }
                } else if input == self.keyconfig.shortcut4 {
                    match self.task_shortcut(4) {
                        Ok(_) => self.update(true).await?,
                        Err(e) => {
                            self.mode = AppMode::TaskError;
                            self.error = e;
                        }
                    }
                } else if input == self.keyconfig.shortcut5 {
                    match self.task_shortcut(5) {
                        Ok(_) => self.update(true).await?,
                        Err(e) => {
                            self.mode = AppMode::TaskError;
                            self.error = e;
                        }
                    }
                } else if input == self.keyconfig.shortcut6 {
                    match self.task_shortcut(6) {
                        Ok(_) => self.update(true).await?,
                        Err(e) => {
                            self.mode = AppMode::TaskError;
                            self.error = e;
                        }
                    }
                } else if input == self.keyconfig.shortcut7 {
                    match self.task_shortcut(7) {
                        Ok(_) => self.update(true).await?,
                        Err(e) => {
                            self.mode = AppMode::TaskError;
                            self.error = e;
                        }
                    }
                } else if input == self.keyconfig.shortcut8 {
                    match self.task_shortcut(8) {
                        Ok(_) => self.update(true).await?,
                        Err(e) => {
                            self.mode = AppMode::TaskError;
                            self.error = e;
                        }
                    }
                } else if input == self.keyconfig.shortcut9 {
                    match self.task_shortcut(9) {
                        Ok(_) => self.update(true).await?,
                        Err(e) => {
                            self.mode = AppMode::TaskError;
                            self.error = e;
                        }
                    }
                } else if input == self.keyconfig.zoom {
                    self.task_report_show_info = !self.task_report_show_info;
                } else if input == self.keyconfig.context_menu {
                    self.mode = AppMode::TaskContextMenu;
                } else if input == self.keyconfig.next_tab {
                    self.mode = AppMode::Calendar;
                }
            }
            AppMode::TaskContextMenu => {
                if input == self.keyconfig.quit || input == Key::Esc {
                    self.mode = AppMode::TaskReport;
                } else if input == Key::Down || input == self.keyconfig.down {
                    self.context_next();
                } else if input == Key::Up || input == self.keyconfig.up {
                    self.context_previous();
                } else if input == Key::Char('\n') {
                    match self.context_select() {
                        Ok(_) => {
                            self.get_context()?;
                            self.update(true).await?;
                        }
                        Err(e) => {
                            self.mode = AppMode::TaskError;
                            self.error = e;
                        }
                    }
                }
            }
            AppMode::TaskHelpPopup => {
                if input == self.keyconfig.quit || input == Key::Esc {
                    self.mode = AppMode::TaskReport;
                } else if input == self.keyconfig.down {
                    self.help_popup.scroll = self.help_popup.scroll.checked_add(1).unwrap_or(0);
                    let th = (self.help_popup.text_height as u16).saturating_sub(1);
                    if self.help_popup.scroll > th {
                        self.help_popup.scroll = th
                    }
                } else if input == self.keyconfig.up {
                    self.help_popup.scroll = self.help_popup.scroll.saturating_sub(1);
                }
            }
            AppMode::TaskModify => match input {
                Key::Char('\n') => match self.task_modify() {
                    Ok(_) => {
                        self.mode = AppMode::TaskReport;
                        self.update(true).await?;
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
                        self.update(true).await?;
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
                        self.update(true).await?;
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
                        self.update(true).await?;
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
                        self.update(true).await?;
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
                    self.update(true).await?;
                }
                _ => {
                    handle_movement(&mut self.filter, input);
                    self.update(true).await?;
                }
            },
            AppMode::TaskError => self.mode = AppMode::TaskReport,
            AppMode::Calendar => {
                if input == self.keyconfig.quit || input == Key::Ctrl('c') {
                    self.should_quit = true;
                } else if input == self.keyconfig.previous_tab {
                    self.mode = AppMode::TaskReport;
                } else if input == Key::Up || input == self.keyconfig.up {
                    if self.calendar_year > 0 {
                        self.calendar_year -= 1;
                    }
                } else if input == Key::Down || input == self.keyconfig.down {
                    self.calendar_year += 1;
                } else if input == Key::PageUp || input == self.keyconfig.page_up {
                    if self.calendar_year > 0 {
                        self.calendar_year -= 10
                    }
                } else if input == Key::PageDown || input == self.keyconfig.page_down {
                    self.calendar_year += 10
                }
            }
        }
        self.update_task_table_state();
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

pub fn remove_tag(task: &mut Task, tag: String) {
    if let Some(t) = task.tags_mut() {
        if let Some(index) = t.iter().position(|x| *x == tag) {
            t.remove(index);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;
    use std::fs::File;
    use std::path::Path;
    use tui::backend::TestBackend;
    use tui::buffer::Buffer;

    #[test]
    fn test_centered_rect() {
        assert_eq!(
            centered_rect(50, 50, Rect::new(0, 0, 100, 100)),
            Rect::new(25, 25, 50, 50)
        );
    }

    fn setup() {
        use std::io::Read;
        use std::io::Write;
        use std::process::Stdio;
        let mut f = File::open(Path::new(env!("TASKDATA")).parent().unwrap().join("export.json")).unwrap();
        let mut s = String::new();
        f.read_to_string(&mut s).unwrap();
        let tasks = task_hookrs::import::import(s.as_bytes()).unwrap();
        // tasks.iter_mut().find(| t | t.id().unwrap() == 1).unwrap().priority_mut().replace(&mut "H".to_string());
        // tasks.iter_mut().find(| t | t.id().unwrap() == 2).unwrap().priority_mut().replace(&mut "H".to_string());
        // tasks.iter_mut().find(| t | t.id().unwrap() == 4).unwrap().tags_mut().replace(&mut vec!["test".to_string(), "another tag".to_string()]);
        assert!(task_hookrs::tw::save(&tasks).is_ok());
    }

    fn teardown() {
        let cd = Path::new(env!("TASKDATA"));
        std::fs::remove_dir_all(cd).unwrap();
    }

    #[test]
    fn test_taskwarrior_tui() {
        let app = TaskwarriorTuiApp::new().unwrap();
        assert!(app.task_by_index(0).is_none());

        let app = TaskwarriorTuiApp::new().unwrap();
        assert!(app
            .task_by_uuid(Uuid::parse_str("3f43831b-88dc-45e2-bf0d-4aea6db634cc").unwrap())
            .is_none());

        test_draw_empty_task_report();

        setup();

        let app = TaskwarriorTuiApp::new().unwrap();
        assert!(app.task_by_index(0).is_some());

        let app = TaskwarriorTuiApp::new().unwrap();
        assert!(app
            .task_by_uuid(Uuid::parse_str("3f43831b-88dc-45e2-bf0d-4aea6db634cc").unwrap())
            .is_some());

        test_draw_task_report();
        test_task_tags();
        test_task_style();
        test_task_context();
        test_task_tomorrow();
        test_task_earlier_today();
        test_task_later_today();
        teardown();
    }

    fn test_task_tags() {
        // testing tags
        let app = TaskwarriorTuiApp::new().unwrap();
        let task = app.task_by_id(1).unwrap();

        let tags = vec!["PENDING".to_string(), "PRIORITY".to_string()];

        for tag in tags {
            assert!(task.tags().unwrap().contains(&tag));
        }

        let app = TaskwarriorTuiApp::new().unwrap();
        let task = app.task_by_id(11).unwrap();
        let tags = vec!["finance", "UNBLOCKED", "PENDING", "TAGGED", "UDA"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        for tag in tags {
            assert!(task.tags().unwrap().contains(&tag));
        }
    }

    fn test_task_style() {
        let app = TaskwarriorTuiApp::new().unwrap();
        let task = app.task_by_id(1).unwrap();
        for r in vec![
            "active",
            "blocked",
            "blocking",
            "completed",
            "deleted",
            "due",
            "due.today",
            "keyword.",
            "overdue",
            "project.",
            "recurring",
            "scheduled",
            "tag.",
            "tagged",
            "uda.",
        ] {
            assert!(app.config.rule_precedence_color.contains(&r.to_string()));
        }
        let style = app.style_for_task(&task);

        assert_eq!(style, Style::default().fg(Color::Indexed(2)));

        let task = app.task_by_id(11).unwrap();
        let style = app.style_for_task(&task);
    }

    fn test_task_context() {
        let mut app = TaskwarriorTuiApp::new().unwrap();

        assert!(app.get_context().is_ok());
        assert!(app.update(true).is_ok());

        app.context_select().unwrap();

        assert_eq!(app.tasks.len(), 26);
        assert_eq!(app.current_context_filter, "");

        assert_eq!(app.context_table_state.current_selection(), Some(0));
        app.context_next();
        app.context_select().unwrap();
        assert_eq!(app.context_table_state.current_selection(), Some(1));

        assert!(app.get_context().is_ok());
        assert!(app.update(true).is_ok());

        assert_eq!(app.tasks.len(), 1);
        assert_eq!(app.current_context_filter, "+finance -private");

        assert_eq!(app.context_table_state.current_selection(), Some(1));
        app.context_previous();
        app.context_select().unwrap();
        assert_eq!(app.context_table_state.current_selection(), Some(0));

        assert!(app.get_context().is_ok());
        assert!(app.update(true).is_ok());

        assert_eq!(app.tasks.len(), 26);
        assert_eq!(app.current_context_filter, "");
    }

    fn test_task_tomorrow() {
        let total_tasks: u64 = 26;

        let mut app = TaskwarriorTuiApp::new().unwrap();
        assert!(app.get_context().is_ok());
        assert!(app.update(true).is_ok());
        assert_eq!(app.tasks.len(), total_tasks as usize);
        assert_eq!(app.current_context_filter, "");

        let now = Local::now();
        let now = TimeZone::from_utc_datetime(now.offset(), &now.naive_utc());

        let mut command = Command::new("task");
        command.arg("add");
        let tomorrow = now + chrono::Duration::days(1);
        let message = format!(
            "'new task for testing tomorrow' due:{:04}-{:02}-{:02}",
            tomorrow.year(),
            tomorrow.month(),
            tomorrow.day(),
        );

        let shell = message.as_str().replace("'", "\\'");
        let cmd = shlex::split(&shell).unwrap();
        for s in cmd {
            command.arg(&s);
        }
        let output = command.output().unwrap();
        let s = String::from_utf8_lossy(&output.stdout);
        let re = Regex::new(r"^Created task (?P<task_id>\d+).\n$").unwrap();
        let caps = re.captures(&s);
        if caps.is_none() {
            let s = String::from_utf8_lossy(&output.stderr);
            assert!(false);
        }
        let caps = re.captures(&s).unwrap();

        let task_id = caps["task_id"].parse::<u64>().unwrap();
        assert_eq!(task_id, total_tasks + 1);

        assert!(app.get_context().is_ok());
        assert!(app.update(true).is_ok());
        assert_eq!(app.tasks.len(), (total_tasks + 1) as usize);
        assert_eq!(app.current_context_filter, "");

        let task = app.task_by_id(task_id).unwrap();

        for s in &[
            "DUE",
            "MONTH",
            "PENDING",
            "QUARTER",
            "TOMORROW",
            "UDA",
            "UNBLOCKED",
            "YEAR",
        ] {
            assert!(task.tags().unwrap().contains(&s.to_string()));
        }

        let output = Command::new("task")
            .arg("rc.confirmation=off")
            .arg("undo")
            .output()
            .unwrap();

        let mut app = TaskwarriorTuiApp::new().unwrap();
        assert!(app.get_context().is_ok());
        assert!(app.update(true).is_ok());
        assert_eq!(app.tasks.len(), total_tasks as usize);
        assert_eq!(app.current_context_filter, "");
    }

    fn test_task_earlier_today() {
        let total_tasks: u64 = 26;

        let mut app = TaskwarriorTuiApp::new().unwrap();
        assert!(app.get_context().is_ok());
        assert!(app.update(true).is_ok());
        assert_eq!(app.tasks.len(), total_tasks as usize);
        assert_eq!(app.current_context_filter, "");

        let now = Local::now();
        let now = TimeZone::from_utc_datetime(now.offset(), &now.naive_utc());

        let mut command = Command::new("task");
        command.arg("add");
        let message = "'new task for testing earlier today' due:now";

        let shell = message.replace("'", "\\'");
        let cmd = shlex::split(&shell).unwrap();
        for s in cmd {
            command.arg(&s);
        }
        let output = command.output().unwrap();
        let s = String::from_utf8_lossy(&output.stdout);
        let re = Regex::new(r"^Created task (?P<task_id>\d+).\n$").unwrap();
        let caps = re.captures(&s).unwrap();
        let task_id = caps["task_id"].parse::<u64>().unwrap();
        assert_eq!(task_id, total_tasks + 1);

        assert!(app.get_context().is_ok());
        assert!(app.update(true).is_ok());
        assert_eq!(app.tasks.len(), (total_tasks + 1) as usize);
        assert_eq!(app.current_context_filter, "");

        let task = app.task_by_id(task_id).unwrap();
        for s in &[
            "DUE",
            "DUETODAY",
            "MONTH",
            "OVERDUE",
            "PENDING",
            "QUARTER",
            "TODAY",
            "UDA",
            "UNBLOCKED",
            "YEAR",
        ] {
            assert!(task.tags().unwrap().contains(&s.to_string()));
        }

        let output = Command::new("task")
            .arg("rc.confirmation=off")
            .arg("undo")
            .output()
            .unwrap();

        let mut app = TaskwarriorTuiApp::new().unwrap();
        assert!(app.get_context().is_ok());
        assert!(app.update(true).is_ok());
        assert_eq!(app.tasks.len(), total_tasks as usize);
        assert_eq!(app.current_context_filter, "");
    }

    fn test_task_later_today() {
        use chrono::Timelike;

        let total_tasks: u64 = 26;

        let mut app = TaskwarriorTuiApp::new().unwrap();
        assert!(app.get_context().is_ok());
        assert!(app.update(true).is_ok());
        assert_eq!(app.tasks.len(), total_tasks as usize);
        assert_eq!(app.current_context_filter, "");

        let now = Local::now();
        let now = TimeZone::from_utc_datetime(now.offset(), &now.naive_utc());

        let mut command = Command::new("task");
        command.arg("add");
        let message = format!(
            "'new task for testing later today' due:'{:04}-{:02}-{:02}T{:02}:{:02}:{:02}'",
            now.year(),
            now.month(),
            now.day(),
            now.hour(),
            now.minute() + 1,
            now.second(),
        );

        let shell = message.as_str().replace("'", "\\'");
        let cmd = shlex::split(&shell).unwrap();
        for s in cmd {
            command.arg(&s);
        }
        let output = command.output().unwrap();
        let s = String::from_utf8_lossy(&output.stdout);
        let re = Regex::new(r"^Created task (?P<task_id>\d+).\n$").unwrap();
        let caps = re.captures(&s).unwrap();
        let task_id = caps["task_id"].parse::<u64>().unwrap();
        assert_eq!(task_id, total_tasks + 1);

        assert!(app.get_context().is_ok());
        assert!(app.update(true).is_ok());
        assert_eq!(app.tasks.len(), (total_tasks + 1) as usize);
        assert_eq!(app.current_context_filter, "");

        let task = app.task_by_id(task_id).unwrap();
        for s in &[
            "DUE",
            "DUETODAY",
            "MONTH",
            "PENDING",
            "QUARTER",
            "TODAY",
            "UDA",
            "UNBLOCKED",
            "YEAR",
        ] {
            assert!(task.tags().unwrap().contains(&s.to_string()));
        }

        let output = Command::new("task")
            .arg("rc.confirmation=off")
            .arg("undo")
            .output()
            .unwrap();

        let mut app = TaskwarriorTuiApp::new().unwrap();
        assert!(app.get_context().is_ok());
        assert!(app.update(true).is_ok());
        assert_eq!(app.tasks.len(), total_tasks as usize);
        assert_eq!(app.current_context_filter, "");
    }

    fn test_draw_empty_task_report() {
        let test_case = |expected: &Buffer| {
            let mut app = TaskwarriorTuiApp::new().unwrap();

            app.task_report_next();
            app.context_next();

            let total_tasks: u64 = 0;

            assert!(app.get_context().is_ok());
            assert!(app.update(true).is_ok());
            assert_eq!(app.tasks.len(), total_tasks as usize);
            assert_eq!(app.current_context_filter, "");

            let now = Local::now();
            let now = TimeZone::from_utc_datetime(now.offset(), &now.naive_utc());

            app.update(true).unwrap();

            let backend = TestBackend::new(50, 15);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal
                .draw(|f| {
                    app.draw(f);
                    app.draw(f);
                })
                .unwrap();

            assert_eq!(terminal.backend().size().unwrap(), expected.area);
            terminal.backend().assert_buffer(expected);
        };

        let mut expected = Buffer::with_lines(vec![
            "╭Task|Calendar───────────────────────────────────╮",
            "│                                                │",
            "│                                                │",
            "│                                                │",
            "│                                                │",
            "╰────────────────────────────────────────────────╯",
            "╭Task not found──────────────────────────────────╮",
            "│                                                │",
            "│                                                │",
            "│                                                │",
            "│                                                │",
            "╰────────────────────────────────────────────────╯",
            "╭Filter Tasks────────────────────────────────────╮",
            "│status:pending -private                         │",
            "╰────────────────────────────────────────────────╯",
        ]);

        for i in 1..=4 {
            // Task
            expected
                .get_mut(i, 0)
                .set_style(Style::default().add_modifier(Modifier::BOLD));
        }
        for i in 6..=13 {
            // Calendar
            expected
                .get_mut(i, 0)
                .set_style(Style::default().add_modifier(Modifier::DIM));
        }

        test_case(&expected);
    }

    fn test_draw_task_report() {
        let test_case = |expected: &Buffer| {
            let mut app = TaskwarriorTuiApp::new().unwrap();

            app.task_report_next();
            app.context_next();

            let total_tasks: u64 = 26;

            assert!(app.get_context().is_ok());
            assert!(app.update(true).is_ok());
            assert_eq!(app.tasks.len(), total_tasks as usize);
            assert_eq!(app.current_context_filter, "");

            let now = Local::now();
            let now = TimeZone::from_utc_datetime(now.offset(), &now.naive_utc());

            let mut command = Command::new("task");
            command.arg("add");
            let message = "'new task 1 for testing draw' priority:U";

            let shell = message.replace("'", "\\'");
            let cmd = shlex::split(&shell).unwrap();
            for s in cmd {
                command.arg(&s);
            }
            let output = command.output().unwrap();
            let s = String::from_utf8_lossy(&output.stdout);
            let re = Regex::new(r"^Created task (?P<task_id>\d+).\n$").unwrap();
            let caps = re.captures(&s).unwrap();
            let task_id = caps["task_id"].parse::<u64>().unwrap();
            assert_eq!(task_id, total_tasks + 1);

            let mut command = Command::new("task");
            command.arg("add");
            let message = "'new task 2 for testing draw' priority:U +none";

            let shell = message.replace("'", "\\'");
            let cmd = shlex::split(&shell).unwrap();
            for s in cmd {
                command.arg(&s);
            }
            let output = command.output().unwrap();
            let s = String::from_utf8_lossy(&output.stdout);
            let re = Regex::new(r"^Created task (?P<task_id>\d+).\n$").unwrap();
            let caps = re.captures(&s).unwrap();
            let task_id = caps["task_id"].parse::<u64>().unwrap();
            assert_eq!(task_id, total_tasks + 2);

            app.task_report_next();
            app.task_report_previous();
            app.task_report_next_page();
            app.task_report_previous_page();
            app.task_report_bottom();
            app.task_report_top();
            app.update(true).unwrap();

            let backend = TestBackend::new(50, 15);
            let mut terminal = Terminal::new(backend).unwrap();
            app.task_report_show_info = !app.task_report_show_info;
            terminal
                .draw(|f| {
                    app.draw(f);
                    app.draw(f);
                })
                .unwrap();
            app.task_report_show_info = !app.task_report_show_info;
            terminal
                .draw(|f| {
                    app.draw(f);
                    app.draw(f);
                })
                .unwrap();

            let output = Command::new("task")
                .arg("rc.confirmation=off")
                .arg("undo")
                .output()
                .unwrap();
            let output = Command::new("task")
                .arg("rc.confirmation=off")
                .arg("undo")
                .output()
                .unwrap();

            assert_eq!(terminal.backend().size().unwrap(), expected.area);
            terminal.backend().assert_buffer(expected);
        };

        let mut expected = Buffer::with_lines(vec![
            "╭Task|Calendar───────────────────────────────────╮",
            "│  ID Age Deps P Projec Tag     Due Descrip Urg  │",
            "│                                                │",
            "│• 27 0s       U                    new ta… 15.00│",
            "│  28 0s       U        none        new ta… 15.00│",
            "╰────────────────────────────────────────────────╯",
            "╭Task 27─────────────────────────────────────────╮",
            "│                                                │",
            "│Name        Value                               │",
            "│----------- ------------------------------------│",
            "│ID          27                                  │",
            "╰────────────────────────────────────────────────╯",
            "╭Filter Tasks────────────────────────────────────╮",
            "│status:pending -private                         │",
            "╰────────────────────────────────────────────────╯",
        ]);

        for i in 1..=4 {
            // Task
            expected
                .get_mut(i, 0)
                .set_style(Style::default().add_modifier(Modifier::BOLD));
        }
        for i in 6..=13 {
            // Calendar
            expected
                .get_mut(i, 0)
                .set_style(Style::default().add_modifier(Modifier::DIM));
        }

        for r in &[
            1..=4,   // ID
            6..=8,   // Age
            10..=13, // Deps
            15..=15, // P
            17..=22, // Projec
            24..=30, // Tag
            32..=34, // Due
            36..=42, // Descr
            44..=48, // Urg
        ] {
            for i in r.clone().into_iter() {
                expected
                    .get_mut(i, 1)
                    .set_style(Style::default().add_modifier(Modifier::UNDERLINED));
            }
        }

        for i in 1..expected.area().width - 1 {
            expected.get_mut(i, 3).set_style(
                Style::default()
                    .fg(Color::Indexed(1))
                    .bg(Color::Reset)
                    .add_modifier(Modifier::BOLD),
            );
        }

        for i in 1..expected.area().width - 1 {
            expected
                .get_mut(i, 4)
                .set_style(Style::default().fg(Color::Indexed(1)).bg(Color::Indexed(4)));
        }

        test_case(&expected);
    }

    #[test]
    fn test_draw_calendar() {
        let test_case = |expected: &Buffer| {
            let mut app = TaskwarriorTuiApp::new().unwrap();

            app.task_report_next();
            app.context_next();
            app.update(true).unwrap();

            app.calendar_year = 2020;
            app.mode = AppMode::Calendar;

            let backend = TestBackend::new(50, 15);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal
                .draw(|f| {
                    app.draw(f);
                    app.draw(f);
                })
                .unwrap();

            assert_eq!(terminal.backend().size().unwrap(), expected.area);
            terminal.backend().assert_buffer(expected);
        };

        let mut expected = Buffer::with_lines(vec![
            "╭Task|Calendar───────────────────────────────────╮",
            "│                                                │",
            "│                      2020                      │",
            "│                                                │",
            "│        January               February          │",
            "│  Su Mo Tu We Th Fr Sa  Su Mo Tu We Th Fr Sa    │",
            "│            1  2  3  4                     1    │",
            "│   5  6  7  8  9 10 11   2  3  4  5  6  7  8    │",
            "│  12 13 14 15 16 17 18   9 10 11 12 13 14 15    │",
            "│  19 20 21 22 23 24 25  16 17 18 19 20 21 22    │",
            "│  26 27 28 29 30 31     23 24 25 26 27 28 29    │",
            "│                                                │",
            "│                                                │",
            "│                                                │",
            "╰────────────────────────────────────────────────╯",
        ]);

        for i in 1..=4 {
            // Task
            expected
                .get_mut(i, 0)
                .set_style(Style::default().add_modifier(Modifier::DIM));
        }
        for i in 6..=13 {
            // Calendar
            expected
                .get_mut(i, 0)
                .set_style(Style::default().add_modifier(Modifier::BOLD));
        }

        for i in 1..=48 {
            expected
                .get_mut(i, 2)
                .set_style(Style::default().add_modifier(Modifier::UNDERLINED));
        }

        for i in 3..=22 {
            expected.get_mut(i, 4).set_style(Style::default().bg(Color::Black));
        }

        for i in 25..=44 {
            expected.get_mut(i, 4).set_style(Style::default().bg(Color::Black));
        }

        for i in 3..=22 {
            expected
                .get_mut(i, 5)
                .set_style(Style::default().bg(Color::Black).add_modifier(Modifier::UNDERLINED));
        }

        for i in 25..=44 {
            expected
                .get_mut(i, 5)
                .set_style(Style::default().bg(Color::Black).add_modifier(Modifier::UNDERLINED));
        }

        test_case(&expected);
    }

    #[test]
    fn test_draw_help_popup() {
        let test_case = |expected: &Buffer| {
            let mut app = TaskwarriorTuiApp::new().unwrap();

            app.mode = AppMode::TaskHelpPopup;
            app.task_report_next();
            app.context_next();
            app.update(true).unwrap();

            let backend = TestBackend::new(40, 12);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal
                .draw(|f| {
                    app.draw_help_popup(f, 100, 100);
                })
                .unwrap();

            assert_eq!(terminal.backend().size().unwrap(), expected.area);
            terminal.backend().assert_buffer(expected);
        };

        let mut expected = Buffer::with_lines(vec![
            "╭Help──────────────────────────────────╮",
            "│# Default Keybindings                 │",
            "│                                      │",
            "│Keybindings:                          │",
            "│                                      │",
            "│    Esc:                              │",
            "│                                      │",
            "│    ]: Next view                      │",
            "│                                      │",
            "│    [: Previous view                  │",
            "│                                      │",
            "╰──────────────────────────────────────╯",
        ]);

        for i in 1..=4 {
            // Calendar
            expected
                .get_mut(i, 0)
                .set_style(Style::default().add_modifier(Modifier::BOLD));
        }

        test_case(&expected);
    }

    fn test_draw_context_menu() {
        let test_case = |expected: &Buffer| {
            let mut app = TaskwarriorTuiApp::new().unwrap();

            app.mode = AppMode::TaskContextMenu;
            app.task_report_next();
            app.context_next();
            app.update(true).unwrap();

            let backend = TestBackend::new(80, 10);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal
                .draw(|f| {
                    app.draw_context_menu(f, 100, 100);
                    app.draw_context_menu(f, 100, 100);
                })
                .unwrap();

            assert_eq!(terminal.backend().size().unwrap(), expected.area);
            terminal.backend().assert_buffer(expected);
        };

        let mut expected = Buffer::with_lines(vec![
            "╭Context───────────────────────────────────────────────────────────────────────╮",
            "│Name       Description                                                  Active│",
            "│                                                                              │",
            "│• none                                                                  yes   │",
            "│  finance  +finance -private                                            no    │",
            "│  personal +personal -private                                           no    │",
            "│  work     -personal -private                                           no    │",
            "│                                                                              │",
            "│                                                                              │",
            "╰──────────────────────────────────────────────────────────────────────────────╯",
        ]);

        for i in 1..=7 {
            // Task
            expected
                .get_mut(i, 0)
                .set_style(Style::default().add_modifier(Modifier::BOLD));
        }

        for i in 1..=10 {
            // Task
            expected
                .get_mut(i, 1)
                .set_style(Style::default().add_modifier(Modifier::UNDERLINED));
        }

        for i in 12..=71 {
            // Task
            expected
                .get_mut(i, 1)
                .set_style(Style::default().add_modifier(Modifier::UNDERLINED));
        }

        for i in 73..=78 {
            // Task
            expected
                .get_mut(i, 1)
                .set_style(Style::default().add_modifier(Modifier::UNDERLINED));
        }

        for i in 1..=78 {
            // Task
            expected
                .get_mut(i, 3)
                .set_style(Style::default().add_modifier(Modifier::BOLD));
        }

        test_case(&expected);
    }
}
