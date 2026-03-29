use chrono::{DateTime, Local, NaiveDateTime, TimeZone};

pub fn local_from_utc(utc: &NaiveDateTime) -> DateTime<Local> {
  Local.from_utc_datetime(utc)
}

pub fn format_local_date_time(utc: &NaiveDateTime) -> String {
  local_from_utc(utc).format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn format_local_date(utc: &NaiveDateTime, format: &str) -> String {
  local_from_utc(utc).format(format).to_string()
}

pub fn format_taskwarrior_datetime_literal(utc: &NaiveDateTime) -> String {
  format_taskwarrior_datetime(local_from_utc(utc))
}

fn format_taskwarrior_datetime<Tz: TimeZone>(dt: DateTime<Tz>) -> String
where
  Tz::Offset: std::fmt::Display,
{
  dt.format("'%Y-%m-%dT%H:%M:%S%:z'").to_string()
}

#[cfg(test)]
mod tests {
  use chrono::{DateTime, FixedOffset, NaiveDate};

  use super::format_taskwarrior_datetime;

  fn fixed_offset_from_utc(offset_seconds: i32, year: i32, month: u32, day: u32, hour: u32, minute: u32, second: u32) -> DateTime<FixedOffset> {
    let utc = NaiveDate::from_ymd_opt(year, month, day)
      .unwrap()
      .and_hms_opt(hour, minute, second)
      .unwrap();
    DateTime::from_naive_utc_and_offset(utc, FixedOffset::east_opt(offset_seconds).unwrap())
  }

  #[test]
  fn formats_exact_one_hour_offsets() {
    let plus_one = fixed_offset_from_utc(60 * 60, 2026, 3, 29, 7, 15, 0);
    let minus_one = fixed_offset_from_utc(-(60 * 60), 2026, 3, 29, 7, 15, 0);

    assert_eq!(format_taskwarrior_datetime(plus_one), "'2026-03-29T08:15:00+01:00'");
    assert_eq!(format_taskwarrior_datetime(minus_one), "'2026-03-29T06:15:00-01:00'");
  }

  #[test]
  fn formats_offsets_across_dst_boundaries() {
    let before_spring_forward = fixed_offset_from_utc(-(5 * 60 * 60), 2024, 3, 10, 6, 59, 59);
    let after_spring_forward = fixed_offset_from_utc(-(4 * 60 * 60), 2024, 3, 10, 7, 0, 0);
    let before_fall_back = fixed_offset_from_utc(-(4 * 60 * 60), 2024, 11, 3, 5, 59, 59);
    let after_fall_back = fixed_offset_from_utc(-(5 * 60 * 60), 2024, 11, 3, 6, 0, 0);

    assert_eq!(format_taskwarrior_datetime(before_spring_forward), "'2024-03-10T01:59:59-05:00'");
    assert_eq!(format_taskwarrior_datetime(after_spring_forward), "'2024-03-10T03:00:00-04:00'");
    assert_eq!(format_taskwarrior_datetime(before_fall_back), "'2024-11-03T01:59:59-04:00'");
    assert_eq!(format_taskwarrior_datetime(after_fall_back), "'2024-11-03T01:00:00-05:00'");
  }

  #[test]
  fn round_trips_taskwarrior_datetime_literals() {
    let original = fixed_offset_from_utc(90 * 60, 2026, 3, 29, 7, 15, 0);
    let formatted = format_taskwarrior_datetime(original);
    let parsed = DateTime::parse_from_rfc3339(formatted.trim_matches('\'')).unwrap();

    assert_eq!(parsed, original);
  }
}
