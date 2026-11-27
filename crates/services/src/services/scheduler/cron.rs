//! A small five-field cron parser (minute hour day-of-month month weekday).
//!
//! Each field is stored as a bit set, so matching a time is a few bit tests
//! and finding the next run walks days, then hours, then minutes instead of
//! stepping one minute at a time. Supports `*`, lists, ranges, steps
//! (`*/15`, `9-17/2`), month and weekday names, `7` for Sunday, and the
//! `@hourly`/`@daily`/`@weekly`/`@monthly`/`@yearly` shortcuts. When both
//! day-of-month and weekday are restricted, a day matches if either does,
//! as in classic cron.

use chrono::{DateTime, Datelike, Duration, LocalResult, NaiveDate, TimeZone};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CronSpec {
    minutes: u64,
    hours: u32,
    days: u32,
    months: u16,
    weekdays: u8,
    days_restricted: bool,
    weekdays_restricted: bool,
}

const MONTHS: [&str; 12] = [
    "jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec",
];
const WEEKDAYS: [&str; 7] = ["sun", "mon", "tue", "wed", "thu", "fri", "sat"];
/// How far ahead to look for the next run (covers Feb 29 and rare weekday matches).
const SEARCH_DAYS: i64 = 366 * 8;

fn value(token: &str, names: &[&str], offset: u32) -> Result<u32, String> {
    let lower = token.to_ascii_lowercase();
    if let Some(index) = names.iter().position(|name| *name == lower) {
        return Ok(index as u32 + offset);
    }
    token
        .parse::<u32>()
        .map_err(|_| format!("`{token}` is not a number"))
}

fn field(
    text: &str,
    min: u32,
    max: u32,
    names: &[&str],
    offset: u32,
) -> Result<(u64, bool), String> {
    let mut bits = 0u64;
    let restricted = text != "*" && text != "?";
    for part in text.split(',') {
        let (range, step) = match part.split_once('/') {
            Some((range, step)) => {
                let step: u32 = step.parse().map_err(|_| format!("bad step in `{part}`"))?;
                if step == 0 {
                    return Err(format!("step cannot be 0 in `{part}`"));
                }
                (range, step)
            }
            None => (part, 1),
        };
        let (start, end) = if range == "*" || range == "?" {
            (min, max)
        } else if let Some((a, b)) = range.split_once('-') {
            (value(a, names, offset)?, value(b, names, offset)?)
        } else {
            let single = value(range, names, offset)?;
            // `5/10` means "from 5 to the end, every 10".
            (single, if part.contains('/') { max } else { single })
        };
        if start < min || end > max || start > end {
            return Err(format!("`{part}` is outside {min}-{max}"));
        }
        let mut current = start;
        while current <= end {
            bits |= 1 << current;
            current += step;
        }
    }
    Ok((bits, restricted))
}

impl CronSpec {
    pub fn parse(expression: &str) -> Result<Self, String> {
        let expanded = match expression.trim() {
            "@hourly" => "0 * * * *",
            "@daily" | "@midnight" => "0 0 * * *",
            "@weekly" => "0 0 * * 0",
            "@monthly" => "0 0 1 * *",
            "@yearly" | "@annually" => "0 0 1 1 *",
            other => other,
        };
        let fields: Vec<&str> = expanded.split_whitespace().collect();
        let [minute, hour, day, month, weekday] = fields.as_slice() else {
            return Err("a schedule needs five fields: minute hour day month weekday".into());
        };
        let (minutes, _) = field(minute, 0, 59, &[], 0)?;
        let (hours, _) = field(hour, 0, 23, &[], 0)?;
        let (days, days_restricted) = field(day, 1, 31, &[], 0)?;
        let (months, _) = field(month, 1, 12, &MONTHS, 1)?;
        let (mut weekdays, weekdays_restricted) = field(weekday, 0, 7, &WEEKDAYS, 0)?;
        if weekdays & (1 << 7) != 0 {
            weekdays = (weekdays | 1) & 0x7f; // 7 is Sunday too
        }
        Ok(Self {
            minutes,
            hours: hours as u32,
            days: days as u32,
            months: months as u16,
            weekdays: weekdays as u8,
            days_restricted,
            weekdays_restricted,
        })
    }

    fn day_matches(&self, date: NaiveDate) -> bool {
        if self.months & (1 << date.month()) == 0 {
            return false;
        }
        let by_day = self.days & (1 << date.day()) != 0;
        let by_weekday = self.weekdays & (1 << date.weekday().num_days_from_sunday()) != 0;
        match (self.days_restricted, self.weekdays_restricted) {
            (true, true) => by_day || by_weekday,
            (true, false) => by_day,
            (false, true) => by_weekday,
            (false, false) => true,
        }
    }

    /// The first matching time strictly after `after`, in `after`'s time zone.
    /// Local times skipped by a daylight-saving jump are skipped; a repeated
    /// hour runs once, at its first occurrence.
    pub fn next_after<Tz: TimeZone>(&self, after: &DateTime<Tz>) -> Option<DateTime<Tz>> {
        let zone = after.timezone();
        let start = after.naive_local();
        let mut date = start.date();
        for _ in 0..SEARCH_DAYS {
            if self.day_matches(date) {
                for hour in (0..24).filter(|h| self.hours & (1 << h) != 0) {
                    for minute in (0..60).filter(|m| self.minutes & (1 << m) != 0) {
                        let naive = date.and_hms_opt(hour, minute, 0)?;
                        if naive <= start {
                            continue;
                        }
                        match zone.from_local_datetime(&naive) {
                            LocalResult::Single(time) => return Some(time),
                            LocalResult::Ambiguous(first, _) => return Some(first),
                            LocalResult::None => continue,
                        }
                    }
                }
            }
            date = date.checked_add_signed(Duration::days(1))?;
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use chrono::{FixedOffset, Utc};

    use super::*;

    fn at(text: &str) -> DateTime<FixedOffset> {
        DateTime::parse_from_rfc3339(text).unwrap()
    }

    fn next(expr: &str, after: &str) -> String {
        CronSpec::parse(expr)
            .unwrap()
            .next_after(&at(after))
            .unwrap()
            .to_rfc3339()
    }

    #[test]
    fn finds_the_next_run() {
        assert_eq!(
            next("*/15 * * * *", "2026-11-27T10:07:30+01:00"),
            "2026-11-27T10:15:00+01:00"
        );
        assert_eq!(
            next("0 9 * * *", "2026-11-27T09:00:00+01:00"),
            "2026-11-28T09:00:00+01:00"
        );
        assert_eq!(
            next("30 8 * * mon-fri", "2026-11-27T09:00:00+01:00"),
            "2026-11-30T08:30:00+01:00"
        );
        assert_eq!(
            next("0 0 29 feb *", "2026-03-01T00:00:00+00:00"),
            "2028-02-29T00:00:00+00:00"
        );
        assert_eq!(
            next("@monthly", "2026-12-15T12:00:00+00:00"),
            "2027-01-01T00:00:00+00:00"
        );
        assert_eq!(
            next("0 12 * * 7", "2026-11-27T00:00:00+00:00"),
            "2026-11-29T12:00:00+00:00"
        );
        assert_eq!(
            next("0 9-17/4 * * *", "2026-11-27T10:00:00+00:00"),
            "2026-11-27T13:00:00+00:00"
        );
    }

    #[test]
    fn day_of_month_and_weekday_match_either() {
        // The 1st of the month OR any Monday.
        assert_eq!(
            next("0 0 1 * mon", "2026-11-27T00:00:00+00:00"),
            "2026-11-30T00:00:00+00:00"
        );
        assert_eq!(
            next("0 0 1 * mon", "2026-11-30T00:00:00+00:00"),
            "2026-12-01T00:00:00+00:00"
        );
    }

    #[test]
    fn rejects_bad_expressions() {
        for bad in [
            "* * * *",
            "60 * * * *",
            "* 24 * * *",
            "*/0 * * * *",
            "5-1 * * * *",
            "x * * * *",
        ] {
            assert!(CronSpec::parse(bad).is_err(), "{bad}");
        }
        assert!(
            CronSpec::parse("0 0 31 2 *")
                .unwrap()
                .next_after(&Utc::now())
                .is_none()
        );
    }
}
