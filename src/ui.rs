use tui::{
  backend::Backend,
  layout::{Alignment, Constraint, Direction, Layout},
  style::{Color, Modifier, Style},
  widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table, TableState},
  Frame, Terminal,
};

use crate::app::App;

/// Renders the user interface widgets.
pub fn render<B: Backend>(f: &mut Frame<'_, B>, app: &mut App) {
  let size = f.size();
  let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Length(3), Constraint::Min(10), Constraint::Length(3)].as_ref())
    .split(size);

  let title = draw_title();
  f.render_widget(title, chunks[0]);
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
