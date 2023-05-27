use tui::{
  backend::Backend,
  layout::{Alignment, Constraint, Direction, Layout, Rect},
  style::{Color, Modifier, Style},
  symbols,
  text::{Span, Spans},
  widgets::{Block, BorderType, Borders, Cell, LineGauge, Paragraph, Row, Table},
  Frame,
};

use crate::app::TaskwarriorTui;

pub fn draw<B>(rect: &mut Frame<B>, app: &TaskwarriorTui)
where
  B: Backend,
{
  let size = rect.size();
  let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Length(3), Constraint::Min(10), Constraint::Length(3)].as_ref())
    .split(size);

  let title = draw_title();
  rect.render_widget(title, chunks[0]);
}

fn draw_title<'a>() -> Paragraph<'a> {
  Paragraph::new("Taskwarrior TUI")
    .style(Style::default().fg(Color::LightCyan))
    .alignment(Alignment::Center)
    .block(
      Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .border_type(BorderType::Plain),
    )
}
