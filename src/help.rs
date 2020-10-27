use std::cmp;

use tui::{
    layout::{Alignment, Rect},
    buffer::Buffer,
    style::{Modifier, Style},
    text::{Text, Span, Spans},
    widgets::{Block, Widget, BorderType, Borders, Clear, Paragraph},
};


const TEXT: &str = include_str!("../KEYBINDINGS.md");

pub struct Help {
    title: String,
}

impl Help {
    pub fn new() -> Self {
        Self {
            title: "Help".to_string(),
        }
    }
}

impl Widget for &Help {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let text: Vec<Spans> = TEXT.lines()
            .map(|line| Spans::from(format!("{}\n", line)))
            .collect();

        Paragraph::new(text)
            .block(
                Block::default()
                    .title(Span::styled(&self.title, Style::default().add_modifier(Modifier::BOLD)))
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
            )
            .alignment(Alignment::Left)
            .render(area, buf);
    }
}
