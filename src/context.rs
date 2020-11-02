use std::cmp;

use tui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, StatefulWidget, Widget},
};

#[derive(Debug, Clone, Default)]
pub struct Context {
    pub name: String,
    pub description: String,
    pub active: String,
}

impl Context {
    pub fn new(name: String, description: String, active: String) -> Self {
        Self {
            name,
            description,
            active,
        }
    }
}
