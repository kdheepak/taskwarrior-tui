use std::collections::HashMap;
use std::error::Error;
use std::process::Command;
use std::str;
use tui::style::{Color, Modifier};

#[derive(Debug, Clone)]
pub struct TColor {
    pub fg: Color,
    pub bg: Color,
    pub modifiers: Vec<Modifier>,
}

impl Default for TColor {
    fn default() -> Self {
        TColor::default()
    }
}

impl TColor {
    pub fn default() -> Self {
        Self {
            fg: Color::Reset,
            bg: Color::Reset,
            modifiers: vec![],
        }
    }

    pub fn new(fg: Color, bg: Color, modifiers: Vec<Modifier>) -> Self {
        Self { fg, bg, modifiers }
    }
}

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
    pub color: HashMap<String, TColor>,
    pub filter: String,
    pub obfuscate: bool,
    pub print_empty_columns: bool,
    pub rule_precedence_color: Vec<String>,
    pub uda_task_report_show_info: bool,
    pub uda_task_report_looping: bool,
    pub uda_selection_indicator: String,
    pub uda_selection_bold: bool,
    pub uda_selection_italic: bool,
    pub uda_selection_dim: bool,
    pub uda_selection_blink: bool,
    pub uda_calendar_months_per_row: usize,
    pub uda_style_context_active: TColor,
    pub uda_style_calendar_title: TColor,
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
            rule_precedence_color: Self::get_rule_precedence_color(),
            uda_task_report_show_info: Self::get_uda_task_report_show_info(),
            uda_task_report_looping: Self::get_uda_task_report_looping(),
            uda_selection_indicator: Self::get_uda_selection_indicator(),
            uda_selection_bold: Self::get_uda_selection_bold(),
            uda_selection_italic: Self::get_uda_selection_italic(),
            uda_selection_dim: Self::get_uda_selection_dim(),
            uda_selection_blink: Self::get_uda_selection_blink(),
            uda_calendar_months_per_row: Self::get_uda_months_per_row(),
            uda_style_calendar_title: Self::get_uda_style("calendar.title")
                .unwrap_or_else(|| TColor::new(Color::Reset, Color::Reset, vec![])),
            uda_style_context_active: Self::get_uda_style("context.active")
                .unwrap_or_else(|| TColor::new(Color::Reset, Color::Reset, vec![])),
        })
    }

    fn get_bool_collection() -> HashMap<String, bool> {
        HashMap::new()
    }

    fn get_uda_style(config: &str) -> Option<TColor> {
        let c = format!("uda.taskwarrior-tui.style.{}", config);
        let s = Self::get_config(&c);
        if s.is_empty() {
            None
        } else {
            Some(Self::get_tcolor(&s))
        }
    }

    fn get_color_collection() -> Result<HashMap<String, TColor>, Box<dyn Error>> {
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

    fn get_tcolor(line: &str) -> TColor {
        let (foreground, background) = line.split_at(line.to_lowercase().find("on ").unwrap_or_else(|| line.len()));
        let (mut foreground, mut background) = (String::from(foreground), String::from(background));
        background = background.replace("on ", "");
        let mut modifiers = vec![];
        if foreground.contains("bright") {
            foreground = foreground.replace("bright ", "");
            background = background.replace("bright ", "");
            background.insert_str(0, "bright ");
        }
        foreground = foreground.replace("grey", "gray");
        background = background.replace("grey", "gray");
        if foreground.contains("underline") {
            modifiers.push(Modifier::UNDERLINED);
        }
        let foreground = foreground.replace("underline ", "");
        if foreground.contains("bold") {
            modifiers.push(Modifier::BOLD);
        }
        let foreground = foreground.replace("bold ", "");
        if foreground.contains("inverse") {
            modifiers.push(Modifier::REVERSED);
        }
        let foreground = foreground.replace("inverse ", "");
        TColor {
            fg: Self::get_color_foreground(foreground.as_str(), Color::Reset),
            bg: Self::get_color_background(background.as_str(), Color::Reset),
            modifiers,
        }
    }
    fn get_color_foreground(s: &str, default: Color) -> Color {
        let s = s.trim_start();
        let s = s.trim_end();
        if s.contains("color") {
            let s = s.trim_start_matches("bright ");
            let c = s.trim_start_matches("color").parse::<u8>().unwrap_or_default();
            Color::Indexed(c)
        } else if s.contains("gray") {
            let s = s.trim_start_matches("bright ");
            let c = 232 + s.trim_start_matches("gray").parse::<u8>().unwrap_or_default();
            Color::Indexed(c)
        } else if s.contains("rgb") {
            let s = s.trim_start_matches("bright ");
            let red = (s.as_bytes()[3] as char).to_digit(10).unwrap_or_default() as u8;
            let green = (s.as_bytes()[4] as char).to_digit(10).unwrap_or_default() as u8;
            let blue = (s.as_bytes()[5] as char).to_digit(10).unwrap_or_default() as u8;
            let c = 16 + red * 36 + green * 6 + blue;
            Color::Indexed(c)
        } else if s == "bright red" {
            Color::Red
        } else if s == "bright green" {
            Color::Green
        } else if s == "bright yellow" {
            Color::Yellow
        } else if s == "bright blue" {
            Color::Blue
        } else if s == "bright magenta" {
            Color::Magenta
        } else if s == "bright cyan" {
            Color::Cyan
        } else if s == "bright white" {
            Color::White
        } else if s == "bright black" {
            Color::Black
        } else if s.contains("red") {
            Color::LightRed
        } else if s.contains("green") {
            Color::LightGreen
        } else if s.contains("yellow") {
            Color::LightYellow
        } else if s.contains("blue") {
            Color::LightBlue
        } else if s.contains("magenta") {
            Color::LightMagenta
        } else if s.contains("cyan") {
            Color::LightCyan
        } else if s.contains("white") {
            Color::Indexed(7)
        } else if s.contains("black") {
            Color::Indexed(0)
        } else {
            default
        }
    }

    fn get_color_background(s: &str, default: Color) -> Color {
        let s = s.trim_start();
        let s = s.trim_end();
        if s.contains("color") {
            let s = s.trim_start_matches("bright ");
            let c = s.trim_start_matches("color").parse::<u8>().unwrap_or_default();
            Color::Indexed(c.wrapping_shl(8))
        } else if s.contains("gray") {
            let s = s.trim_start_matches("bright ");
            let c = 232 + s.trim_start_matches("gray").parse::<u8>().unwrap_or_default();
            Color::Indexed(c.wrapping_shl(8))
        } else if s.contains("rgb") {
            let s = s.trim_start_matches("bright ");
            let red = (s.as_bytes()[3] as char).to_digit(10).unwrap_or_default() as u8;
            let green = (s.as_bytes()[4] as char).to_digit(10).unwrap_or_default() as u8;
            let blue = (s.as_bytes()[5] as char).to_digit(10).unwrap_or_default() as u8;
            let c = 16 + red * 36 + green * 6 + blue;
            Color::Indexed(c.wrapping_shl(8))
        } else if s == "bright red" {
            Color::LightRed
        } else if s == "bright green" {
            Color::LightGreen
        } else if s == "bright yellow" {
            Color::LightYellow
        } else if s == "bright blue" {
            Color::LightBlue
        } else if s == "bright magenta" {
            Color::LightMagenta
        } else if s == "bright cyan" {
            Color::LightCyan
        } else if s == "bright white" {
            Color::White
        } else if s == "bright black" {
            Color::Black
        } else if s.contains("red") {
            Color::Red
        } else if s.contains("green") {
            Color::Green
        } else if s.contains("yellow") {
            Color::Yellow
        } else if s.contains("blue") {
            Color::Blue
        } else if s.contains("magenta") {
            Color::Magenta
        } else if s.contains("cyan") {
            Color::Cyan
        } else if s.contains("white") {
            Color::Indexed(7)
        } else if s.contains("black") {
            Color::Indexed(0)
        } else {
            default
        }
    }

    fn get_config(config: &str) -> String {
        let output = Command::new("task")
            .arg("rc.color=off")
            .arg("show")
            .arg(config)
            .output()
            .expect("Unable to run `task show`");

        let data = String::from_utf8_lossy(&output.stdout);

        for line in data.split('\n') {
            if line.starts_with(config) {
                return line.trim_start_matches(config).trim_start().trim_end().to_string();
            } else if line.starts_with(&config.replace('-', "_")) {
                return line
                    .trim_start_matches(&config.replace('-', "_"))
                    .trim_start()
                    .trim_end()
                    .to_string();
            }
        }
        "".to_string()
    }

    fn get_rule_precedence_color() -> Vec<String> {
        let data = Self::get_config("rule.precedence.color");
        data.split(',')
            .filter(|s| !s.ends_with('.'))
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
    }

    fn get_filter() -> String {
        Self::get_config("report.next.filter")
    }

    fn get_uda_task_report_show_info() -> bool {
        Self::get_config("uda.taskwarrior-tui.task-report.show-info")
            .get_bool()
            .unwrap_or(true)
    }

    fn get_uda_task_report_looping() -> bool {
        Self::get_config("uda.taskwarrior-tui.task-report.looping")
            .get_bool()
            .unwrap_or(true)
    }

    fn get_uda_selection_indicator() -> String {
        let indicator = Self::get_config("uda.taskwarrior-tui.selection.indicator");
        if indicator.is_empty() {
            "• ".to_string()
        } else {
            format!("{} ", indicator)
        }
    }

    fn get_uda_selection_bold() -> bool {
        Self::get_config("uda.taskwarrior-tui.selection.bold")
            .get_bool()
            .unwrap_or(true)
    }

    fn get_uda_selection_italic() -> bool {
        Self::get_config("uda.taskwarrior-tui.selection.italic")
            .get_bool()
            .unwrap_or(false)
    }

    fn get_uda_selection_dim() -> bool {
        Self::get_config("uda.taskwarrior-tui.selection.dim")
            .get_bool()
            .unwrap_or(false)
    }

    fn get_uda_selection_blink() -> bool {
        Self::get_config("uda.taskwarrior-tui.selection.blink")
            .get_bool()
            .unwrap_or(false)
    }

    fn get_uda_months_per_row() -> usize {
        Self::get_config("uda.taskwarrior-tui.calendar.months-per-row")
            .parse::<usize>()
            .unwrap_or(4)
    }
}

#[cfg(test)]
mod tests {
    use crate::config::Config;
    #[test]
    fn test_colors() {
        let tc = Config::default();
    }
}
