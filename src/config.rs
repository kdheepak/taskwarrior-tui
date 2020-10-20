use std::collections::HashMap;
use std::process::Command;
use std::str;
use tui::style::Color;

#[derive(Debug, Clone, Copy)]
pub struct TColor {
    pub fg: Color,
    pub bg: Color,
}

impl Default for TColor {
    fn default() -> Self {
        TColor::default()
    }
}

impl TColor {
    pub fn default() -> Self {
        TColor {
            fg: Color::Black,
            bg: Color::White,
        }
    }
}

#[derive(Debug)]
pub struct TConfig {
    pub enabled: bool,
    pub color: HashMap<String, TColor>,
    pub obfuscate: bool,
    pub print_empty_columns: bool,
    pub rule_precedence_color: Vec<String>,
}

impl TConfig {
    pub fn default() -> Self {
        let bool_collection = Self::get_bool_collection();
        Self {
            enabled: true,
            obfuscate: bool_collection.get("obfuscate").cloned().unwrap_or(false),
            print_empty_columns: bool_collection.get("print_empty_columns").cloned().unwrap_or(false),
            color: Self::get_color_collection(),
            rule_precedence_color: Self::get_rule_precedence_color(),
        }
    }

    fn get_bool_collection() -> HashMap<String, bool> {
        HashMap::new()
    }

    fn get_color_collection() -> HashMap<String, TColor> {
        let output = Command::new("task")
            .arg("rc.color=off")
            .arg("show")
            .output()
            .expect("Unable to run `task show`");

        let data = String::from_utf8(output.stdout).expect("Unable to convert stdout to string");
        let mut color_collection = HashMap::new();
        for line in data.split('\n') {
            if line.starts_with("color.") {
                let mut i = line.split(" ");
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
        color_collection
    }

    fn get_tcolor(line: &str) -> TColor {
        if line.contains(" on ") {
            let foreground = line.split(' ').collect::<Vec<&str>>()[0];
            let background = line.split(' ').collect::<Vec<&str>>()[2];
            TColor {
                fg: Self::get_color(foreground, Color::Black),
                bg: Self::get_color(background, Color::White),
            }
        } else if line.contains("on ") {
            let background = line.split(' ').collect::<Vec<&str>>()[1];
            TColor {
                fg: Color::Black,
                bg: Self::get_color(background, Color::White),
            }
        } else {
            let foreground = line;
            TColor {
                fg: Self::get_color(foreground, Color::Black),
                bg: Color::White,
            }
        }
    }

    fn get_color(s: &str, default: Color) -> Color {
        if s.starts_with("color") {
            let fg = (s.as_bytes()[5] as char).to_digit(10).unwrap() as u8;
            Color::Indexed(fg)
        } else if s.starts_with("rgb") {
            let red = (s.as_bytes()[3] as char).to_digit(10).unwrap() as u8;
            let green = (s.as_bytes()[4] as char).to_digit(10).unwrap() as u8;
            let blue = (s.as_bytes()[5] as char).to_digit(10).unwrap() as u8;
            Color::Indexed(16 + red * 36 + green * 6 + blue)
        } else if s == "black" {
            Color::Black
        } else if s == "red" {
            Color::Red
        } else if s == "green" {
            Color::Green
        } else if s == "yellow" {
            Color::Yellow
        } else if s == "blue" {
            Color::Blue
        } else if s == "magenta" {
            Color::Magenta
        } else if s == "cyan" {
            Color::Cyan
        } else if s == "white" {
            Color::White
        } else {
            default
        }
    }

    fn get_rule_precedence_color() -> Vec<String> {

        let output = Command::new("task")
            .arg("rc.color=off")
            .arg("show")
            .arg("rule.precedence.color")
            .output()
            .expect("Unable to run `task show`");

        let data = String::from_utf8(output.stdout).expect("Unable to convert stdout to string");
        let mut rule_precedence_color = vec![];
        for line in data.split('\n') {
            if line.starts_with("rule.precedence.color ") {
                rule_precedence_color = line
                    .trim_start_matches("rule.precedence.color ")
                    .split(',')
                    .map(|s| s.trim_end_matches('.'))
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            }
        }

        return rule_precedence_color;
    }

}

#[cfg(test)]
mod tests {
    use crate::config::TConfig;
    #[test]
    fn test_colors() {
        let tc = TConfig::default();
        dbg!(tc);
    }
}
