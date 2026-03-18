use ratatui::{
  backend::Backend,
  layout::{Alignment, Constraint, Direction, Layout, Rect},
  style::Style,
  symbols,
  text::{Line, Span},
  widgets::{Block, BorderType, Borders, Cell, LineGauge, Paragraph, Row, Table},
  Frame,
};

use crate::app::TaskwarriorTui;

pub fn draw(rect: &mut Frame, app: &TaskwarriorTui) {
  let size = rect.area();
  let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Length(3), Constraint::Min(10), Constraint::Length(3)].as_ref())
    .split(size);

  let title = draw_title(app.config.uda_style_title, app.config.uda_style_title_border);
  rect.render_widget(title, chunks[0]);
}

fn draw_title<'a>(title_style: Style, border_style: Style) -> Paragraph<'a> {
  Paragraph::new("Taskwarrior TUI")
    .style(title_style)
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::ALL).style(border_style).border_type(BorderType::Plain))
}
