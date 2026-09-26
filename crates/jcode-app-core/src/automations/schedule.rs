use chrono::{DateTime, Datelike, Days, Duration, NaiveTime, TimeZone, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Schedule {
    Interval {
        seconds: u64,
    },
    Calendar {
        weekdays: Vec<u32>,
        time: String,
        timezone: String,
    },
}

impl Schedule {
    pub fn validate(&self) -> anyhow::Result<()> {
        match self {
            Self::Interval { seconds } if *seconds < 60 => {
                anyhow::bail!("interval must be at least 60 seconds")
            }
            Self::Interval { .. } => Ok(()),
            Self::Calendar {
                weekdays,
                time,
                timezone,
            } => {
                if weekdays.is_empty() || weekdays.iter().any(|d| *d > 6) {
                    anyhow::bail!("weekdays must contain values 0..=6")
                }
                if weekdays
                    .iter()
                    .copied()
                    .collect::<std::collections::HashSet<_>>()
                    .len()
                    != weekdays.len()
                {
                    anyhow::bail!("weekdays must be unique")
                }
                if NaiveTime::parse_from_str(time, "%H:%M").is_err() {
                    anyhow::bail!("time must use HH:MM")
                }
                Tz::from_str(timezone)
                    .map_err(|_| anyhow::anyhow!("unknown timezone: {timezone}"))?;
                Ok(())
            }
        }
    }

    pub fn next_after(&self, after: DateTime<Utc>) -> anyhow::Result<DateTime<Utc>> {
        self.validate()?;
        match self {
            Self::Interval { seconds } => after
                .checked_add_signed(
                    Duration::try_seconds(i64::try_from(*seconds)?)
                        .ok_or_else(|| anyhow::anyhow!("interval duration overflow"))?,
                )
                .ok_or_else(|| anyhow::anyhow!("interval timestamp overflow")),
            Self::Calendar {
                weekdays,
                time,
                timezone,
            } => {
                let zone = Tz::from_str(timezone)
                    .map_err(|_| anyhow::anyhow!("unknown timezone: {timezone}"))?;
                let time = NaiveTime::parse_from_str(time, "%H:%M")?;
                let local = after.with_timezone(&zone);
                for offset in 0..=370 {
                    let date = local
                        .date_naive()
                        .checked_add_days(Days::new(offset))
                        .ok_or_else(|| anyhow::anyhow!("calendar date overflow"))?;
                    if !weekdays.contains(&date.weekday().num_days_from_monday()) {
                        continue;
                    }
                    let naive = date.and_time(time);
                    let candidate = match zone.from_local_datetime(&naive) {
                        chrono::LocalResult::Single(dt) => dt,
                        chrono::LocalResult::Ambiguous(a, b) => a.min(b),
                        chrono::LocalResult::None => continue,
                    }
                    .with_timezone(&Utc);
                    if candidate > after {
                        return Ok(candidate);
                    }
                }
                anyhow::bail!("no calendar occurrence found within one year")
            }
        }
    }

    pub fn next_from(
        &self,
        anchor: DateTime<Utc>,
        now: DateTime<Utc>,
    ) -> anyhow::Result<DateTime<Utc>> {
        let mut next = self.next_after(anchor)?;
        if next > now {
            return Ok(next);
        }
        match self {
            Self::Interval { seconds } => {
                let elapsed = (now - anchor).num_seconds().max(0) as u64;
                let ticks = elapsed / seconds + 1;
                let delta = seconds
                    .checked_mul(ticks)
                    .and_then(|s| i64::try_from(s).ok())
                    .ok_or_else(|| anyhow::anyhow!("interval timestamp overflow"))?;
                next = anchor
                    .checked_add_signed(
                        Duration::try_seconds(delta)
                            .ok_or_else(|| anyhow::anyhow!("interval duration overflow"))?,
                    )
                    .ok_or_else(|| anyhow::anyhow!("interval timestamp overflow"))?;
            }
            Self::Calendar { .. } => next = self.next_after(now)?,
        }
        Ok(next)
    }

    pub fn preview(
        &self,
        after: DateTime<Utc>,
        count: usize,
    ) -> anyhow::Result<Vec<DateTime<Utc>>> {
        let mut cursor = after;
        let mut dates = Vec::with_capacity(count);
        for _ in 0..count {
            cursor = self.next_after(cursor)?;
            dates.push(cursor);
        }
        Ok(dates)
    }
}
