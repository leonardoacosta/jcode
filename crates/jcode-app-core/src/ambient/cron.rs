//! Cron expressions evaluated by local calendar time in an IANA zone.
//!
//! Pure functions only: no clock reads, no persistence. Preview and dispatch
//! both call [`next_occurrences`], so they cannot disagree.
//!
//! Grammar (five fields, `minute hour day-of-month month day-of-week`):
//! `*`, `N`, `A-B`, `A,B,...`, `*/S`, `A-B/S`, `N/S`. Month and weekday names
//! are not supported. Weekday is `0-6` (Sunday = 0) and `7` also means Sunday.
//! When both day-of-month and day-of-week are restricted a day matches if
//! either field matches, as in Vixie cron. `@aliases` and seconds are rejected.
//!
//! Zone resolution: a local wall time that does not exist (spring-forward gap)
//! is skipped, never shifted; an ambiguous one (fall-back overlap) resolves to
//! its earlier UTC instant once. The occurrence key is derived from the local
//! calendar timestamp and the zone name, so it is stable across restarts, clock
//! movement and tz-database updates.

use anyhow::{Result, bail};
use chrono::{DateTime, Datelike, LocalResult, NaiveDate, NaiveDateTime, TimeZone, Timelike, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Upper bound on a preview or a single missed-run evaluation, so a bad
/// schedule can never make one pass unbounded.
pub const MAX_OCCURRENCES_PER_PASS: usize = 1_000;
/// Occurrences are searched at most this many days ahead of a start instant.
/// A valid expression such as `0 0 29 2 *` needs up to eight years across the
/// century leap-year gap.
const SEARCH_HORIZON_DAYS: i64 = 366 * 9;

/// A parsed five-field cron expression.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CronExpr {
    source: String,
    minutes: BTreeSet<u32>,
    hours: BTreeSet<u32>,
    days_of_month: BTreeSet<u32>,
    months: BTreeSet<u32>,
    days_of_week: BTreeSet<u32>,
    dom_restricted: bool,
    dow_restricted: bool,
}

/// One resolved occurrence: its stable key, the local timestamp it was derived
/// from, and the UTC instant it runs at.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Occurrence {
    pub local: NaiveDateTime,
    pub at: DateTime<Utc>,
}

fn parse_field(
    field: &str,
    name: &str,
    min: u32,
    max: u32,
    normalize: impl Fn(u32) -> u32,
) -> Result<(BTreeSet<u32>, bool)> {
    let mut values = BTreeSet::new();
    let mut restricted = field != "*";
    for part in field.split(',') {
        if part.is_empty() {
            bail!("cron {name} field has an empty list item");
        }
        let (range, step) = match part.split_once('/') {
            Some((range, step)) => {
                let step: u32 = step
                    .parse()
                    .map_err(|_| anyhow::anyhow!("cron {name} step '{step}' is not a number"))?;
                if step == 0 {
                    bail!("cron {name} step must be positive");
                }
                (range, step)
            }
            None => (part, 1),
        };
        let (lo, hi) = if range == "*" {
            if step > 1 {
                restricted = true;
            }
            (min, max)
        } else if let Some((lo, hi)) = range.split_once('-') {
            (num(lo, name)?, num(hi, name)?)
        } else {
            let value = num(range, name)?;
            // `N/S` runs from N to the field maximum.
            (value, if part.contains('/') { max } else { value })
        };
        // Weekday 7 is Sunday; accept it as a bound before normalising.
        let top = if name == "day-of-week" { 7 } else { max };
        if lo < min || hi > top || lo > hi {
            bail!("cron {name} range {lo}-{hi} is outside {min}-{top}");
        }
        let mut value = lo;
        while value <= hi {
            values.insert(normalize(value));
            match value.checked_add(step) {
                Some(next) => value = next,
                None => break,
            }
        }
    }
    if values.is_empty() {
        bail!("cron {name} field matches nothing");
    }
    // A field that lists every value is not a restriction.
    if values.len() as u32 == max - min + 1 {
        restricted = false;
    }
    Ok((values, restricted))
}

fn num(text: &str, name: &str) -> Result<u32> {
    if text.is_empty() || !text.bytes().all(|b| b.is_ascii_digit()) {
        bail!("cron {name} value '{text}' is not a number (names and symbols are unsupported)");
    }
    text.parse()
        .map_err(|_| anyhow::anyhow!("cron {name} value '{text}' is out of range"))
}

impl CronExpr {
    pub fn parse(source: &str) -> Result<Self> {
        let source = source.trim();
        if source.starts_with('@') {
            bail!("cron @aliases are unsupported; use five explicit fields");
        }
        let fields: Vec<&str> = source.split_whitespace().collect();
        if fields.len() != 5 {
            bail!(
                "cron expression needs exactly 5 fields (minute hour day-of-month month day-of-week), got {}",
                fields.len()
            );
        }
        let (minutes, _) = parse_field(fields[0], "minute", 0, 59, |v| v)?;
        let (hours, _) = parse_field(fields[1], "hour", 0, 23, |v| v)?;
        let (days_of_month, dom_restricted) = parse_field(fields[2], "day-of-month", 1, 31, |v| v)?;
        let (months, _) = parse_field(fields[3], "month", 1, 12, |v| v)?;
        let (days_of_week, dow_restricted) =
            parse_field(fields[4], "day-of-week", 0, 6, |v| v % 7)?;
        Ok(Self {
            source: fields.join(" "),
            minutes,
            hours,
            days_of_month,
            months,
            days_of_week,
            dom_restricted,
            dow_restricted,
        })
    }

    pub fn as_str(&self) -> &str {
        &self.source
    }

    fn day_matches(&self, date: NaiveDate) -> bool {
        if !self.months.contains(&date.month()) {
            return false;
        }
        let dom = self.days_of_month.contains(&date.day());
        let dow = self
            .days_of_week
            .contains(&date.weekday().num_days_from_sunday());
        match (self.dom_restricted, self.dow_restricted) {
            (true, true) => dom || dow,
            (true, false) => dom,
            (false, true) => dow,
            (false, false) => true,
        }
    }
}

/// Parse an IANA zone name. Only canonical tz database names are accepted, so
/// `UTC+2` style offsets and host-local zones are rejected.
pub fn parse_zone(name: &str) -> Result<Tz> {
    name.trim()
        .parse::<Tz>()
        .map_err(|_| anyhow::anyhow!("unknown IANA time zone '{name}'"))
}

/// Stable idempotency key: schedule id, resolved local calendar timestamp and
/// zone. Never derived from the UTC instant, which moves with zone rules.
pub fn occurrence_key(schedule_id: &str, local: NaiveDateTime, zone: &str) -> String {
    format!(
        "cron:{schedule_id}:{}:{zone}",
        local.format("%Y-%m-%dT%H:%M")
    )
}

/// Resolve a local wall time: `None` for a gap, the earlier instant for an
/// overlap.
fn resolve(zone: Tz, local: NaiveDateTime) -> Option<DateTime<Utc>> {
    match zone.from_local_datetime(&local) {
        LocalResult::None => None,
        LocalResult::Single(at) => Some(at.with_timezone(&Utc)),
        LocalResult::Ambiguous(earlier, later) => Some(earlier.min(later).with_timezone(&Utc)),
    }
}

/// Up to `limit` occurrences strictly after `after`, in time order, never
/// looking past `until` when given. Pure: the same inputs always give the same
/// answer, which is what lets preview and dispatch share it.
pub fn next_occurrences(
    expr: &CronExpr,
    zone: Tz,
    after: DateTime<Utc>,
    until: Option<DateTime<Utc>>,
    limit: usize,
) -> Vec<Occurrence> {
    let limit = limit.min(MAX_OCCURRENCES_PER_PASS);
    let mut out: Vec<Occurrence> = Vec::new();
    if limit == 0 {
        return out;
    }
    // Local dates are walked from a day before `after`'s local date: zone
    // offsets can put an instant on either side of a UTC date boundary.
    let start_local = after.with_timezone(&zone).date_naive();
    let mut date = start_local.pred_opt().unwrap_or(start_local);
    let Some(horizon) = start_local.checked_add_signed(chrono::Duration::days(SEARCH_HORIZON_DAYS))
    else {
        return out; // a corrupt far-future start yields no occurrences, never a panic
    };
    while date <= horizon && out.len() < limit {
        if expr.day_matches(date) {
            for hour in &expr.hours {
                for minute in &expr.minutes {
                    let Some(naive) = date.and_hms_opt(*hour, *minute, 0) else {
                        continue;
                    };
                    let Some(at) = resolve(zone, naive) else {
                        continue; // spring-forward gap: skipped, never shifted
                    };
                    if at <= after || until.is_some_and(|until| at > until) {
                        continue;
                    }
                    out.push(Occurrence { local: naive, at });
                }
            }
        }
        date = match date.succ_opt() {
            Some(next) => next,
            None => break,
        };
        // Local order within a day is hour/minute order, but an overlap can
        // map a later local time to an earlier instant, so sort at the end.
        if let Some(until) = until
            && date.and_hms_opt(0, 0, 0).is_some_and(|midnight| {
                resolve(zone, midnight).is_some_and(|at| {
                    until
                        .checked_add_signed(chrono::Duration::days(1))
                        .is_none_or(|limit| at > limit)
                })
            })
        {
            break;
        }
    }
    out.sort_by_key(|occurrence| occurrence.at);
    out.dedup_by_key(|occurrence| occurrence.at);
    out.truncate(limit);
    out
}

/// Hour and minute of an occurrence's local time, for tests and display.
pub fn local_hm(occurrence: &Occurrence) -> (u32, u32) {
    (occurrence.local.hour(), occurrence.local.minute())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn utc(text: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(text)
            .unwrap()
            .with_timezone(&Utc)
    }

    fn times(expr: &str, zone: &str, after: &str, n: usize) -> Vec<String> {
        next_occurrences(
            &CronExpr::parse(expr).unwrap(),
            parse_zone(zone).unwrap(),
            utc(after),
            None,
            n,
        )
        .into_iter()
        .map(|o| o.at.to_rfc3339())
        .collect()
    }

    #[test]
    fn cron_occurrence_resolves_dst_gap_and_overlap() {
        // US spring forward, 2026-03-08: 02:30 America/New_York does not exist.
        let gap = times("30 2 * * *", "America/New_York", "2026-03-06T12:00:00Z", 4);
        assert_eq!(
            gap,
            vec![
                "2026-03-07T07:30:00+00:00".to_string(), // 02:30 EST
                // 2026-03-08 02:30 is skipped, not shifted to 03:30 or 01:30
                "2026-03-09T06:30:00+00:00".to_string(), // 02:30 EDT
                "2026-03-10T06:30:00+00:00".to_string(),
                "2026-03-11T06:30:00+00:00".to_string(),
            ],
        );

        // US fall back, 2026-11-01: 01:30 happens twice. Only the earlier
        // instant (EDT, 05:30Z) runs; the second (EST, 06:30Z) does not.
        let overlap = times("30 1 * * *", "America/New_York", "2026-10-31T12:00:00Z", 3);
        assert_eq!(
            overlap,
            vec![
                "2026-11-01T05:30:00+00:00".to_string(),
                "2026-11-02T06:30:00+00:00".to_string(),
                "2026-11-03T06:30:00+00:00".to_string(),
            ],
        );

        // A non-DST zone is a fixed offset all year.
        let flat = times("0 9 * * *", "Asia/Kolkata", "2026-03-07T00:00:00Z", 2);
        assert_eq!(
            flat,
            vec![
                "2026-03-07T03:30:00+00:00".to_string(),
                "2026-03-08T03:30:00+00:00".to_string()
            ]
        );
        // The southern hemisphere gap in October is handled by the same rule.
        let south = times("30 2 * * *", "Australia/Sydney", "2026-10-03T00:00:00Z", 3);
        // 2026-10-04 02:30 is skipped; the next run is 10-05 02:30 AEDT.
        assert_eq!(south[0], "2026-10-04T15:30:00+00:00");
    }

    #[test]
    fn keys_use_local_time_and_zone_not_the_utc_instant() {
        let expr = CronExpr::parse("30 1 * * *").unwrap();
        let zone = parse_zone("America/New_York").unwrap();
        let occ = next_occurrences(&expr, zone, utc("2026-10-31T12:00:00Z"), None, 1);
        let key = occurrence_key("sched_a", occ[0].local, "America/New_York");
        assert_eq!(key, "cron:sched_a:2026-11-01T01:30:America/New_York");
        // The second, later rendering of 01:30 never yields a second key.
        let again = next_occurrences(&expr, zone, occ[0].at, None, 1);
        assert_ne!(
            occurrence_key("sched_a", again[0].local, "America/New_York"),
            key
        );
    }

    #[test]
    fn grammar_accepts_the_documented_forms_and_rejects_the_rest() {
        for ok in [
            "* * * * *",
            "*/15 * * * *",
            "0 9-17/2 * * 1-5",
            "0,30 8 1,15 * *",
            "0 0 * * 7",
            "5/20 * * * *",
        ] {
            CronExpr::parse(ok).unwrap_or_else(|e| panic!("{ok}: {e}"));
        }
        for bad in [
            "",
            "* * * *",
            "* * * * * *",
            "60 * * * *",
            "* 24 * * *",
            "* * 0 * *",
            "* * * 13 *",
            "* * * * 8",
            "*/0 * * * *",
            "@daily",
            "a * * * *",
            "* * * jan *",
            "1,,2 * * * *",
            "5-1 * * * *",
        ] {
            assert!(CronExpr::parse(bad).is_err(), "{bad:?} must be rejected");
        }
        assert!(parse_zone("America/New_York").is_ok());
        for bad in ["", "Nowhere/City", "UTC+2", "EST5EDTX"] {
            assert!(parse_zone(bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn day_of_month_and_weekday_combine_like_vixie_cron() {
        // 1st of the month OR any Monday.
        let got = times("0 0 1 * 1", "UTC", "2026-06-01T00:00:00Z", 4);
        assert_eq!(
            got,
            vec![
                "2026-06-08T00:00:00+00:00".to_string(),
                "2026-06-15T00:00:00+00:00".to_string(),
                "2026-06-22T00:00:00+00:00".to_string(),
                "2026-06-29T00:00:00+00:00".to_string(),
            ]
        );
        // Sunday is 0 or 7.
        let sun0 = times("0 0 * * 0", "UTC", "2026-06-01T00:00:00Z", 1);
        let sun7 = times("0 0 * * 7", "UTC", "2026-06-01T00:00:00Z", 1);
        assert_eq!(sun0, sun7);
        assert_eq!(sun0, vec!["2026-06-07T00:00:00+00:00".to_string()]);
    }

    #[test]
    fn occurrences_are_strictly_after_bounded_and_deterministic() {
        let expr = CronExpr::parse("*/10 * * * *").unwrap();
        let zone = parse_zone("UTC").unwrap();
        let after = utc("2026-01-01T00:10:00Z");
        let first = next_occurrences(&expr, zone, after, None, 3);
        assert_eq!(first[0].at, utc("2026-01-01T00:20:00Z"), "strictly after");
        assert_eq!(first, next_occurrences(&expr, zone, after, None, 3));
        let bounded = next_occurrences(&expr, zone, after, Some(utc("2026-01-01T00:40:00Z")), 100);
        assert_eq!(bounded.len(), 3);
        assert!(next_occurrences(&expr, zone, after, None, 0).is_empty());
        assert_eq!(
            next_occurrences(&expr, zone, after, None, usize::MAX).len(),
            MAX_OCCURRENCES_PER_PASS
        );
        // A leap-day schedule still resolves across the horizon.
        let leap = CronExpr::parse("0 0 29 2 *").unwrap();
        let next = next_occurrences(&leap, zone, utc("2026-03-01T00:00:00Z"), None, 1);
        assert_eq!(next[0].at, utc("2028-02-29T00:00:00Z"));
    }
}
