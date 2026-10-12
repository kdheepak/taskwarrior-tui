//! Per-cell styling for the task report table.
//!
//! Row-wide colors come from taskwarrior color rules (see
//! `TaskwarriorTui::style_for_task`); this module adds cell-level overrides on
//! top: a dynamic urgency gradient and a distinct tint for the columns that
//! sit next to the description.

use ratatui::style::{Color, Modifier, Style};

/// Everything needed to compute cell style overrides for one report row.
pub struct CellStyleContext {
  /// Display index of the urgency column, if shown.
  pub urgency_col: Option<usize>,
  /// Display index of the description column, if shown.
  pub description_col: Option<usize>,
  /// Number of displayed columns.
  pub column_count: usize,
  /// Absolute urgency value at which the gradient saturates.
  pub urgency_cap: f64,
  /// Denominator used to normalize urgencies (max visible urgency, capped).
  pub urgency_denom: f64,
  /// Whether the dynamic urgency gradient is enabled.
  pub dynamic_urgency: bool,
  /// Style applied to columns adjacent to the description.
  pub near_description_style: Style,
}

impl CellStyleContext {
  /// Normalization denominator: proportional to the most urgent visible task,
  /// but a single outlier never compresses the scale beyond the cap.
  pub fn denom(max_urgency: f64, cap: f64) -> f64 {
    if max_urgency > 0.0 { max_urgency.min(cap) } else { cap }
  }

  /// Build the per-cell style overrides for a row with the given urgency.
  pub fn cell_styles(&self, urgency: Option<f64>) -> Vec<Option<Style>> {
    let mut cell_styles: Vec<Option<Style>> = vec![None; self.column_count];

    if self.dynamic_urgency
      && let (Some(uc), Some(u)) = (self.urgency_col, urgency)
      && uc < cell_styles.len()
    {
      let t = (u.min(self.urgency_cap) / self.urgency_denom).clamp(0.0, 1.0);
      let mut s = Style::default().fg(urgency_ramp_color(t));
      if t > 0.85 {
        s = s.add_modifier(Modifier::BOLD);
      }
      cell_styles[uc] = Some(s);
    }

    if let Some(dc) = self.description_col {
      for n in [dc.wrapping_sub(1), dc + 1] {
        if n < cell_styles.len() && n != dc && Some(n) != self.urgency_col && cell_styles[n].is_none() {
          cell_styles[n] = Some(self.near_description_style);
        }
      }
    }

    cell_styles
  }
}

/// Map a normalized urgency value [0, 1] onto a green -> yellow -> red ramp
/// in the 256-color cube (index = 16 + 36r + 6g + b, with r/g/b in 0..=5).
pub fn urgency_ramp_color(t: f64) -> Color {
  let t = t.clamp(0.0, 1.0);
  let (r, g) = if t < 0.5 {
    ((5.0 * t / 0.5).round() as u16, 4)
  } else {
    (5, (4.0 * (1.0 - (t - 0.5) / 0.5)).round() as u16)
  };
  Color::Indexed((16 + 36 * r + 6 * g) as u8)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn ramp_endpoints() {
    assert_eq!(urgency_ramp_color(0.0), Color::Indexed(40)); // green
    assert_eq!(urgency_ramp_color(1.0), Color::Indexed(196)); // red
  }

  #[test]
  fn denom_caps_outliers() {
    assert_eq!(CellStyleContext::denom(30.0, 15.0), 15.0);
    assert_eq!(CellStyleContext::denom(8.0, 15.0), 8.0);
    assert_eq!(CellStyleContext::denom(0.0, 15.0), 15.0);
  }

  #[test]
  fn urgency_cell_styled_and_neighbors_tinted() {
    let ctx = CellStyleContext {
      urgency_col: Some(3),
      description_col: Some(2),
      column_count: 4,
      urgency_cap: 15.0,
      urgency_denom: 15.0,
      dynamic_urgency: true,
      near_description_style: Style::default().fg(Color::Cyan),
    };
    let styles = ctx.cell_styles(Some(15.0));
    assert!(styles[3].is_some()); // urgency gradient wins over neighbor tint
    assert_eq!(styles[1], Some(Style::default().fg(Color::Cyan)));
    assert_eq!(styles[2], None); // description itself untouched
  }
}
