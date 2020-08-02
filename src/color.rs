use std::process::Command;
use tui::style::Color;

pub struct TColor {
    pub enabled: bool,
    pub active: Color,
    pub alternate: Color,
    pub blocked: Color,
    pub blocking: Color,
    pub burndown_done: Color,
    pub burndown_pending: Color,
    pub burndown_started: Color,
    pub calendar_due: Color,
    pub calendar_due_today: Color,
    pub calendar_holiday: Color,
    pub calendar_overdue: Color,
    pub calendar_today: Color,
    pub calendar_weekend: Color,
    pub calendar_weeknumber: Color,
    pub completed: Color,
    pub debug: Color,
    pub deleted: Color,
    pub due: Color,
    pub due_today: Color,
    pub error: Color,
    pub footnote: Color,
    pub header: Color,
    pub history_add: Color,
    pub history_delete: Color,
    pub history_done: Color,
    pub label: Color,
    pub label_sort: Color,
    pub overdue: Color,
    pub project: Color,
    pub recurring: Color,
    pub scheduled: Color,
    pub summary_background: Color,
    pub summary_bar: Color,
    pub sync_added: Color,
    pub sync_changed: Color,
    pub sync_rejected: Color,
    pub tag_next: Color,
    pub tag: Color,
    pub tagged: Color,
    pub uda_priority: Color,
    pub uda_priority_h: Color,
    pub uda_priority_l: Color,
    pub uda_priority_m: Color,
    pub undo_after: Color,
    pub undo_before: Color,
    pub until: Color,
    pub warning: Color,
}

pub fn get_color(line: &str) -> Color {
    let sline = line.split(" ").collect::<Vec<&str>>();
    if sline.len() == 1 {
        return Color::Rgb(0, 0, 0);
    }
    if line.contains(" on ") {
        let foreground = line.split(" ").collect::<Vec<&str>>()[1];
        let background = line.split(" ").collect::<Vec<&str>>()[3];
        if foreground.starts_with("color") {
            // TODO: get the correct color here
            Color::Rgb(0, 0, 0)
        } else if foreground.starts_with("rgb") {
            Color::Rgb(
                foreground.as_bytes()[3],
                foreground.as_bytes()[4],
                foreground.as_bytes()[5],
            )
        } else {
            Color::Rgb(0, 0, 0)
        }
    } else {
        let foreground = line.split(" ").collect::<Vec<&str>>()[1];
        if foreground.starts_with("color") {
            // TODO: get the correct color here
            Color::Rgb(0, 0, 0)
        } else if foreground.starts_with("rgb") {
            Color::Rgb(
                foreground.as_bytes()[3],
                foreground.as_bytes()[4],
                foreground.as_bytes()[5],
            )
        } else {
            Color::Rgb(0, 0, 0)
        }
    }
}

impl TColor {
    pub fn default() -> Self {
        let output = Command::new("task")
            .arg("rc.color=off")
            .arg("show")
            .output()
            .expect("Unable to run `task show`");

        let data = String::from_utf8(output.stdout).expect("Unable to convert stdout to string");

        let enabled = true;
        let mut active = Color::Black;
        let mut alternate = Color::Black;
        let mut blocked = Color::Black;
        let mut blocking = Color::Black;
        let mut burndown_done = Color::Black;
        let mut burndown_pending = Color::Black;
        let mut burndown_started = Color::Black;
        let mut calendar_due = Color::Black;
        let mut calendar_due_today = Color::Black;
        let mut calendar_holiday = Color::Black;
        let mut calendar_overdue = Color::Black;
        let mut calendar_today = Color::Black;
        let mut calendar_weekend = Color::Black;
        let mut calendar_weeknumber = Color::Black;
        let mut completed = Color::Black;
        let mut debug = Color::Black;
        let mut deleted = Color::Black;
        let mut due = Color::Black;
        let mut due_today = Color::Black;
        let mut error = Color::Black;
        let mut footnote = Color::Black;
        let mut header = Color::Black;
        let mut history_add = Color::Black;
        let mut history_delete = Color::Black;
        let mut history_done = Color::Black;
        let mut label = Color::Black;
        let mut label_sort = Color::Black;
        let mut overdue = Color::Black;
        let mut project = Color::Black;
        let mut recurring = Color::Black;
        let mut scheduled = Color::Black;
        let mut summary_background = Color::Black;
        let mut summary_bar = Color::Black;
        let mut sync_added = Color::Black;
        let mut sync_changed = Color::Black;
        let mut sync_rejected = Color::Black;
        let mut tag_next = Color::Black;
        let mut tag = Color::Black;
        let mut tagged = Color::Black;
        let mut uda_priority = Color::Black;
        let mut uda_priority_h = Color::Black;
        let mut uda_priority_l = Color::Black;
        let mut uda_priority_m = Color::Black;
        let mut undo_after = Color::Black;
        let mut undo_before = Color::Black;
        let mut until = Color::Black;
        let mut warning = Color::Black;

        for line in data.split('\n') {
            if line.starts_with("color.active") {
                active = get_color(line);
            }
            if line.starts_with("color.alternate") {
                alternate = get_color(line);
            }
            if line.starts_with("color.blocked") {
                blocked = get_color(line);
            }
            if line.starts_with("color.blocking") {
                blocking = get_color(line);
            }
            if line.starts_with("color.burndown.done") {
                burndown_done = get_color(line);
            }
            if line.starts_with("color.burndown.pending") {
                burndown_pending = get_color(line);
            }
            if line.starts_with("color.burndown.started") {
                burndown_started = get_color(line);
            }
            if line.starts_with("color.calendar.due") {
                calendar_due = get_color(line);
            }
            if line.starts_with("color.calendar.due.today") {
                calendar_due_today = get_color(line);
            }
            if line.starts_with("color.calendar.holiday") {
                calendar_holiday = get_color(line);
            }
            if line.starts_with("color.calendar.overdue") {
                calendar_overdue = get_color(line);
            }
            if line.starts_with("color.calendar.today") {
                calendar_today = get_color(line);
            }
            if line.starts_with("color.calendar.weekend") {
                calendar_weekend = get_color(line);
            }
            if line.starts_with("color.calendar.weeknumber") {
                calendar_weeknumber = get_color(line);
            }
            if line.starts_with("color.completed") {
                completed = get_color(line);
            }
            if line.starts_with("color.debug") {
                debug = get_color(line);
            }
            if line.starts_with("color.deleted") {
                deleted = get_color(line);
            }
            if line.starts_with("color.due") {
                due = get_color(line);
            }
            if line.starts_with("color.due.today") {
                due_today = get_color(line);
            }
            if line.starts_with("color.error") {
                error = get_color(line);
            }
            if line.starts_with("color.footnote") {
                footnote = get_color(line);
            }
            if line.starts_with("color.header") {
                header = get_color(line);
            }
            if line.starts_with("color.history.add") {
                history_add = get_color(line);
            }
            if line.starts_with("color.history.delete") {
                history_delete = get_color(line);
            }
            if line.starts_with("color.history.done") {
                history_done = get_color(line);
            }
            if line.starts_with("color.label") {
                label = get_color(line);
            }
            if line.starts_with("color.label.sort") {
                label_sort = get_color(line);
            }
            if line.starts_with("color.overdue") {
                overdue = get_color(line);
            }
            if line.starts_with("color.project.none") {
                project = get_color(line);
            }
            if line.starts_with("color.recurring") {
                recurring = get_color(line);
            }
            if line.starts_with("color.scheduled") {
                scheduled = get_color(line);
            }
            if line.starts_with("color.summary.background") {
                summary_background = get_color(line);
            }
            if line.starts_with("color.summary.bar") {
                summary_bar = get_color(line);
            }
            if line.starts_with("color.sync.added") {
                sync_added = get_color(line);
            }
            if line.starts_with("color.sync.changed") {
                sync_changed = get_color(line);
            }
            if line.starts_with("color.sync.rejected") {
                sync_rejected = get_color(line);
            }
            if line.starts_with("color.tag.next") {
                tag_next = get_color(line);
            }
            if line.starts_with("color.tag.none") {
                tag = get_color(line);
            }
            if line.starts_with("color.tagged") {
                tagged = get_color(line);
            }
            if line.starts_with("color.uda.priority") {
                uda_priority = get_color(line);
            }
            if line.starts_with("color.uda.priority.H") {
                uda_priority_h = get_color(line);
            }
            if line.starts_with("color.uda.priority.L") {
                uda_priority_l = get_color(line);
            }
            if line.starts_with("color.uda.priority.M") {
                uda_priority_m = get_color(line);
            }
            if line.starts_with("color.undo.after") {
                undo_after = get_color(line);
            }
            if line.starts_with("color.undo.before") {
                undo_before = get_color(line);
            }
            if line.starts_with("color.until") {
                until = get_color(line);
            }
            if line.starts_with("color.warning") {
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
            uda_priority_h: uda_priority_h,
            uda_priority_l: uda_priority_l,
            uda_priority_m: uda_priority_m,
            undo_after: undo_after,
            undo_before: undo_before,
            until: until,
            warning: warning,
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::color::TColor;
    #[test]
    fn test_colors() {
        let tc = TColor::default();
    }
}
