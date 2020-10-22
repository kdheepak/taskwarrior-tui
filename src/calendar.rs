// Based on https://play.rust-lang.org/?gist=1057364daeee4cff472a&version=nightly
// See: https://old.reddit.com/r/rust/comments/37b6oo/the_calendar_example_challenge/crlmbsg/

use std::fmt;

const COL_WIDTH: usize = 21;

use Day::*;
use Month::*;

use chrono::{Datelike, Local, Duration, NaiveDate, NaiveDateTime, TimeZone};

use tui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    symbols,
    widgets::{Block, Widget},
};

use std::cmp::min;
use unicode_width::UnicodeWidthStr;

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum Day {
    Sun,
    Mon,
    Tue,
    Wed,
    Thu,
    Fri,
    Sat,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Month {
    Jan,
    Feb,
    Mar,
    Apr,
    May,
    Jun,
    Jul,
    Aug,
    Sep,
    Oct,
    Nov,
    Dec,
}
impl Month {
    fn len(self) -> u8 {
        match self {
            Jan => 31,
            Feb => 28,
            Mar => 31,
            Apr => 30,
            May => 31,
            Jun => 30,
            Jul => 31,
            Aug => 31,
            Sep => 30,
            Oct => 31,
            Nov => 30,
            Dec => 31,
        }
    }
    fn leap_len(self, leap_year: bool) -> u8 {
        match self {
            Feb => {
                if leap_year {
                    29
                } else {
                    28
                }
            }
            mon => mon.len(),
        }
    }
    fn first_day(self, year: i32) -> Day {
        let y = year - 1;
        let jan_first = (1 + (5 * (y % 4)) + (4 * (y % 100)) + (6 * (y % 400))) % 7;
        let mut len = 0;
        for m in Jan {
            if m == self {
                break;
            }
            len += m.leap_len(is_leap_year(year)) as i32;
        }
        match (len + jan_first) % 7 {
            0 => Sun,
            1 => Mon,
            2 => Tue,
            3 => Wed,
            4 => Thu,
            5 => Fri,
            _ => Sat,
        }
    }
}
impl Iterator for Month {
    type Item = Month;
    fn next(&mut self) -> Option<Month> {
        let ret = Some(*self);
        *self = match *self {
            Jan => Feb,
            Feb => Mar,
            Mar => Apr,
            Apr => May,
            May => Jun,
            Jun => Jul,
            Jul => Aug,
            Aug => Sep,
            Sep => Oct,
            Oct => Nov,
            Nov => Dec,
            Dec => Jan,
        };
        ret
    }
}
impl fmt::Display for Month {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match *self {
            Jan => "January",
            Feb => "February",
            Mar => "March",
            Apr => "April",
            May => "May",
            Jun => "June",
            Jul => "July",
            Aug => "August",
            Sep => "September",
            Oct => "October",
            Nov => "November",
            Dec => "December",
        };
        let padding = COL_WIDTH - name.len();
        write!(f, "{:1$}", "", padding / 2)?;
        if padding % 2 != 0 {
            f.write_str(" ")?;
        }
        f.write_str(name)?;
        write!(f, "{:1$}", "", padding / 2)
    }
}

#[derive(Debug, Clone)]
pub struct Calendar<'a> {
    pub block: Option<Block<'a>>,
    pub year: i32,
    pub days: Vec<(NaiveDate, NaiveDate)>,
    pub style: Style,
    pub months_per_row: i8,
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

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

impl <'a> Widget for Calendar<'a> {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
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
        let cols = area.width;

        let year = self.year;
        let leap_year = is_leap_year(year);
        let months = [Jan, Feb, Mar, Apr, May, Jun, Jul, Aug, Sep, Oct, Nov, Dec];
        let mut dates = [
            0..Jan.len(),
            0..Feb.leap_len(leap_year),
            0..Mar.len(),
            0..Apr.len(),
            0..May.len(),
            0..Jun.len(),
            0..Jul.len(),
            0..Aug.len(),
            0..Sep.len(),
            0..Oct.len(),
            0..Nov.len(),
            0..Dec.len(),
        ];
        if self.months_per_row > cols as i8 / 8 / 3 || self.months_per_row <= 0 {
            self.months_per_row = cols as i8 / 8 / 3;
        }
        let chunks = dates.chunks_mut(self.months_per_row as usize).zip(months.chunks(self.months_per_row as usize));
        let mut y = area.y;
        for (days_chunk, months) in chunks {
            let mut x = area.x;
            for month in months {
                buf.set_string(x as u16, y, month.to_string(), Style::default().add_modifier(Modifier::DIM));
                x = x + month.to_string().len() as u16 + 2;
            }
            y += 1;
            let mut x = area.x;
            for month in months {
                buf.set_string(x as u16, y, " Su Mo Tu We Th Fr Sa", Style::default());
                x += 21 + 1;
            }
            y += 1;
            let mut x = area.x;
            for (days, mon) in days_chunk.iter_mut().zip(months.iter()) {
                let first_day = mon.first_day(year) as u8;
                for _ in 0..(first_day) {
                    buf.set_string(x as u16, y, "", Style::default());
                    x += 3;
                }
                for _ in 0..(7 - first_day) {
                    buf.set_string(x as u16, y, format!("{:>3}", days.next().unwrap() + 1), Style::default());
                    x += 3;
                }
                buf.set_string(x as u16, y, " ", Style::default());
                x += 1;
            }
            y += 1;
            while !days_chunk.iter().all(|r| r.start == r.end) {
                let mut x = area.x;
                for days in days_chunk.iter_mut() {
                    for _ in 0..7 {
                        match days.next() {
                            Some(s) => {
                                buf.set_string(x as u16, y, format!("{:>3}", s + 1), Style::default());
                                x += 3;
                            },
                            None => x += 3,
                        }
                    }
                    x += 1;
                }
                y += 1;
            }
            y += 1;
        }
    }
}

impl<'a> fmt::Display for Calendar<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let today = Local::today();
        let cols = f.width().unwrap_or(3);
        let year = self.year;
        let leap_year = is_leap_year(year);
        let months = [Jan, Feb, Mar, Apr, May, Jun, Jul, Aug, Sep, Oct, Nov, Dec];
        let mut dates = [
            0..Jan.len(),
            0..Feb.leap_len(leap_year),
            0..Mar.len(),
            0..Apr.len(),
            0..May.len(),
            0..Jun.len(),
            0..Jul.len(),
            0..Aug.len(),
            0..Sep.len(),
            0..Oct.len(),
            0..Nov.len(),
            0..Dec.len(),
        ];
        let chunks = dates.chunks_mut(cols).zip(months.chunks(cols));
        for (days_chunk, months) in chunks {
            for month in months {
                write!(f, "{:>1$} ", month, COL_WIDTH)?;
            }
            f.write_str("\n")?;
            for month in months {
                write!(f, "{:>1$} ", " S  M  T  W  T  F  S", COL_WIDTH)?;
            }
            f.write_str("\n")?;
            for (days, mon) in days_chunk.iter_mut().zip(months.iter()) {
                let first_day = mon.first_day(year) as u8;
                for _ in 0..(first_day) {
                    f.write_str("   ")?;
                }
                for _ in 0..(7 - first_day) {
                    write!(f, "{:>3}", days.next().unwrap() + 1)?;
                }
                f.write_str(" ")?;
            }
            f.write_str("\n")?;
            while !days_chunk.iter().all(|r| r.start == r.end) {
                for days in days_chunk.iter_mut() {
                    for _ in 0..7 {
                        match days.next() {
                            Some(s) => write!(f, "{:>3}", s + 1)?,
                            None => f.write_str("   ")?,
                        }
                    }
                    f.write_str(" ")?;
                }
                f.write_str("\n")?;
            }
            f.write_str("\n")?;
        }
        Ok(())
    }
}
