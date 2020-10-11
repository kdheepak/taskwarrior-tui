use std::collections::HashMap;
use std::process::Command;
use std::str;
use tui::style::Color;

#[derive(Debug, Clone, Copy)]
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
pub struct TConfig {
    pub enabled: bool,
    pub color_active: TColor,
    pub color_alternate: TColor,
    pub color_blocked: TColor,
    pub color_blocking: TColor,
    pub color_burndown_done: TColor,
    pub color_burndown_pending: TColor,
    pub color_burndown_started: TColor,
    pub color_calendar_due: TColor,
    pub color_calendar_due_today: TColor,
    pub color_calendar_holiday: TColor,
    pub color_calendar_overdue: TColor,
    pub color_calendar_today: TColor,
    pub color_calendar_weekend: TColor,
    pub color_calendar_weeknumber: TColor,
    pub color_completed: TColor,
    pub color_debug: TColor,
    pub color_deleted: TColor,
    pub color_due: TColor,
    pub color_due_today: TColor,
    pub color_error: TColor,
    pub color_footnote: TColor,
    pub color_header: TColor,
    pub color_history_add: TColor,
    pub color_history_delete: TColor,
    pub color_history_done: TColor,
    pub color_label: TColor,
    pub color_label_sort: TColor,
    pub color_overdue: TColor,
    pub color_project: TColor,
    pub color_recurring: TColor,
    pub color_scheduled: TColor,
    pub color_summary_background: TColor,
    pub color_summary_bar: TColor,
    pub color_sync_added: TColor,
    pub color_sync_changed: TColor,
    pub color_sync_rejected: TColor,
    pub color_tag_next: TColor,
    pub color_tag: TColor,
    pub color_tagged: TColor,
    pub color_uda_priority: TColor,
    pub color_uda_priority_h: TColor,
    pub color_uda_priority_l: TColor,
    pub color_uda_priority_m: TColor,
    pub color_undo_after: TColor,
    pub color_undo_before: TColor,
    pub color_until: TColor,
    pub color_warning: TColor,
    pub obfuscate: bool,
    pub print_empty_columns: bool,
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

impl TConfig {
    pub fn default() -> Self {
        let output = Command::new("task")
            .arg("rc.color=off")
            .arg("show")
            .output()
            .expect("Unable to run `task show`");

        let data = String::from_utf8(output.stdout).expect("Unable to convert stdout to string");

        let enabled = true;

        let attributes = vec![
            "active",
            "color.alternate",
            "color.blocked",
            "color.blocking",
            "color.burndown.done",
            "color.burndown.pending",
            "color.burndown.started",
            "color.calendar.due",
            "color.calendar.due.today",
            "color.calendar.holiday",
            "color.calendar.overdue",
            "color.calendar.today",
            "color.calendar.weekend",
            "color.calendar.weeknumber",
            "color.completed",
            "color.debug",
            "color.deleted",
            "color.due",
            "color.due.today",
            "color.error",
            "color.footnote",
            "color.header",
            "color.header.add",
            "color.history.delete",
            "color.history.done",
            "color.label",
            "color.label.sort",
            "color.overdue",
            "color.project.none",
            "color.recurring",
            "color.scheduled",
            "color.summary.background",
            "color.summary.bar",
            "color.sync.added",
            "color.sync.changed",
            "color.sync.rejected",
            "color.tag.next",
            "color.tag.none",
            "color.tagged",
            "color.uda.priority",
            "color.uda.priority.H",
            "color.uda.priority.L",
            "color.uda.priority.M",
            "color.undo.after",
            "color.undo.before",
            "color.undo.until",
            "color.until",
            "color.warning",
            // "obfuscate",
            // "print.empty.columns",
        ];

        let mut color_collection = HashMap::new();
        for line in data.split('\n') {
            for attribute in &attributes {
                if line.starts_with(attribute) {
                    color_collection.insert(
                        attribute.to_string(),
                        get_tcolor(line.trim_start_matches(attribute).trim_start_matches(" ")),
                    );
                }
            }
        }

        let mut bool_collection = HashMap::new();
        for line in data.split('\n') {
            for attribute in &attributes {
                if line.starts_with(attribute) {
                    bool_collection.insert(
                        attribute.to_string(),
                        line.trim_start_matches(attribute).trim_start_matches(" ") == "yes",
                    );
                }
            }
        }

        Self {
            enabled: true,
            obfuscate: bool_collection.get("obfuscate").cloned().unwrap_or(false),
            print_empty_columns: bool_collection.get("print_empty_columns").cloned().unwrap_or(false),
            color_active: color_collection.get("active").cloned().unwrap_or(TColor::default()),
            color_alternate: color_collection.get("alternate").cloned().unwrap_or(TColor::default()),
            color_blocked: color_collection.get("blocked").cloned().unwrap_or(TColor::default()),
            color_blocking: color_collection.get("blocking").cloned().unwrap_or(TColor::default()),
            color_burndown_done: color_collection
                .get("burndown.done")
                .cloned()
                .unwrap_or(TColor::default()),
            color_burndown_pending: color_collection
                .get("burndown.pending")
                .cloned()
                .unwrap_or(TColor::default()),
            color_burndown_started: color_collection
                .get("burndown.started")
                .cloned()
                .unwrap_or(TColor::default()),
            color_calendar_due: color_collection
                .get("calendar.due")
                .cloned()
                .unwrap_or(TColor::default()),
            color_calendar_due_today: color_collection
                .get("calendar.due.today")
                .cloned()
                .unwrap_or(TColor::default()),
            color_calendar_holiday: color_collection
                .get("calendar.holiday")
                .cloned()
                .unwrap_or(TColor::default()),
            color_calendar_overdue: color_collection
                .get("calendar.overdue")
                .cloned()
                .unwrap_or(TColor::default()),
            color_calendar_today: color_collection
                .get("calendar.today")
                .cloned()
                .unwrap_or(TColor::default()),
            color_calendar_weekend: color_collection
                .get("calendar.weekend")
                .cloned()
                .unwrap_or(TColor::default()),
            color_calendar_weeknumber: color_collection
                .get("calendar.weeknumber")
                .cloned()
                .unwrap_or(TColor::default()),
            color_completed: color_collection.get("completed").cloned().unwrap_or(TColor::default()),
            color_debug: color_collection.get("debug").cloned().unwrap_or(TColor::default()),
            color_deleted: color_collection.get("deleted").cloned().unwrap_or(TColor::default()),
            color_due: color_collection.get("due").cloned().unwrap_or(TColor::default()),
            color_due_today: color_collection.get("due.today").cloned().unwrap_or(TColor::default()),
            color_error: color_collection.get("error").cloned().unwrap_or(TColor::default()),
            color_footnote: color_collection.get("footnote").cloned().unwrap_or(TColor::default()),
            color_header: color_collection.get("header").cloned().unwrap_or(TColor::default()),
            color_history_add: color_collection
                .get("history.add")
                .cloned()
                .unwrap_or(TColor::default()),
            color_history_delete: color_collection
                .get("history.delete")
                .cloned()
                .unwrap_or(TColor::default()),
            color_history_done: color_collection
                .get("history.done")
                .cloned()
                .unwrap_or(TColor::default()),
            color_label: color_collection.get("label").cloned().unwrap_or(TColor::default()),
            color_label_sort: color_collection.get("label.sort").cloned().unwrap_or(TColor::default()),
            color_overdue: color_collection.get("overdue").cloned().unwrap_or(TColor::default()),
            color_project: color_collection.get("project").cloned().unwrap_or(TColor::default()),
            color_recurring: color_collection.get("recurring").cloned().unwrap_or(TColor::default()),
            color_scheduled: color_collection.get("scheduled").cloned().unwrap_or(TColor::default()),
            color_summary_background: color_collection
                .get("summary.background")
                .cloned()
                .unwrap_or(TColor::default()),
            color_summary_bar: color_collection
                .get("summary_bar")
                .cloned()
                .unwrap_or(TColor::default()),
            color_sync_added: color_collection.get("sync.added").cloned().unwrap_or(TColor::default()),
            color_sync_changed: color_collection
                .get("sync.changed")
                .cloned()
                .unwrap_or(TColor::default()),
            color_sync_rejected: color_collection
                .get("sync.rejected")
                .cloned()
                .unwrap_or(TColor::default()),
            color_tag_next: color_collection.get("tag.next").cloned().unwrap_or(TColor::default()),
            color_tag: color_collection.get("tag").cloned().unwrap_or(TColor::default()),
            color_tagged: color_collection.get("tagged").cloned().unwrap_or(TColor::default()),
            color_uda_priority: color_collection
                .get("uda.priority")
                .cloned()
                .unwrap_or(TColor::default()),
            color_uda_priority_h: color_collection
                .get("uda.priority.h")
                .cloned()
                .unwrap_or(TColor::default()),
            color_uda_priority_l: color_collection
                .get("uda.priority.l")
                .cloned()
                .unwrap_or(TColor::default()),
            color_uda_priority_m: color_collection
                .get("uda.priority.m")
                .cloned()
                .unwrap_or(TColor::default()),
            color_undo_after: color_collection.get("undo.after").cloned().unwrap_or(TColor::default()),
            color_undo_before: color_collection
                .get("undo.before")
                .cloned()
                .unwrap_or(TColor::default()),
            color_until: color_collection.get("until").cloned().unwrap_or(TColor::default()),
            color_warning: color_collection.get("warning").cloned().unwrap_or(TColor::default()),
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
