use std::process::Command;
use tui::style::Color;
use std::str;

#[derive(Debug)]
pub struct TColor {
    pub fg: Color,
    pub bg: Color,
}

impl TColor {
    pub fn default() -> Self {
        TColor {
            fg: Color::Indexed(0),
            bg: Color::Indexed(7),
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

pub fn get_color(line: &str) -> TColor {
    let sline = line.split(" ").collect::<Vec<&str>>();
    if sline.len() == 1 {
        return TColor::default();
    }
    if line.contains(" on ") {
        let foreground = line.split(" ").collect::<Vec<&str>>()[1];
        let background = line.split(" ").collect::<Vec<&str>>()[3];
        if foreground.starts_with("color") {
            // TODO: get the correct color here
            TColor::default()
        } else if foreground.starts_with("rgb") {
            let red = (foreground.as_bytes()[3] as char).to_digit(10).unwrap() as u8;
            let green = (foreground.as_bytes()[4] as char).to_digit(10).unwrap() as u8;
            let blue = (foreground.as_bytes()[5] as char).to_digit(10).unwrap() as u8;
            TColor{
                fg: Color::Indexed(16 + red * 36 + green * 6 + blue),
                bg: Color::Indexed(15),
            }
        } else {
            TColor::default()
        }
    } else {
        let foreground = line.split(" ").filter(|x| x.len() > 0).collect::<Vec<&str>>()[1];
        if foreground.starts_with("color") {
            // TODO: get the correct color here
            TColor::default()
        } else if foreground.starts_with("rgb") {
            let red = (foreground.as_bytes()[3] as char).to_digit(10).unwrap() as u8;
            let green = (foreground.as_bytes()[4] as char).to_digit(10).unwrap() as u8;
            let blue = (foreground.as_bytes()[5] as char).to_digit(10).unwrap() as u8;
            TColor{
                fg: Color::Indexed(16 + red * 36 + green * 6 + blue),
                bg: Color::Indexed(15),
            }
        } else {
            TColor::default()
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
        let mut active = TColor::default();
        let mut alternate = TColor::default();
        let mut blocked = TColor::default();
        let mut blocking = TColor::default();
        let mut burndown_done = TColor::default();
        let mut burndown_pending = TColor::default();
        let mut burndown_started = TColor::default();
        let mut calendar_due = TColor::default();
        let mut calendar_due_today = TColor::default();
        let mut calendar_holiday = TColor::default();
        let mut calendar_overdue = TColor::default();
        let mut calendar_today = TColor::default();
        let mut calendar_weekend = TColor::default();
        let mut calendar_weeknumber = TColor::default();
        let mut completed = TColor::default();
        let mut debug = TColor::default();
        let mut deleted = TColor::default();
        let mut due = TColor::default();
        let mut due_today = TColor::default();
        let mut error = TColor::default();
        let mut footnote = TColor::default();
        let mut header = TColor::default();
        let mut history_add = TColor::default();
        let mut history_delete = TColor::default();
        let mut history_done = TColor::default();
        let mut label = TColor::default();
        let mut label_sort = TColor::default();
        let mut overdue = TColor::default();
        let mut project = TColor::default();
        let mut recurring = TColor::default();
        let mut scheduled = TColor::default();
        let mut summary_background = TColor::default();
        let mut summary_bar = TColor::default();
        let mut sync_added = TColor::default();
        let mut sync_changed = TColor::default();
        let mut sync_rejected = TColor::default();
        let mut tag_next = TColor::default();
        let mut tag = TColor::default();
        let mut tagged = TColor::default();
        let mut uda_priority = TColor::default();
        let mut uda_priority_h = TColor::default();
        let mut uda_priority_l = TColor::default();
        let mut uda_priority_m = TColor::default();
        let mut undo_after = TColor::default();
        let mut undo_before = TColor::default();
        let mut until = TColor::default();
        let mut warning = TColor::default();

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
            enabled,
            active,
            alternate,
            blocked,
            blocking,
            burndown_done,
            burndown_pending,
            burndown_started,
            calendar_due,
            calendar_due_today,
            calendar_holiday,
            calendar_overdue,
            calendar_today,
            calendar_weekend,
            calendar_weeknumber,
            completed,
            debug,
            deleted,
            due,
            due_today,
            error,
            footnote,
            header,
            history_add,
            history_delete,
            history_done,
            label,
            label_sort,
            overdue,
            project,
            recurring,
            scheduled,
            summary_background,
            summary_bar,
            sync_added,
            sync_changed,
            sync_rejected,
            tag_next,
            tag,
            tagged,
            uda_priority,
            uda_priority_h,
            uda_priority_l,
            uda_priority_m,
            undo_after,
            undo_before,
            until,
            warning,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::color::TColorConfig;
    #[test]
    fn test_colors() {
        let tc = TColorConfig::default();
    }
}
