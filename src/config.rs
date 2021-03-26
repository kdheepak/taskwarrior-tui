use std::collections::HashMap;
use std::error::Error;
use std::process::Command;
use std::str;
use tui::style::{Color, Modifier, Style};

trait TaskWarriorBool {
    fn get_bool(&self) -> Option<bool>;
}

impl TaskWarriorBool for String {
    fn get_bool(&self) -> Option<bool> {
        if self == "true" || self == "1" || self == "y" || self == "yes" || self == "on" {
            Some(true)
        } else if self == "false" || self == "0" || self == "n" || self == "no" || self == "off" {
            Some(false)
        } else {
            None
        }
    }
}

impl TaskWarriorBool for str {
    fn get_bool(&self) -> Option<bool> {
        if self == "true" || self == "1" || self == "y" || self == "yes" || self == "on" {
            Some(true)
        } else if self == "false" || self == "0" || self == "n" || self == "no" || self == "off" {
            Some(false)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct Config {
    pub enabled: bool,
    pub color: HashMap<String, Style>,
    pub filter: String,
    pub data_location: String,
    pub obfuscate: bool,
    pub print_empty_columns: bool,
    pub due: usize,
    pub rule_precedence_color: Vec<String>,
    pub uda_task_report_show_info: bool,
    pub uda_task_report_looping: bool,
    pub uda_selection_indicator: String,
    pub uda_mark_indicator: String,
    pub uda_unmark_indicator: String,
    pub uda_selection_bold: bool,
    pub uda_selection_italic: bool,
    pub uda_selection_dim: bool,
    pub uda_selection_blink: bool,
    pub uda_calendar_months_per_row: usize,
    pub uda_style_context_active: Style,
    pub uda_style_calendar_title: Style,
    pub uda_shortcuts: Vec<String>,
}

impl Config {
    pub fn default() -> Result<Self, Box<dyn Error>> {
        let bool_collection = Self::get_bool_collection();
        Ok(Self {
            enabled: true,
            obfuscate: bool_collection.get("obfuscate").cloned().unwrap_or(false),
            print_empty_columns: bool_collection.get("print_empty_columns").cloned().unwrap_or(false),
            color: Self::get_color_collection()?,
            filter: Self::get_filter(),
            data_location: Self::get_data_location(),
            due: Self::get_due(),
            rule_precedence_color: Self::get_rule_precedence_color(),
            uda_task_report_show_info: Self::get_uda_task_report_show_info(),
            uda_task_report_looping: Self::get_uda_task_report_looping(),
            uda_selection_indicator: Self::get_uda_selection_indicator(),
            uda_mark_indicator: Self::get_uda_mark_indicator(),
            uda_unmark_indicator: Self::get_uda_unmark_indicator(),
            uda_selection_bold: Self::get_uda_selection_bold(),
            uda_selection_italic: Self::get_uda_selection_italic(),
            uda_selection_dim: Self::get_uda_selection_dim(),
            uda_selection_blink: Self::get_uda_selection_blink(),
            uda_calendar_months_per_row: Self::get_uda_months_per_row(),
            uda_style_calendar_title: Self::get_uda_style("calendar.title").unwrap_or_default(),
            uda_style_context_active: Self::get_uda_style("context.active").unwrap_or_default(),
            uda_shortcuts: Self::get_uda_shortcuts(),
        })
    }

    fn get_bool_collection() -> HashMap<String, bool> {
        HashMap::new()
    }

    fn get_uda_shortcuts() -> Vec<String> {
        let mut v = vec![];
        for s in 0..=9 {
            let c = format!("uda.taskwarrior-tui.shortcuts.{}", s);
            let s = Self::get_config(&c).unwrap_or_default();
            v.push(s);
        }
        v
    }

    fn get_uda_style(config: &str) -> Option<Style> {
        let c = format!("uda.taskwarrior-tui.style.{}", config);
        let s = Self::get_config(&c)?;
        Some(Self::get_tcolor(&s))
    }

    fn get_color_collection() -> Result<HashMap<String, Style>, Box<dyn Error>> {
        let mut color_collection = HashMap::new();
        let output = Command::new("task").arg("rc.color=off").arg("show").output()?;

        let data = String::from_utf8_lossy(&output.stdout);
        for line in data.split('\n') {
            if line.starts_with("color.") {
                let mut i = line.split(' ');
                let attribute = i.next();
                let line = i.collect::<Vec<_>>().join(" ");
                let line = line.trim_start_matches(' ');
                let tcolor = Self::get_tcolor(&line);
                match attribute {
                    Some(attr) => color_collection.insert(attr.to_string(), tcolor),
                    None => None,
                };
            }
        }

        Ok(color_collection)
    }

    pub fn get_tcolor(line: &str) -> Style {
        let (foreground, background) = line.split_at(line.to_lowercase().find("on ").unwrap_or_else(|| line.len()));
        let (mut foreground, mut background) = (String::from(foreground), String::from(background));
        background = background.replace("on ", "");
        let mut modifiers = Modifier::empty();
        if foreground.contains("bright") {
            foreground = foreground.replace("bright ", "");
            background = background.replace("bright ", "");
            background.insert_str(0, "bright ");
        }
        foreground = foreground.replace("grey", "gray");
        background = background.replace("grey", "gray");
        if foreground.contains("underline") {
            modifiers |= Modifier::UNDERLINED;
        }
        let foreground = foreground.replace("underline ", "");
        // TODO: use bold, bright boolean flags
        if foreground.contains("bold") {
            modifiers |= Modifier::BOLD;
        }
        // let foreground = foreground.replace("bold ", "");
        if foreground.contains("inverse") {
            modifiers |= Modifier::REVERSED;
        }
        let foreground = foreground.replace("inverse ", "");
        let mut style = Style::default();
        if let Some(fg) = Self::get_color_foreground(foreground.as_str()) {
            style = style.fg(fg);
        }
        if let Some(bg) = Self::get_color_background(background.as_str()) {
            style = style.bg(bg);
        }
        style = style.add_modifier(modifiers);
        style
    }

    fn get_color_foreground(s: &str) -> Option<Color> {
        let s = s.trim_start();
        let s = s.trim_end();
        if s.contains("color") {
            let c = s.trim_start_matches("color").parse::<u8>().unwrap_or_default();
            Some(Color::Indexed(c))
        } else if s.contains("gray") {
            let c = 232 + s.trim_start_matches("gray").parse::<u8>().unwrap_or_default();
            Some(Color::Indexed(c))
        } else if s.contains("rgb") {
            let red = (s.as_bytes()[3] as char).to_digit(10).unwrap_or_default() as u8;
            let green = (s.as_bytes()[4] as char).to_digit(10).unwrap_or_default() as u8;
            let blue = (s.as_bytes()[5] as char).to_digit(10).unwrap_or_default() as u8;
            let c = 16 + red * 36 + green * 6 + blue;
            Some(Color::Indexed(c))
        } else if s == "bold black" {
            Some(Color::Indexed(8))
        } else if s == "bold red" {
            Some(Color::Indexed(9))
        } else if s == "bold green" {
            Some(Color::Indexed(10))
        } else if s == "bold yellow" {
            Some(Color::Indexed(11))
        } else if s == "bold blue" {
            Some(Color::Indexed(12))
        } else if s == "bold magenta" {
            Some(Color::Indexed(13))
        } else if s == "bold cyan" {
            Some(Color::Indexed(14))
        } else if s == "bold white" {
            Some(Color::Indexed(15))
        } else if s == "black" {
            Some(Color::Indexed(0))
        } else if s == "red" {
            Some(Color::Indexed(1))
        } else if s == "green" {
            Some(Color::Indexed(2))
        } else if s == "yellow" {
            Some(Color::Indexed(3))
        } else if s == "blue" {
            Some(Color::Indexed(4))
        } else if s == "magenta" {
            Some(Color::Indexed(5))
        } else if s == "cyan" {
            Some(Color::Indexed(6))
        } else if s == "white" {
            Some(Color::Indexed(7))
        } else {
            None
        }
    }

    fn get_color_background(s: &str) -> Option<Color> {
        let s = s.trim_start();
        let s = s.trim_end();
        if s.contains("bright color") {
            let s = s.trim_start_matches("bright ");
            let c = s.trim_start_matches("color").parse::<u8>().unwrap_or_default();
            Some(Color::Indexed(c.wrapping_shl(8)))
        } else if s.contains("color") {
            let c = s.trim_start_matches("color").parse::<u8>().unwrap_or_default();
            Some(Color::Indexed(c))
        } else if s.contains("gray") {
            let s = s.trim_start_matches("bright ");
            let c = 232 + s.trim_start_matches("gray").parse::<u8>().unwrap_or_default();
            Some(Color::Indexed(c.wrapping_shl(8)))
        } else if s.contains("rgb") {
            let s = s.trim_start_matches("bright ");
            let red = (s.as_bytes()[3] as char).to_digit(10).unwrap_or_default() as u8;
            let green = (s.as_bytes()[4] as char).to_digit(10).unwrap_or_default() as u8;
            let blue = (s.as_bytes()[5] as char).to_digit(10).unwrap_or_default() as u8;
            let c = 16 + red * 36 + green * 6 + blue;
            Some(Color::Indexed(c.wrapping_shl(8)))
        } else if s == "bright black" {
            Some(Color::Indexed(8))
        } else if s == "bright red" {
            Some(Color::Indexed(9))
        } else if s == "bright green" {
            Some(Color::Indexed(10))
        } else if s == "bright yellow" {
            Some(Color::Indexed(11))
        } else if s == "bright blue" {
            Some(Color::Indexed(12))
        } else if s == "bright magenta" {
            Some(Color::Indexed(13))
        } else if s == "bright cyan" {
            Some(Color::Indexed(14))
        } else if s == "bright white" {
            Some(Color::Indexed(15))
        } else if s == "black" {
            Some(Color::Indexed(0))
        } else if s == "red" {
            Some(Color::Indexed(1))
        } else if s == "green" {
            Some(Color::Indexed(2))
        } else if s == "yellow" {
            Some(Color::Indexed(3))
        } else if s == "blue" {
            Some(Color::Indexed(4))
        } else if s == "magenta" {
            Some(Color::Indexed(5))
        } else if s == "cyan" {
            Some(Color::Indexed(6))
        } else if s == "white" {
            Some(Color::Indexed(7))
        } else {
            None
        }
    }

    fn get_config(config: &str) -> Option<String> {
        let output = Command::new("task")
            .arg("rc.color=off")
            .arg("show")
            .arg(config)
            .output()
            .expect("Unable to run `task show`");

        let data = String::from_utf8_lossy(&output.stdout);

        for line in data.split('\n') {
            let config = &config.replace('-', "_");
            if line.starts_with(config) {
                return Some(line.trim_start_matches(config).trim_start().trim_end().to_string());
            }
        }
        None
    }

    fn get_due() -> usize {
        Self::get_config("due")
            .unwrap_or_default()
            .parse::<usize>()
            .unwrap_or(7)
    }

    fn get_rule_precedence_color() -> Vec<String> {
        let data = Self::get_config("rule.precedence.color").unwrap();
        data.split(',').map(|s| s.to_string()).collect::<Vec<_>>()
    }

    fn get_filter() -> String {
        Self::get_config("report.next.filter").unwrap()
    }

    fn get_data_location() -> String {
        Self::get_config("data.location").unwrap()
    }

    fn get_uda_task_report_show_info() -> bool {
        Self::get_config("uda.taskwarrior-tui.task-report.show-info")
            .unwrap_or_default()
            .get_bool()
            .unwrap_or(true)
    }

    fn get_uda_task_report_looping() -> bool {
        Self::get_config("uda.taskwarrior-tui.task-report.looping")
            .unwrap_or_default()
            .get_bool()
            .unwrap_or(true)
    }

    fn get_uda_selection_indicator() -> String {
        let indicator = Self::get_config("uda.taskwarrior-tui.selection.indicator");
        match indicator {
            None => "• ".to_string(),
            Some(indicator) => format!("{} ", indicator),
        }
    }

    fn get_uda_mark_indicator() -> String {
        let indicator = Self::get_config("uda.taskwarrior-tui.mark.indicator");
        match indicator {
            None => "☑ ".to_string(),
            Some(indicator) => format!("{} ", indicator),
        }
    }

    fn get_uda_unmark_indicator() -> String {
        let indicator = Self::get_config("uda.taskwarrior-tui.unmark.indicator");
        match indicator {
            None => "☐ ".to_string(),
            Some(indicator) => format!("{} ", indicator),
        }
    }

    fn get_uda_selection_bold() -> bool {
        Self::get_config("uda.taskwarrior-tui.selection.bold")
            .unwrap_or_default()
            .get_bool()
            .unwrap_or(true)
    }

    fn get_uda_selection_italic() -> bool {
        Self::get_config("uda.taskwarrior-tui.selection.italic")
            .unwrap_or_default()
            .get_bool()
            .unwrap_or(false)
    }

    fn get_uda_selection_dim() -> bool {
        Self::get_config("uda.taskwarrior-tui.selection.dim")
            .unwrap_or_default()
            .get_bool()
            .unwrap_or(false)
    }

    fn get_uda_selection_blink() -> bool {
        Self::get_config("uda.taskwarrior-tui.selection.blink")
            .unwrap_or_default()
            .get_bool()
            .unwrap_or(false)
    }

    fn get_uda_months_per_row() -> usize {
        Self::get_config("uda.taskwarrior-tui.calendar.months-per-row")
            .unwrap_or_default()
            .parse::<usize>()
            .unwrap_or(4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_colors() {
        let tc = Config::default();
        let c = Config::get_tcolor("red on blue");
        assert_eq!(c.fg.unwrap(), Color::Indexed(1));
        assert_eq!(c.bg.unwrap(), Color::Indexed(4));

        let c = Config::get_tcolor("bold red");
        assert_eq!(c.fg.unwrap(), Color::Indexed(9));
        assert!(c.bg.is_none());
        assert_eq!(c.add_modifier & Modifier::BOLD, Modifier::BOLD);

        let c = Config::get_tcolor("white on red");
        assert_eq!(c.fg.unwrap(), Color::Indexed(7));
        assert_eq!(c.bg.unwrap(), Color::Indexed(1));

        let c = Config::get_tcolor("blue");
        assert_eq!(c.fg.unwrap(), Color::Indexed(4));
        assert!(c.bg.is_none());

        let c = Config::get_tcolor("black on bright green");
        assert_eq!(c.fg.unwrap(), Color::Indexed(0));
        assert_eq!(c.bg.unwrap(), Color::Indexed(10));

        let c = Config::get_tcolor("magenta");
        assert_eq!(c.fg.unwrap(), Color::Indexed(5));
        assert!(c.bg.is_none());

        let c = Config::get_tcolor("white on green");
        assert_eq!(c.fg.unwrap(), Color::Indexed(7));
        assert_eq!(c.bg.unwrap(), Color::Indexed(2));

        let c = Config::get_tcolor("black on white");
        assert_eq!(c.fg.unwrap(), Color::Indexed(0));
        assert_eq!(c.bg.unwrap(), Color::Indexed(7));

        let c = Config::get_tcolor("black on bright white");
        assert_eq!(c.fg.unwrap(), Color::Indexed(0));
        assert_eq!(c.bg.unwrap(), Color::Indexed(15));

        let c = Config::get_tcolor("bold white");
        assert_eq!(c.fg.unwrap(), Color::Indexed(15));
        assert!(c.bg.is_none());

        let c = Config::get_tcolor("white");
        assert_eq!(c.fg.unwrap(), Color::Indexed(7));
        assert!(c.bg.is_none());

        let c = Config::get_tcolor("bold yellow");
        assert_eq!(c.fg.unwrap(), Color::Indexed(11));
        assert!(c.bg.is_none());

        let c = Config::get_tcolor("green");
        assert_eq!(c.fg.unwrap(), Color::Indexed(2));
        assert!(c.bg.is_none());

        let c = Config::get_tcolor("yellow");
        assert_eq!(c.fg.unwrap(), Color::Indexed(3));
        assert!(c.bg.is_none());

        let c = Config::get_tcolor("red");
        assert_eq!(c.fg.unwrap(), Color::Indexed(1));
        assert!(c.bg.is_none());

        let c = Config::get_tcolor("bold red");
        assert_eq!(c.fg.unwrap(), Color::Indexed(9));
        assert!(c.bg.is_none());

        let c = Config::get_tcolor("on green");
        assert!(c.fg.is_none());
        assert_eq!(c.bg.unwrap(), Color::Indexed(2));

        let c = Config::get_tcolor("on red");
        assert!(c.fg.is_none());
        assert_eq!(c.bg.unwrap(), Color::Indexed(1));

        let c = Config::get_tcolor("on yellow");
        assert!(c.fg.is_none());
        assert_eq!(c.bg.unwrap(), Color::Indexed(3));

        let c = Config::get_tcolor("black on red");
        assert_eq!(c.fg.unwrap(), Color::Indexed(0));
        assert_eq!(c.bg.unwrap(), Color::Indexed(1));

        let c = Config::get_tcolor("black on yellow");
        assert_eq!(c.fg.unwrap(), Color::Indexed(0));
        assert_eq!(c.bg.unwrap(), Color::Indexed(3));

        let c = Config::get_tcolor("black on green");
        assert_eq!(c.fg.unwrap(), Color::Indexed(0));
        assert_eq!(c.bg.unwrap(), Color::Indexed(2));

        let c = Config::get_tcolor("white on black");
        assert_eq!(c.fg.unwrap(), Color::Indexed(7));
        assert_eq!(c.bg.unwrap(), Color::Indexed(0));

        let c = Config::get_tcolor("black on green");
        assert_eq!(c.fg.unwrap(), Color::Indexed(0));
        assert_eq!(c.bg.unwrap(), Color::Indexed(2));

        let c = Config::get_tcolor("white on red");
        assert_eq!(c.fg.unwrap(), Color::Indexed(7));
        assert_eq!(c.bg.unwrap(), Color::Indexed(1));

        let c = Config::get_tcolor("bold white on red");
        assert_eq!(c.fg.unwrap(), Color::Indexed(15));
        assert_eq!(c.bg.unwrap(), Color::Indexed(1));

        let c = Config::get_tcolor("black on bright yellow");
        assert_eq!(c.fg.unwrap(), Color::Indexed(0));
        assert_eq!(c.bg.unwrap(), Color::Indexed(11));

        let c = Config::get_tcolor("black on bright red");
        assert_eq!(c.fg.unwrap(), Color::Indexed(0));
        assert_eq!(c.bg.unwrap(), Color::Indexed(9));

        let c = Config::get_tcolor("bold white on bright blue");
        assert_eq!(c.fg.unwrap(), Color::Indexed(15));
        assert_eq!(c.bg.unwrap(), Color::Indexed(12));

        let c = Config::get_tcolor("white on bright black");
        assert_eq!(c.fg.unwrap(), Color::Indexed(7));
        assert_eq!(c.bg.unwrap(), Color::Indexed(8));

        let c = Config::get_tcolor("bold blue");
        assert_eq!(c.fg.unwrap(), Color::Indexed(12));
        assert!(c.bg.is_none());
    }
}
