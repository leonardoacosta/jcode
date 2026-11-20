//! Recurring (cron) schedules.
//!
//! Definitions and occurrence decisions live in `schedules.json`, a sidecar of
//! the one-shot `queue.json`. They cannot live inside the queue file: that file
//! is a bare JSON list that older binaries read, and a definition there would
//! be treated as a ready item. Work still reaches the runner only through the
//! existing queue, as ordinary items whose id is the occurrence key, so the
//! claim, acknowledgement, uncertain-delivery and history contract of
//! `ScheduledQueue` applies unchanged. [`tick`] is called only by the ambient
//! runner loop, which is the single scheduler owner.
//!
//! Lock order is always store, then queue. Decisions are written to the store
//! (`Due`, `Skipped`, `Coalesced`) before anything is pushed to the queue, and
//! a push is idempotent on the occurrence key, so a crash at any point either
//! replays the push or finds it already there.
//!
//! "Active run" is the queue delivery lifetime (`Pending` or `Dispatched`).
//! The scheduled path has no task-completion signal, so an acknowledged
//! delivery releases its slot; it is not evidence that the task finished.

use super::cron::{self, CronExpr, MAX_OCCURRENCES_PER_PASS};
use super::{ItemState, ScheduleRequest, ScheduledQueue};
use crate::storage;
use anyhow::{Result, bail};
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// The most recent occurrence is on time if it is at most this old; anything
/// older, and every earlier occurrence of the same pass, is "missed".
const ON_TIME_GRACE_MINUTES: i64 = 5;
/// Occurrence records kept per schedule. The evaluation boundary, not these
/// records, is what stops older occurrences being recreated.
const MAX_RECENT_OCCURRENCES: usize = 100;
/// Usage events remembered for replay deduplication.
const MAX_USAGE_EVENTS: usize = 2_000;
pub const DEFAULT_GLOBAL_MAX_ACTIVE: u32 = 4;
pub const MAX_PREVIEW: usize = 50;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "policy", rename_all = "kebab-case")]
pub enum MissedPolicy {
    Skip,
    RunOnce,
    CatchUp { limit: u32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OccurrenceState {
    /// Decided to run; waiting for admission (limits, budget).
    Due,
    /// Pushed to the queue under the same key; the queue owns its fate now.
    Enqueued,
    Skipped,
    Coalesced,
    BudgetExhausted,
    BudgetUnavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OccurrenceRecord {
    pub key: String,
    pub local: NaiveDateTime,
    pub at: DateTime<Utc>,
    pub state: OccurrenceState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageEvent {
    pub event_id: String,
    pub key: String,
    pub tokens: u64,
    pub spend_usd: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schedule {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub cron: String,
    pub zone: String,
    pub missed: MissedPolicy,
    pub max_active: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_cap: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spend_cap_usd: Option<f64>,
    pub paused: bool,
    pub created_at: DateTime<Utc>,
    /// Occurrences at or before this instant have been decided. Never moves
    /// backward, so a backward clock jump cannot re-create occurrences.
    pub evaluated_through: DateTime<Utc>,
    pub template: ScheduleRequest,
    #[serde(default)]
    pub occurrences: Vec<OccurrenceRecord>,
    #[serde(default)]
    pub tokens_used: u64,
    #[serde(default)]
    pub spend_used_usd: f64,
    #[serde(default)]
    pub usage: Vec<UsageEvent>,
}

/// Parameters for a new schedule, already parsed from tool input.
pub struct NewSchedule {
    pub name: Option<String>,
    pub cron: String,
    pub zone: String,
    pub missed: MissedPolicy,
    pub max_active: u32,
    pub token_cap: Option<u64>,
    pub spend_cap_usd: Option<f64>,
    pub template: ScheduleRequest,
}

impl NewSchedule {
    /// Validate everything before any state is touched.
    pub fn validate(&self) -> Result<()> {
        let expr = CronExpr::parse(&self.cron)?;
        let zone = cron::parse_zone(&self.zone)?;
        if cron::next_occurrences(&expr, zone, Utc::now(), None, 1).is_empty() {
            bail!("cron expression '{}' never matches a real date", self.cron);
        }
        if self.max_active == 0 {
            bail!("max_active must be positive");
        }
        if let MissedPolicy::CatchUp { limit } = self.missed
            && !(1..=MAX_CATCH_UP_LIMIT).contains(&limit)
        {
            bail!("catch-up limit must be between 1 and {MAX_CATCH_UP_LIMIT}");
        }
        if self.token_cap == Some(0) {
            bail!("token_cap must be positive");
        }
        if let Some(cap) = self.spend_cap_usd
            && !(cap.is_finite() && cap > 0.0)
        {
            bail!("spend_cap_usd must be a positive number");
        }
        Ok(())
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct StoreFile {
    #[serde(default)]
    schedules: Vec<Schedule>,
}

pub struct RecurringStore {
    path: PathBuf,
}

impl RecurringStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn open_default() -> Result<Self> {
        Ok(Self::new(
            super::paths::ambient_dir()?.join("schedules.json"),
        ))
    }

    fn read(&self) -> Result<StoreFile> {
        if self.path.exists() {
            storage::read_json(&self.path)
        } else {
            Ok(StoreFile::default())
        }
    }

    /// Lock a stable sidecar inode, reload, mutate, write only if changed.
    fn transact<T>(&self, update: impl FnOnce(&mut Vec<Schedule>) -> Result<T>) -> Result<T> {
        if let Some(parent) = self.path.parent() {
            storage::ensure_dir(parent)?;
        }
        let lock = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(self.path.with_extension("json.lock"))?;
        lock.lock()?;
        let mut file = self.read()?;
        let before = serde_json::to_value(&file)?;
        let result = update(&mut file.schedules)?;
        if serde_json::to_value(&file)? != before {
            storage::write_json(&self.path, &file)?;
        }
        Ok(result)
    }

    pub fn list(&self) -> Result<Vec<Schedule>> {
        Ok(self.read()?.schedules)
    }

    pub fn get(&self, id: &str) -> Result<Option<Schedule>> {
        Ok(self.list()?.into_iter().find(|s| s.id == id))
    }

    /// Validate, then persist. Nothing is written on error.
    pub fn create(&self, new: NewSchedule, now: DateTime<Utc>) -> Result<Schedule> {
        new.validate()?;
        let schedule = Schedule {
            id: format!("cron_{:016x}", rand::random::<u64>()),
            name: new.name,
            cron: CronExpr::parse(&new.cron)?.as_str().to_string(),
            zone: cron::parse_zone(&new.zone)?.name().to_string(),
            missed: new.missed,
            max_active: new.max_active,
            token_cap: new.token_cap,
            spend_cap_usd: new.spend_cap_usd,
            paused: false,
            created_at: now,
            evaluated_through: now,
            template: new.template,
            occurrences: Vec::new(),
            tokens_used: 0,
            spend_used_usd: 0.0,
            usage: Vec::new(),
        };
        let stored = schedule.clone();
        self.transact(move |all| {
            all.push(schedule);
            Ok(())
        })?;
        Ok(stored)
    }

    pub fn set_paused(&self, id: &str, paused: bool) -> Result<Schedule> {
        self.transact(|all| {
            let s = all
                .iter_mut()
                .find(|s| s.id == id)
                .ok_or_else(|| anyhow::anyhow!("no recurring schedule '{id}'"))?;
            s.paused = paused;
            Ok(s.clone())
        })
    }

    /// Remove the definition. Returns its keys so the caller can cancel
    /// queue items that are still pending.
    pub fn remove(&self, id: &str) -> Result<Option<Schedule>> {
        self.transact(|all| {
            Ok(all
                .iter()
                .position(|s| s.id == id)
                .map(|index| all.remove(index)))
        })
    }

    /// Count a usage event once, by its stable event id. Returns `false` for a
    /// replay.
    pub fn record_usage(
        &self,
        id: &str,
        event_id: &str,
        key: &str,
        tokens: u64,
        spend_usd: f64,
    ) -> Result<bool> {
        self.transact(|all| {
            let s = all
                .iter_mut()
                .find(|s| s.id == id)
                .ok_or_else(|| anyhow::anyhow!("no recurring schedule '{id}'"))?;
            if s.usage.iter().any(|event| event.event_id == event_id) {
                return Ok(false);
            }
            s.tokens_used = s.tokens_used.saturating_add(tokens);
            s.spend_used_usd += spend_usd.max(0.0);
            s.usage.push(UsageEvent {
                event_id: event_id.to_string(),
                key: key.to_string(),
                tokens,
                spend_usd,
            });
            let excess = s.usage.len().saturating_sub(MAX_USAGE_EVENTS);
            s.usage.drain(..excess);
            Ok(true)
        })
    }
}

impl Schedule {
    pub fn expr(&self) -> Result<CronExpr> {
        CronExpr::parse(&self.cron)
    }

    pub fn tz(&self) -> Result<chrono_tz::Tz> {
        cron::parse_zone(&self.zone)
    }

    pub fn has_cap(&self) -> bool {
        self.token_cap.is_some() || self.spend_cap_usd.is_some()
    }

    /// Next occurrences after `after`. The one calculator used by preview and
    /// dispatch.
    pub fn upcoming(&self, after: DateTime<Utc>, n: usize) -> Vec<cron::Occurrence> {
        match (self.expr(), self.tz()) {
            (Ok(expr), Ok(zone)) => cron::next_occurrences(&expr, zone, after, None, n),
            _ => Vec::new(),
        }
    }
}

/// Whether a hard, authoritative budget enforcement primitive exists for a
/// schedule. No such primitive exists on the scheduled path today (usage is
/// only known after a request), so [`NoHardEnforcement`] is the production
/// source and any capped schedule fails closed as `budget_unavailable`.
pub trait BudgetSource {
    fn enforces(&self, schedule: &Schedule) -> bool;
}

pub struct NoHardEnforcement;

impl BudgetSource for NoHardEnforcement {
    fn enforces(&self, _schedule: &Schedule) -> bool {
        false
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct TickReport {
    pub enqueued: usize,
    /// Due occurrences still waiting for capacity.
    pub waiting: usize,
    pub next_due: Option<DateTime<Utc>>,
}

fn record_key(schedule: &Schedule, occ: &cron::Occurrence) -> String {
    cron::occurrence_key(&schedule.id, occ.local, &schedule.zone)
}

/// Batches of [`MAX_OCCURRENCES_PER_PASS`] walked for one decision pass. A
/// span past this (about two million occurrences, e.g. `* * * * *` after years
/// of downtime) is not enumerated: the rest is recorded as skipped in bulk.
const MAX_BATCHES_PER_DECISION: usize = 200;
/// Largest catch-up limit accepted, so waiting work stays bounded.
pub const MAX_CATCH_UP_LIMIT: u32 = 100;

/// Decide every newly due occurrence of an unpaused schedule under its
/// missed-run policy, over the whole span since the last evaluation, and
/// persist the decisions before anything becomes runnable.
///
/// Only the most recent occurrence can be on time (within the grace window);
/// every earlier one is "missed" and goes through the policy, so a short
/// outage cannot bypass `skip` or `run-once`.
fn decide(schedule: &mut Schedule, now: DateTime<Utc>) {
    if schedule.paused || now <= schedule.evaluated_through {
        return;
    }
    let (Ok(expr), Ok(zone)) = (schedule.expr(), schedule.tz()) else {
        return; // an unreadable definition decides nothing
    };
    let grace = chrono::Duration::minutes(ON_TIME_GRACE_MINUTES);
    let known: std::collections::HashSet<String> =
        schedule.occurrences.iter().map(|o| o.key.clone()).collect();
    // Streamed, with memory bounded: keep the first `limit` (catch-up), the
    // latest (run-once, on time), and a tail of settled decisions to record.
    let mut keep: Vec<(cron::Occurrence, OccurrenceState)> = Vec::new();
    let mut previous_latest: Option<cron::Occurrence> = None;
    let mut missed_index = 0usize;
    let mut from = schedule.evaluated_through;
    let mut through = now;
    'batches: for batch_index in 0..MAX_BATCHES_PER_DECISION {
        let batch = cron::next_occurrences(&expr, zone, from, Some(now), MAX_OCCURRENCES_PER_PASS);
        let full = batch.len() == MAX_OCCURRENCES_PER_PASS;
        if let Some(last) = batch.last() {
            from = last.at;
        }
        for occ in batch {
            // The previous occurrence is now known not to be the latest: it
            // is missed, so the policy applies to it.
            if let Some(prev) = previous_latest.replace(occ) {
                let state = match schedule.missed {
                    MissedPolicy::Skip => OccurrenceState::Skipped,
                    MissedPolicy::RunOnce => OccurrenceState::Coalesced,
                    MissedPolicy::CatchUp { limit } if missed_index < limit as usize => {
                        OccurrenceState::Due
                    }
                    MissedPolicy::CatchUp { .. } => OccurrenceState::Skipped,
                };
                missed_index += 1;
                keep.push((prev, state));
                bound_kept(&mut keep);
            }
        }
        if !full {
            break;
        }
        if batch_index + 1 == MAX_BATCHES_PER_DECISION {
            // Too long a span to enumerate: stop here and let the remainder
            // fall to the next pass's evaluation start.
            through = from;
            break 'batches;
        }
    }
    // The last occurrence is the only one that can be on time.
    if let Some(last) = previous_latest {
        let state = if now - last.at <= grace {
            OccurrenceState::Due
        } else {
            match schedule.missed {
                MissedPolicy::Skip => OccurrenceState::Skipped,
                MissedPolicy::RunOnce => OccurrenceState::Due,
                MissedPolicy::CatchUp { limit } if missed_index < limit as usize => {
                    OccurrenceState::Due
                }
                MissedPolicy::CatchUp { .. } => OccurrenceState::Skipped,
            }
        };
        keep.push((last, state));
    }
    for (occ, state) in keep {
        let key = record_key(schedule, &occ);
        if known.contains(&key) {
            continue;
        }
        schedule.occurrences.push(OccurrenceRecord {
            key,
            local: occ.local,
            at: occ.at,
            state,
        });
    }
    schedule.evaluated_through = schedule.evaluated_through.max(through);
    bound_waiting(schedule);
    prune_records(schedule);
}

/// Drop settled decisions beyond the record bound while streaming; keep every
/// `Due` one.
fn bound_kept(keep: &mut Vec<(cron::Occurrence, OccurrenceState)>) {
    let settled = keep
        .iter()
        .filter(|(_, state)| *state != OccurrenceState::Due)
        .count();
    if settled <= 2 * MAX_RECENT_OCCURRENCES {
        return;
    }
    let mut excess = settled - MAX_RECENT_OCCURRENCES;
    keep.retain(|(_, state)| {
        if *state != OccurrenceState::Due && excess > 0 {
            excess -= 1;
            false
        } else {
            true
        }
    });
}

/// Waiting (`Due`) occurrences are bounded: a schedule whose capacity stays
/// full must not grow its file forever or run stale work later. Older waiting
/// occurrences beyond the bound are settled by the policy: coalesced for
/// `run-once`, skipped otherwise. `catch-up` keeps up to its limit.
fn bound_waiting(schedule: &mut Schedule) {
    let keep = match schedule.missed {
        MissedPolicy::CatchUp { limit } => limit as usize,
        _ => 1,
    }
    .max(1);
    let waiting: Vec<usize> = schedule
        .occurrences
        .iter()
        .enumerate()
        .filter(|(_, o)| o.state == OccurrenceState::Due)
        .map(|(index, _)| index)
        .collect();
    let overflow = waiting.len().saturating_sub(keep);
    let settled = match schedule.missed {
        MissedPolicy::RunOnce => OccurrenceState::Coalesced,
        _ => OccurrenceState::Skipped,
    };
    for index in waiting.into_iter().take(overflow) {
        schedule.occurrences[index].state = settled;
    }
}

/// Keep every `Due` record and the most recent settled ones.
fn prune_records(schedule: &mut Schedule) {
    let settled = schedule
        .occurrences
        .iter()
        .filter(|o| o.state != OccurrenceState::Due)
        .count();
    let mut excess = settled.saturating_sub(MAX_RECENT_OCCURRENCES);
    schedule.occurrences.retain(|o| {
        if o.state != OccurrenceState::Due && excess > 0 {
            excess -= 1;
            false
        } else {
            true
        }
    });
}

fn budget_block(schedule: &Schedule, source: &dyn BudgetSource) -> Option<OccurrenceState> {
    if !schedule.has_cap() {
        return None;
    }
    if !source.enforces(schedule) {
        return Some(OccurrenceState::BudgetUnavailable);
    }
    let tokens_out = schedule
        .token_cap
        .is_some_and(|cap| schedule.tokens_used >= cap);
    let spend_out = schedule
        .spend_cap_usd
        .is_some_and(|cap| schedule.spend_used_usd >= cap);
    (tokens_out || spend_out).then_some(OccurrenceState::BudgetExhausted)
}

/// One scheduler pass. Decide due occurrences, then admit as many as the
/// per-schedule and global limits allow, oldest first across all schedules.
pub fn tick(
    store: &RecurringStore,
    queue_path: PathBuf,
    now: DateTime<Utc>,
    global_max_active: u32,
    budget: &dyn BudgetSource,
) -> Result<TickReport> {
    store.transact(|all| {
        all.iter_mut().for_each(|s| decide(s, now));
        Ok(())
    })?;
    store.transact(|all| {
        // Loaded under the store lock so the live counts are current.
        let mut queue = ScheduledQueue::load(queue_path);
        let live = |item: &super::ScheduledItem| {
            matches!(
                item.delivery.state,
                ItemState::Pending | ItemState::Dispatched
            )
        };
        let live_keys: Vec<String> = queue
            .items()
            .iter()
            .chain(queue.in_flight())
            .filter(|item| live(item))
            .map(|item| item.id.clone())
            .collect();
        let mut global_active = live_keys.iter().filter(|k| k.starts_with("cron:")).count();
        let mut active: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for schedule in all.iter() {
            let prefix = format!("cron:{}:", schedule.id);
            let count = live_keys.iter().filter(|k| k.starts_with(&prefix)).count();
            active.insert(schedule.id.clone(), count);
        }
        let mut order: Vec<(DateTime<Utc>, usize, usize)> = Vec::new();
        for (si, schedule) in all.iter().enumerate() {
            if schedule.paused {
                continue;
            }
            for (oi, occ) in schedule.occurrences.iter().enumerate() {
                if occ.state == OccurrenceState::Due {
                    order.push((occ.at, si, oi));
                }
            }
        }
        order.sort();
        let mut report = TickReport::default();
        for (_, si, oi) in order {
            let schedule = &mut all[si];
            let key = schedule.occurrences[oi].key.clone();
            // Already queued by an earlier pass that crashed before recording.
            let queued = queue
                .items()
                .iter()
                .chain(queue.in_flight())
                .chain(queue.recent_history(usize::MAX))
                .any(|item| item.id == key);
            if queued {
                schedule.occurrences[oi].state = OccurrenceState::Enqueued;
                continue;
            }
            if let Some(blocked) = budget_block(schedule, budget) {
                schedule.occurrences[oi].state = blocked;
                continue;
            }
            let per_schedule = active.get(&schedule.id).copied().unwrap_or(0);
            if per_schedule >= schedule.max_active as usize
                || global_active >= global_max_active as usize
            {
                report.waiting += 1;
                continue;
            }
            let mut request = schedule.template.clone();
            request.wake_at = Some(schedule.occurrences[oi].at);
            request.wake_in_minutes = None;
            let mut item = super::AmbientManager::scheduled_item(request);
            item.id = key;
            // Idempotent under the queue lock: a key already present in any
            // state (a crash between push and record) is not pushed twice.
            queue.try_push_unique(item)?;
            schedule.occurrences[oi].state = OccurrenceState::Enqueued;
            *active.entry(schedule.id.clone()).or_default() += 1;
            global_active += 1;
            report.enqueued += 1;
        }
        report.next_due = all
            .iter()
            .filter(|s| !s.paused)
            .filter_map(|s| s.upcoming(now, 1).into_iter().next().map(|o| o.at))
            .min();
        for schedule in all.iter_mut() {
            prune_records(schedule);
        }
        Ok(report)
    })
}

#[cfg(test)]
#[path = "recurring_tests.rs"]
mod recurring_tests;
