use tui::{
  backend::Backend,
  buffer::Buffer,
  layout::{Margin, Rect},
  style::{Color, Style},
  symbols::{block::FULL, line::DOUBLE_VERTICAL},
  widgets::Widget,
  Frame,
};

pub struct Scrollbar {
  pub pos: u16,
  pub len: u16,
  pub pos_style: Style,
  pub pos_symbol: String,
  pub area_style: Style,
  pub area_symbol: String,
}

impl Scrollbar {
  pub fn new(pos: usize, len: usize) -> Self {
    Self {
      pos: pos as u16,
      len: len as u16,
      pos_style: Style::default(),
      pos_symbol: FULL.to_string(),
      area_style: Style::default(),
      area_symbol: DOUBLE_VERTICAL.to_string(),
    }
  }
}

impl Widget for Scrollbar {
  fn render(self, area: Rect, buf: &mut Buffer) {
    if area.height <= 2 {
      return;
    }

    if self.len == 0 {
      return;
    }

    let right = area.right().saturating_sub(1);

    if right <= area.left() {
      return;
    };

    let (top, height) = { (area.top() + 3, area.height.saturating_sub(4)) };

    for y in top..(top + height) {
      buf.set_string(right, y, self.area_symbol.clone(), self.area_style);
    }

    let progress = self.pos as f64 / self.len as f64;
    let progress = if progress > 1.0 { 1.0 } else { progress };
    let pos = height as f64 * progress;

    let pos = pos as i64 as u16;

    buf.set_string(right, top + pos, self.pos_symbol, self.pos_style);
  }
}
