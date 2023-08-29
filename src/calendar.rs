// Based on https://gist.github.com/diwic/5c20a283ca3a03752e1a27b0f3ebfa30
// See https://old.reddit.com/r/rust/comments/4xneq5/the_calendar_example_challenge_ii_why_eddyb_all/

use std::fmt;

const COL_WIDTH: usize = 21;

use std::cmp::min;

use chrono::{format::Fixed, DateTime, Datelike, Duration, FixedOffset, Local, Month, NaiveDate, NaiveDateTime, TimeZone};
use ratatui::{
  buffer::Buffer,
  layout::Rect,
  style::{Color, Modifier, Style},
  symbols,
  widgets::{Block, Widget},
};

#[derive(Debug, Clone)]
pub struct Calendar<'a> {
  pub block: Option<Block<'a>>,
  pub year: i32,
  pub month: u32,
  pub style: Style,
  pub months_per_row: usize,
  pub date_style: Vec<(NaiveDate, Style)>,
  pub today_style: Style,
  pub start_on_monday: bool,
  pub title_background_color: Color,
}

impl<'a> Default for Calendar<'a> {
  fn default() -> Calendar<'a> {
    let year = Local::now().year();
    let month = Local::now().month();
    Calendar {
      block: None,
      style: Style::default(),
      months_per_row: 0,
      year,
      month,
      date_style: vec![],
      today_style: Style::default(),
      start_on_monday: false,
      title_background_color: Color::Reset,
    }
  }
}

impl<'a> Calendar<'a> {
  pub fn block(mut self, block: Block<'a>) -> Self {
    self.block = Some(block);
    self
  }

  pub fn style(mut self, style: Style) -> Self {
    self.style = style;
    self
  }

  pub fn year(mut self, year: i32) -> Self {
    self.year = year;
    if self.year < 0 {
      self.year = 0;
    }
    self
  }

  pub fn month(mut self, month: u32) -> Self {
    self.month = month;
    self
  }

  pub fn date_style(mut self, date_style: Vec<(NaiveDate, Style)>) -> Self {
    self.date_style = date_style;
    self
  }

  pub fn today_style(mut self, today_style: Style) -> Self {
    self.today_style = today_style;
    self
  }

  pub fn months_per_row(mut self, months_per_row: usize) -> Self {
    self.months_per_row = months_per_row;
    self
  }

  pub fn start_on_monday(mut self, start_on_monday: bool) -> Self {
    self.start_on_monday = start_on_monday;
    self
  }
}

impl<'a> Widget for Calendar<'a> {
  fn render(mut self, area: Rect, buf: &mut Buffer) {
    let month_names = Self::generate_month_names();
    buf.set_style(area, self.style);

    let area = match self.block.take() {
      Some(b) => {
        let inner_area = b.inner(area);
        b.render(area, buf);
        inner_area
      }
      None => area,
    };

    if area.height < 7 {
      return;
    }

    let style = self.style;
    let today = Local::now();

    let year = self.year;
    let month = self.month;

    let months: Vec<_> = (0..12).collect();

    let mut days: Vec<(NaiveDate, NaiveDate)> = months
      .iter()
      .map(|i| {
        let first = NaiveDate::from_ymd_opt(year, i + 1, 1).unwrap();
        let num_days = if self.start_on_monday {
          first.weekday().num_days_from_monday()
        } else {
          first.weekday().num_days_from_sunday()
        };
        (first, first - Duration::days(i64::from(num_days)))
      })
      .collect();

    let mut start_m = 0_usize;
    if self.months_per_row > area.width as usize / 8 / 3 || self.months_per_row == 0 {
      self.months_per_row = area.width as usize / 8 / 3;
    }
    let mut y = area.y;
    y += 1;

    let x = area.x;
    let s = format!("{year:^width$}", year = year, width = area.width as usize);

    let mut new_year = 0;
    let style = Style::default().add_modifier(Modifier::UNDERLINED);
    if self.year + new_year as i32 == today.year() {
      buf.set_string(x, y, &s, self.today_style.add_modifier(Modifier::UNDERLINED));
    } else {
      buf.set_string(x, y, &s, style);
    }

    let start_x = (area.width - 3 * 7 * self.months_per_row as u16 - self.months_per_row as u16) / 2;
    y += 2;
    loop {
      let endm = std::cmp::min(start_m + self.months_per_row, 12);
      let mut x = area.x + start_x;
      for (c, d) in days.iter_mut().enumerate().take(endm).skip(start_m) {
        if c > start_m {
          x += 1;
        }
        let m = d.0.month() as usize;
        let s = format!("{:^20}", month_names[m - 1]);
        let style = Style::default().bg(self.title_background_color);
        if m == today.month() as usize && self.year + new_year as i32 == today.year() {
          buf.set_string(x, y, &s, self.today_style);
        } else {
          buf.set_string(x, y, &s, style);
        }
        x += s.len() as u16 + 1;
      }
      y += 1;
      let mut x = area.x + start_x;
      for d in days.iter_mut().take(endm).skip(start_m) {
        let m = d.0.month() as usize;
        let style = Style::default().bg(self.title_background_color);
        let days_string = if self.start_on_monday {
          "Mo Tu We Th Fr Sa Su"
        } else {
          "Su Mo Tu We Th Fr Sa"
        };
        buf.set_string(x, y, days_string, style.add_modifier(Modifier::UNDERLINED));
        x += 21 + 1;
      }
      y += 1;
      loop {
        let mut moredays = false;
        let mut x = area.x + start_x;
        for c in start_m..endm {
          if c > start_m {
            x += 1;
          }
          let d = &mut days[c + new_year * 12];
          for _ in 0..7 {
            let s = if d.0.month() == d.1.month() {
              format!("{:>2}", d.1.day())
            } else {
              "   ".to_string()
            };
            let mut style = Style::default();
            let index = self.date_style.iter().position(|(date, style)| d.1 == *date);
            if let Some(i) = index {
              style = self.date_style[i].1;
            }
            if d.1 == Local::now().date_naive() {
              buf.set_string(x, y, s, self.today_style);
            } else {
              buf.set_string(x, y, s, style);
            }
            x += 3;
            d.1 += Duration::days(1);
          }
          moredays |= d.0.month() == d.1.month() || d.1 < d.0;
        }
        y += 1;
        if !moredays {
          break;
        }
      }
      start_m += self.months_per_row;
      y += 2;
      if y + 8 > area.height {
        break;
      } else if start_m >= 12 {
        start_m = 0;
        new_year += 1;
        days.append(
          &mut months
            .iter()
            .map(|i| {
              let first = NaiveDate::from_ymd_opt(self.year + new_year as i32, i + 1, 1).unwrap();
              (first, first - Duration::days(i64::from(first.weekday().num_days_from_sunday())))
            })
            .collect::<Vec<_>>(),
        );

        let x = area.x;
        let s = format!("{year:^width$}", year = self.year as usize + new_year, width = area.width as usize);
        let mut style = Style::default().add_modifier(Modifier::UNDERLINED);
        if self.year + new_year as i32 == today.year() {
          style = style.add_modifier(Modifier::BOLD);
        }
        buf.set_string(x, y, &s, style);
        y += 1;
      }
      y += 1;
    }
  }
}

impl<'a> Calendar<'a> {
  fn generate_month_names() -> [&'a str; 12] {
    let month_names = [
      Month::January.name(),
      Month::February.name(),
      Month::March.name(),
      Month::April.name(),
      Month::May.name(),
      Month::June.name(),
      Month::July.name(),
      Month::August.name(),
      Month::September.name(),
      Month::October.name(),
      Month::November.name(),
      Month::December.name(),
    ];
    month_names
  }
}
