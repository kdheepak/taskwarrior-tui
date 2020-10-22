// Based on https://play.rust-lang.org/?gist=1057364daeee4cff472a&version=nightly
// See: https://old.reddit.com/r/rust/comments/37b6oo/the_calendar_example_challenge/crlmbsg/

use std::fmt;

const COL_WIDTH: usize = 21;

use chrono::{Datelike, Local, Duration, NaiveDate, NaiveDateTime, TimeZone, Month};

use tui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    symbols,
    widgets::{Block, Widget},
};

use std::cmp::min;
use unicode_width::UnicodeWidthStr;

#[derive(Debug, Clone)]
pub struct Calendar<'a> {
    pub block: Option<Block<'a>>,
    pub year: i32,
    pub days: Vec<(NaiveDate, NaiveDate)>,
    pub style: Style,
    pub months_per_row: usize,
}

impl<'a> Default for Calendar<'a> {
    fn default() -> Calendar<'a> {
        let year = Local::today().year().into();
        let days: Vec<_> = (0..12).into_iter().map(|i| {
            let first = NaiveDate::from_ymd(year, i+1, 1);
            (first, first - Duration::days(first.weekday().num_days_from_sunday() as i64))
        }).collect();
        Calendar {
            block: None,
            style: Default::default(),
            months_per_row: 0,
            year,
            days,
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
        let days: Vec<_> = (0..12).into_iter().map(|i| {
            let first = NaiveDate::from_ymd(year, i+1, 1);
            (first, first - Duration::days(first.weekday().num_days_from_sunday() as i64))
        }).collect();
        self.days = days;
        self
    }
}

impl <'a> Widget for Calendar<'a> {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
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
            return
        }

        let style = self.style;
        let today = Local::today();

        let year = self.year;
        let months: Vec<_> = (0..12).collect();
        let mut startm = 0;
        if self.months_per_row > area.width as usize / 8 / 3 || self.months_per_row <= 0 {
            self.months_per_row = area.width as usize / 8 / 3;
        }
        let mut y = area.y;
        y += 1;
        let x = area.x;
        let s = format!("{year:^width$}", year = year, width = area.width as usize - 4);
        buf.set_string(x, y, &s, Style::default().add_modifier(Modifier::BOLD));
        let startx = (area.width - 3 * 7 * self.months_per_row as u16 - self.months_per_row as u16) / 2;
        y += 2;
        loop {
            let endm = std::cmp::min(startm + self.months_per_row, 12);
            let mut x = area.x + startx;
            for c in startm..endm {
                if c > startm {
                    x += 1;
                }
                let d = &mut self.days[c];
                let m = d.0.month() as usize;
                let s = format!("{:^21}", month_names[m - 1]);
                buf.set_string(x, y, &s, Style::default());
                x += s.len() as u16;
            }
            y += 1;
            let mut x = area.x + startx;
            for c in startm..endm {
                buf.set_string(x as u16, y, " Su Mo Tu We Th Fr Sa", Style::default());
                x += 21 + 1;
            }
            y += 1;
            loop {
                let mut moredays = false;
                let mut x = area.x + startx;
                for c in startm..endm {
                    if c > startm {
                        x += 1;
                    }
                    let d = &mut self.days[c];
                    for _ in 0..7 {
                        let s = if d.0.month() == d.1.month() {
                            format!("{:>2}", d.1.day())
                        } else {
                            "   ".to_string()
                        };
                        if d.1 == Local::today().naive_local() {
                            buf.set_string(x, y, s, Style::default().add_modifier(Modifier::REVERSED));
                        } else {
                            buf.set_string(x, y, s, Style::default());
                        }
                        x += 3;
                        d.1 = d.1 + Duration::days(1);
                    }
                    moredays |= d.0.month() == d.1.month() || d.1 < d.0;
                }
                y += 1;
                if !moredays {
                    break;
                }
            }
            startm += self.months_per_row;
            if startm >= 12 {
                return;
            }
            y += 1;
        }
    }
}
