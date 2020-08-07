use std::process::Command;
use std::str;
use std::collections::HashMap;
use tui::style::Color;

#[derive(Debug,Clone,Copy)]
pub struct TColor {
    pub fg: Color,
    pub bg: Color,
}

impl TColor {
    pub fn default() -> Self {
        TColor {
            fg: Color::Indexed(0),
            bg: Color::Indexed(15),
        }
    }
}

#[derive(Debug)]
pub struct TColorConfig {
    pub enabled: bool,
    pub active: TColor,
    pub alternate: TColor,
    pub blocked: TColor,
    pub blocking: TColor,
    pub burndown_done: TColor,
    pub burndown_pending: TColor,
    pub burndown_started: TColor,
    pub calendar_due: TColor,
    pub calendar_due_today: TColor,
    pub calendar_holiday: TColor,
    pub calendar_overdue: TColor,
    pub calendar_today: TColor,
    pub calendar_weekend: TColor,
    pub calendar_weeknumber: TColor,
    pub completed: TColor,
    pub debug: TColor,
    pub deleted: TColor,
    pub due: TColor,
    pub due_today: TColor,
    pub error: TColor,
    pub footnote: TColor,
    pub header: TColor,
    pub history_add: TColor,
    pub history_delete: TColor,
    pub history_done: TColor,
    pub label: TColor,
    pub label_sort: TColor,
    pub overdue: TColor,
    pub project: TColor,
    pub recurring: TColor,
    pub scheduled: TColor,
    pub summary_background: TColor,
    pub summary_bar: TColor,
    pub sync_added: TColor,
    pub sync_changed: TColor,
    pub sync_rejected: TColor,
    pub tag_next: TColor,
    pub tag: TColor,
    pub tagged: TColor,
    pub uda_priority: TColor,
    pub uda_priority_h: TColor,
    pub uda_priority_l: TColor,
    pub uda_priority_m: TColor,
    pub undo_after: TColor,
    pub undo_before: TColor,
    pub until: TColor,
    pub warning: TColor,
}

pub fn get_color(s: &str) -> Color {
    if s.starts_with("color") {
        let fg = (s.as_bytes()[5] as char).to_digit(10).unwrap() as u8;
        Color::Indexed(fg)
    } else if s.starts_with("rgb") {
        let red = (s.as_bytes()[3] as char).to_digit(10).unwrap() as u8;
        let green = (s.as_bytes()[4] as char).to_digit(10).unwrap() as u8;
        let blue = (s.as_bytes()[5] as char).to_digit(10).unwrap() as u8;
        Color::Indexed(16 + red * 36 + green * 6 + blue)
    } else {
        if s == "white" {
            Color::White
        } else if s == "black" {
            Color::Black
        } else {
            Color::Indexed(15)
        }
    }
}

pub fn get_tcolor(line: &str) -> TColor {
    if line.contains(" on ") {
        let foreground = line.split(" ").collect::<Vec<&str>>()[0];
        let background = line.split(" ").collect::<Vec<&str>>()[2];
        TColor {
            fg: get_color(foreground),
            bg: get_color(background),
        }
    } else if line.contains("on ") {
        let background = line.split(" ").collect::<Vec<&str>>()[1];
        TColor {
            fg: Color::Indexed(0),
            bg: get_color(background),
        }
    } else {
        let foreground = line;
        TColor {
            fg: get_color(foreground),
            bg: Color::Indexed(15),
        }
    }
}

impl TColorConfig {
    pub fn default() -> Self {
        let output = Command::new("task")
            .arg("rc.color=off")
            .arg("show")
            .output()
            .expect("Unable to run `task show`");

        let data = String::from_utf8(output.stdout).expect("Unable to convert stdout to string");

        let enabled = true;

        let attributes = vec![
            "alternate",
            "blocked",
            "blocking",
            "burndown.done",
            "burndown.pending",
            "burndown.started",
            "calendar.due",
            "calendar.due.today",
            "calendar.holiday",
            "calendar.overdue",
            "calendar.today",
            "calendar.weekend",
            "calendar.weeknumber",
            "completed",
            "debug",
            "deleted",
            "due",
            "due.today",
            "error",
            "footnote",
            "header",
            "header.add",
            "history.delete",
            "history.done",
            "label",
            "label.sort",
            "overdue",
            "project.none",
            "recurring",
            "scheduled",
            "summary.background",
            "summary.bar",
            "sync.added",
            "sync.changed",
            "sync.rejected",
            "tag.next",
            "tag.none",
            "tagged",
            "uda.priority",
            "uda.priority.H",
            "uda.priority.L",
            "uda.priority.M",
            "undo.after",
            "undo.before",
            "undo.until",
            "until",
            "warning",
        ];

        let mut color_collection = HashMap::new();
        for line in data.split('\n') {
            for attribute in &attributes {
                let attr = format!("color.{} ", attribute);
                if line.starts_with(&attr) {
                    color_collection.insert(
                        attribute.to_string(),
                        get_tcolor(line.trim_start_matches(&attr).trim_start_matches(" "))
                    );
                }
            }
        }

        Self {
            enabled: true,
            active: match color_collection.get("active") {
                Some(c) => *c,
                None => TColor::default(),
            },
            alternate: match color_collection.get("alternate") {
                Some(c) => *c,
                None => TColor::default(),
            },
            blocked: match color_collection.get("blocked") {
                Some(c) => *c,
                None => TColor::default(),
            },
            blocking: match color_collection.get("blocking") {
                Some(c) => *c,
                None => TColor::default(),
            },
            burndown_done: match color_collection.get("burndown.done") {
                Some(c) => *c,
                None => TColor::default(),
            },
            burndown_pending: match color_collection.get("burndown.pending") {
                Some(c) => *c,
                None => TColor::default(),
            },
            burndown_started: match color_collection.get("burndown.started") {
                Some(c) => *c,
                None => TColor::default(),
            },
            calendar_due: match color_collection.get("calendar.due") {
                Some(c) => *c,
                None => TColor::default(),
            },
            calendar_due_today: match color_collection.get("calendar.due.today") {
                Some(c) => *c,
                None => TColor::default(),
            },
            calendar_holiday: match color_collection.get("calendar.holiday") {
                Some(c) => *c,
                None => TColor::default(),
            },
            calendar_overdue: match color_collection.get("calendar.overdue") {
                Some(c) => *c,
                None => TColor::default(),
            },
            calendar_today: match color_collection.get("calendar.today") {
                Some(c) => *c,
                None => TColor::default(),
            },
            calendar_weekend: match color_collection.get("calendar.weekend") {
                Some(c) => *c,
                None => TColor::default(),
            },
            calendar_weeknumber: match color_collection.get("calendar.weeknumber") {
                Some(c) => *c,
                None => TColor::default(),
            },
            completed: match color_collection.get("completed") {
                Some(c) => *c,
                None => TColor::default(),
            },
            debug: match color_collection.get("debug") {
                Some(c) => *c,
                None => TColor::default(),
            },
            deleted: match color_collection.get("deleted") {
                Some(c) => *c,
                None => TColor::default(),
            },
            due: match color_collection.get("due") {
                Some(c) => *c,
                None => TColor::default(),
            },
            due_today: match color_collection.get("due.today") {
                Some(c) => *c,
                None => TColor::default(),
            },
            error: match color_collection.get("error") {
                Some(c) => *c,
                None => TColor::default(),
            },
            footnote: match color_collection.get("footnote") {
                Some(c) => *c,
                None => TColor::default(),
            },
            header: match color_collection.get("header") {
                Some(c) => *c,
                None => TColor::default(),
            },
            history_add: match color_collection.get("history.add") {
                Some(c) => *c,
                None => TColor::default(),
            },
            history_delete: match color_collection.get("history.delete") {
                Some(c) => *c,
                None => TColor::default(),
            },
            history_done: match color_collection.get("history.done") {
                Some(c) => *c,
                None => TColor::default(),
            },
            label: match color_collection.get("label") {
                Some(c) => *c,
                None => TColor::default(),
            },
            label_sort: match color_collection.get("label.sort") {
                Some(c) => *c,
                None => TColor::default(),
            },
            overdue: match color_collection.get("overdue") {
                Some(c) => *c,
                None => TColor::default(),
            },
            project: match color_collection.get("project") {
                Some(c) => *c,
                None => TColor::default(),
            },
            recurring: match color_collection.get("recurring") {
                Some(c) => *c,
                None => TColor::default(),
            },
            scheduled: match color_collection.get("scheduled") {
                Some(c) => *c,
                None => TColor::default(),
            },
            summary_background: match color_collection.get("summary.background") {
                Some(c) => *c,
                None => TColor::default(),
            },
            summary_bar: match color_collection.get("summary_bar") {
                Some(c) => *c,
                None => TColor::default(),
            },
            sync_added: match color_collection.get("sync.added") {
                Some(c) => *c,
                None => TColor::default(),
            },
            sync_changed: match color_collection.get("sync.changed") {
                Some(c) => *c,
                None => TColor::default(),
            },
            sync_rejected: match color_collection.get("sync.rejected") {
                Some(c) => *c,
                None => TColor::default(),
            },
            tag_next: match color_collection.get("tag.next") {
                Some(c) => *c,
                None => TColor::default(),
            },
            tag: match color_collection.get("tag") {
                Some(c) => *c,
                None => TColor::default(),
            },
            tagged: match color_collection.get("tagged") {
                Some(c) => *c,
                None => TColor::default(),
            },
            uda_priority: match color_collection.get("uda.priority") {
                Some(c) => *c,
                None => TColor::default(),
            },
            uda_priority_h: match color_collection.get("uda.priority.h") {
                Some(c) => *c,
                None => TColor::default(),
            },
            uda_priority_l: match color_collection.get("uda.priority.l") {
                Some(c) => *c,
                None => TColor::default(),
            },
            uda_priority_m: match color_collection.get("uda.priority.m") {
                Some(c) => *c,
                None => TColor::default(),
            },
            undo_after: match color_collection.get("undo.after") {
                Some(c) => *c,
                None => TColor::default(),
            },
            undo_before: match color_collection.get("undo.before") {
                Some(c) => *c,
                None => TColor::default(),
            },
            until: match color_collection.get("until") {
                Some(c) => *c,
                None => TColor::default(),
            },
            warning: match color_collection.get("warning") {
                Some(c) => *c,
                None => TColor::default(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::color::TColorConfig;
    #[test]
    fn test_colors() {
        let tc = TColorConfig::default();
        println!("{:?}", tc.due_today);
    }
}
