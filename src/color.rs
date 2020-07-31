use std::process::Command;

pub struct RGB {
    r: f32,
    g: f32,
    b: f32,
}

pub struct TColor {
    pub enabled: bool,
    pub active: RGB,
    pub alternate: RGB,
    pub blocked: RGB,
    pub blocking: RGB,
    pub burndown_done: RGB,
    pub burndown_pending: RGB,
    pub burndown_started: RGB,
    pub calendar_due: RGB,
    pub calendar_due_today: RGB,
    pub calendar_holiday: RGB,
    pub calendar_overdue: RGB,
    pub calendar_today: RGB,
    pub calendar_weekend: RGB,
    pub calendar_weeknumber: RGB,
    pub completed: RGB,
    pub debug: RGB,
    pub deleted: RGB,
    pub due: RGB,
    pub due_today: RGB,
    pub error: RGB,
    pub footnote: RGB,
    pub header: RGB,
    pub history_add: RGB,
    pub history_delete: RGB,
    pub history_done: RGB,
    pub label: RGB,
    pub label_sort: RGB,
    pub overdue: RGB,
    pub project: RGB,
    pub recurring: RGB,
    pub scheduled: RGB,
    pub summary_background: RGB,
    pub summary_bar: RGB,
    pub sync_added: RGB,
    pub sync_changed: RGB,
    pub sync_rejected: RGB,
    pub tag_next: RGB,
    pub tag: RGB,
    pub tagged: RGB,
    pub uda_priority: RGB,
    pub uda_priority_H: RGB,
    pub uda_priority_L: RGB,
    pub uda_priority_M: RGB,
    pub undo_after: RGB,
    pub undo_before: RGB,
    pub until: RGB,
    pub warning: RGB,
}

pub fn get_color(line: &str) -> RGB {
    RGB {
        r: 0.0,
        g: 0.0,
        b: 0.0,
    }
}

impl TColor {
    pub fn default() -> Self {
        let output = Command::new("task")
            .arg("show")
            .output()
            .expect("Unable to run `task show`");

        let data = String::from_utf8(output.stdout).expect("Unable to convert stdout to string");

        let mut enabled = true;
        let mut active = RGB {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
        let mut alternate = RGB {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
        let mut blocked = RGB {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
        let mut blocking = RGB {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
        let mut burndown_done = RGB {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
        let mut burndown_pending = RGB {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
        let mut burndown_started = RGB {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
        let mut calendar_due = RGB {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
        let mut calendar_due_today = RGB {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
        let mut calendar_holiday = RGB {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
        let mut calendar_overdue = RGB {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
        let mut calendar_today = RGB {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
        let mut calendar_weekend = RGB {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
        let mut calendar_weeknumber = RGB {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
        let mut completed = RGB {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
        let mut debug = RGB {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
        let mut deleted = RGB {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
        let mut due = RGB {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
        let mut due_today = RGB {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
        let mut error = RGB {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
        let mut footnote = RGB {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
        let mut header = RGB {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
        let mut history_add = RGB {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
        let mut history_delete = RGB {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
        let mut history_done = RGB {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
        let mut label = RGB {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
        let mut label_sort = RGB {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
        let mut overdue = RGB {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
        let mut project = RGB {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
        let mut recurring = RGB {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
        let mut scheduled = RGB {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
        let mut summary_background = RGB {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
        let mut summary_bar = RGB {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
        let mut sync_added = RGB {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
        let mut sync_changed = RGB {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
        let mut sync_rejected = RGB {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
        let mut tag_next = RGB {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
        let mut tag = RGB {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
        let mut tagged = RGB {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
        let mut uda_priority = RGB {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
        let mut uda_priority_H = RGB {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
        let mut uda_priority_L = RGB {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
        let mut uda_priority_M = RGB {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
        let mut undo_after = RGB {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
        let mut undo_before = RGB {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
        let mut until = RGB {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
        let mut warning = RGB {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };

        for line in data.split('\n') {
            if line.starts_with("active") {
                active = get_color(line);
            }
            if line.starts_with("alternate") {
                alternate = get_color(line);
            }
            if line.starts_with("blocked") {
                blocked = get_color(line);
            }
            if line.starts_with("blocking") {
                blocking = get_color(line);
            }
            if line.starts_with("burndown.done") {
                burndown_done = get_color(line);
            }
            if line.starts_with("burndown.pending") {
                burndown_pending = get_color(line);
            }
            if line.starts_with("burndown.started") {
                burndown_started = get_color(line);
            }
            if line.starts_with("calendar.due") {
                calendar_due = get_color(line);
            }
            if line.starts_with("calendar.due.today") {
                calendar_due_today = get_color(line);
            }
            if line.starts_with("calendar.holiday") {
                calendar_holiday = get_color(line);
            }
            if line.starts_with("calendar.overdue") {
                calendar_overdue = get_color(line);
            }
            if line.starts_with("calendar.today") {
                calendar_today = get_color(line);
            }
            if line.starts_with("calendar.weekend") {
                calendar_weekend = get_color(line);
            }
            if line.starts_with("calendar.weeknumber") {
                calendar_weeknumber = get_color(line);
            }
            if line.starts_with("completed") {
                completed = get_color(line);
            }
            if line.starts_with("debug") {
                debug = get_color(line);
            }
            if line.starts_with("deleted") {
                deleted = get_color(line);
            }
            if line.starts_with("due") {
                due = get_color(line);
            }
            if line.starts_with("due.today") {
                due_today = get_color(line);
            }
            if line.starts_with("error") {
                error = get_color(line);
            }
            if line.starts_with("footnote") {
                footnote = get_color(line);
            }
            if line.starts_with("header") {
                header = get_color(line);
            }
            if line.starts_with("history.add") {
                history_add = get_color(line);
            }
            if line.starts_with("history.delete") {
                history_delete = get_color(line);
            }
            if line.starts_with("history.done") {
                history_done = get_color(line);
            }
            if line.starts_with("label") {
                label = get_color(line);
            }
            if line.starts_with("label.sort") {
                label_sort = get_color(line);
            }
            if line.starts_with("overdue") {
                overdue = get_color(line);
            }
            if line.starts_with("project.none") {
                project = get_color(line);
            }
            if line.starts_with("recurring") {
                recurring = get_color(line);
            }
            if line.starts_with("scheduled") {
                scheduled = get_color(line);
            }
            if line.starts_with("summary.background") {
                summary_background = get_color(line);
            }
            if line.starts_with("summary.bar") {
                summary_bar = get_color(line);
            }
            if line.starts_with("sync.added") {
                sync_added = get_color(line);
            }
            if line.starts_with("sync.changed") {
                sync_changed = get_color(line);
            }
            if line.starts_with("sync.rejected") {
                sync_rejected = get_color(line);
            }
            if line.starts_with("tag.next") {
                tag_next = get_color(line);
            }
            if line.starts_with("tag.none") {
                tag = get_color(line);
            }
            if line.starts_with("tagged") {
                tagged = get_color(line);
            }
            if line.starts_with("uda.priority") {
                uda_priority = get_color(line);
            }
            if line.starts_with("uda.priority.H") {
                uda_priority_H = get_color(line);
            }
            if line.starts_with("uda.priority.L") {
                uda_priority_L = get_color(line);
            }
            if line.starts_with("uda.priority.M") {
                uda_priority_M = get_color(line);
            }
            if line.starts_with("undo.after") {
                undo_after = get_color(line);
            }
            if line.starts_with("undo.before") {
                undo_before = get_color(line);
            }
            if line.starts_with("until") {
                until = get_color(line);
            }
            if line.starts_with("warning") {
                warning = get_color(line);
            }
        }

        Self {
            enabled: enabled,
            active: active,
            alternate: alternate,
            blocked: blocked,
            blocking: blocking,
            burndown_done: burndown_done,
            burndown_pending: burndown_pending,
            burndown_started: burndown_started,
            calendar_due: calendar_due,
            calendar_due_today: calendar_due_today,
            calendar_holiday: calendar_holiday,
            calendar_overdue: calendar_overdue,
            calendar_today: calendar_today,
            calendar_weekend: calendar_weekend,
            calendar_weeknumber: calendar_weeknumber,
            completed: completed,
            debug: debug,
            deleted: deleted,
            due: due,
            due_today: due_today,
            error: error,
            footnote: footnote,
            header: header,
            history_add: history_add,
            history_delete: history_delete,
            history_done: history_done,
            label: label,
            label_sort: label_sort,
            overdue: overdue,
            project: project,
            recurring: recurring,
            scheduled: scheduled,
            summary_background: summary_background,
            summary_bar: summary_bar,
            sync_added: sync_added,
            sync_changed: sync_changed,
            sync_rejected: sync_rejected,
            tag_next: tag_next,
            tag: tag,
            tagged: tagged,
            uda_priority: uda_priority,
            uda_priority_H: uda_priority_H,
            uda_priority_L: uda_priority_L,
            uda_priority_M: uda_priority_M,
            undo_after: undo_after,
            undo_before: undo_before,
            until: until,
            warning: warning,
        }
    }
}
