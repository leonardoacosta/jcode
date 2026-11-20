//! Run tracking for recurring schedules: when a delivered occurrence's turn
//! ends, and hard token caps enforced from the provider's own usage reports.
//!
//! Queue acknowledgement only says the target accepted the message. This module
//! adds the missing signal. A run is tracked from delivery until the target
//! session's turn that carried it ends (`Completed`), or until a cap stops it
//! (`LimitStopped`), or until we can no longer know (`Unknown`).
//!
//! Hooks are in-memory and cheap: the turn registry calls [`turn_started`] and
//! [`turn_ended`], the stream recorder calls [`observe_usage`]. None of them
//! writes a file except a usage record for a *capped* tracked run. Completion is
//! persisted by the scheduler tick ([`reconcile`]), so the turn guard's `Drop`
//! never does I/O.
//!
//! Limits, stated plainly:
//! * Ambient-target occurrences are not tracked (no turn signal): they count as
//!   active only until delivery.
//! * A cap fires the cancel signal of the target session's turn and of every
//!   session it owns. One provider response can overshoot the cap, and a new
//!   turn started later in the same session is not blocked.
//! * Spend caps are not enforceable (no price or reservation source), so
//!   [`super::recurring::ProviderUsageBudget`] refuses to start such schedules.

use super::recurring::{RecurringStore, RunRecord, RunState, Schedule};
use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

/// A run with no completion signal after this long becomes `Unknown`.
pub const RUN_TIMEOUT: Duration = Duration::hours(6);
/// A cap-stopped run whose turns are still alive after this long is `Unknown`.
pub const STOP_DEADLINE: Duration = Duration::minutes(5);
/// A persisted active run that this process never tracked is stale after this.
const UNTRACKED_GRACE: Duration = Duration::minutes(2);
/// A tracked run with no persisted record (delivery never acknowledged) is
/// forgotten after this.
const PENDING_DELIVERY_GRACE: Duration = Duration::hours(1);
const MAX_RUNS: usize = 100;
const MAX_OWNER_HOPS: usize = 16;

#[derive(Debug, Clone)]
pub(crate) struct Tracked {
    pub schedule_id: String,
    pub key: String,
    pub session_id: String,
    pub since: DateTime<Utc>,
    pub turn_seen: bool,
    pub ended: bool,
    pub token_cap: Option<u64>,
    pub stopping_since: Option<DateTime<Utc>>,
}

#[derive(Default, Clone)]
pub(crate) struct Tracker {
    runs: Vec<Tracked>,
}

impl Tracker {
    pub(crate) fn has_runs(&self) -> bool {
        !self.runs.is_empty()
    }

    /// Only a capped run needs usage attribution.
    pub(crate) fn has_capped_runs(&self) -> bool {
        self.runs.iter().any(|r| r.token_cap.is_some() && !r.ended)
    }

    pub(crate) fn turn_started(&mut self, session: &str) {
        for run in self.runs.iter_mut().filter(|r| r.session_id == session) {
            run.turn_seen = true;
        }
    }

    /// Returns whether a tracked run finished, so the caller wakes the runner.
    pub(crate) fn turn_ended(&mut self, session: &str, remaining: usize) -> bool {
        if remaining > 0 {
            return false;
        }
        let mut woke = false;
        for run in self.runs.iter_mut().filter(|r| r.session_id == session) {
            if run.turn_seen && !run.ended {
                run.ended = true;
                woke = true;
            }
        }
        woke
    }

    /// Track a run. Registered before delivery for session targets, so usage
    /// and turn events during delivery are not missed. Registering the same key
    /// again keeps what was already observed.
    pub(crate) fn register(&mut self, mut run: Tracked, turn_active_now: bool) {
        run.turn_seen = run.turn_seen || turn_active_now;
        if let Some(existing) = self.runs.iter().find(|r| r.key == run.key) {
            run.turn_seen |= existing.turn_seen;
            run.ended |= existing.ended;
            run.stopping_since = run.stopping_since.or(existing.stopping_since);
        }
        self.runs.retain(|r| r.key != run.key);
        self.runs.push(run);
    }

    pub(crate) fn forget(&mut self, key: &str) {
        self.runs.retain(|r| r.key != key);
    }

    fn tracked(&self, key: &str) -> Option<&Tracked> {
        self.runs.iter().find(|r| r.key == key)
    }
}

static TRACKER: LazyLock<Mutex<Tracker>> = LazyLock::new(Default::default);

pub(crate) fn tracker() -> std::sync::MutexGuard<'static, Tracker> {
    TRACKER.lock().unwrap_or_else(|e| e.into_inner())
}

/// Everything the module needs from the live process, so logic is testable.
pub(crate) trait Probe {
    fn turn_active(&self, session: &str) -> bool;
    /// Sessions the run's target owns, directly or transitively.
    fn owned(&self, session: &str) -> Vec<String>;
    /// The session followed up its owner chain, nearest first, itself included.
    fn owner_chain(&self, session: &str) -> Vec<String>;
    fn fire_cancel(&self, session: &str);
}

pub(crate) struct LiveProbe;

impl Probe for LiveProbe {
    fn turn_active(&self, session: &str) -> bool {
        !crate::turn_cancel_registry::active_turn_signals(session).is_empty()
    }

    fn owned(&self, session: &str) -> Vec<String> {
        crate::server::session_control::store()
            .owned_records(session)
            .into_iter()
            .map(|record| record.session_id)
            .collect()
    }

    fn owner_chain(&self, session: &str) -> Vec<String> {
        let store = crate::server::session_control::store();
        let mut chain = vec![session.to_string()];
        let mut current = session.to_string();
        for _ in 0..MAX_OWNER_HOPS {
            let Some(owner) = store
                .inspect(&current)
                .ok()
                .flatten()
                .and_then(|record| record.owner_session_id)
            else {
                break;
            };
            chain.push(owner.clone());
            current = owner;
        }
        chain
    }

    fn fire_cancel(&self, session: &str) {
        for signal in crate::turn_cancel_registry::active_turn_signals(session) {
            signal.fire();
        }
    }
}

// ---------------------------------------------------------------------------
// Process-wide hooks
// ---------------------------------------------------------------------------

/// Turn registry hook: a turn started in `session`.
pub(crate) fn turn_started(session: &str) {
    let mut tracker = tracker();
    if tracker.has_runs() {
        tracker.turn_started(session);
    }
}

/// Turn registry hook: a turn in `session` ended; `remaining` turns are still
/// active there. Never does I/O: completion is persisted by the next tick.
pub(crate) fn turn_ended(session: &str, remaining: usize) {
    let wake = {
        let mut tracker = tracker();
        tracker.has_runs() && tracker.turn_ended(session, remaining)
    };
    if wake {
        crate::tool::ambient::wake_schedule_runner();
    }
}

/// The session a recurring item's delivery targets, if its turns can be
/// observed. Ambient and spawn targets cannot be tracked from before delivery.
fn trackable_session(item: &crate::ambient::ScheduledItem) -> Option<String> {
    match &item.target {
        crate::ambient::ScheduleTarget::Session { session_id } => Some(session_id.clone()),
        _ => None,
    }
}

/// Start of a delivery attempt. For a recurring item aimed at a session this
/// registers the run in memory BEFORE delivery, so turn and usage events that
/// happen during delivery (a headless resume awaits its whole turn) are seen.
/// In memory only: nothing is persisted until the delivery is acknowledged.
pub(crate) fn begin_dispatch(item: &crate::ambient::ScheduledItem) -> DateTime<Utc> {
    let now = Utc::now();
    let Some((schedule_id, _)) = item
        .id
        .strip_prefix("cron:")
        .and_then(|rest| rest.split_once(':'))
    else {
        return now;
    };
    let Some(session) = trackable_session(item) else {
        return now;
    };
    // The cap is read before taking the tracker lock; never I/O under it.
    let cap = RecurringStore::open_default()
        .and_then(|store| store.get(schedule_id))
        .ok()
        .flatten()
        .and_then(|schedule| schedule.token_cap);
    let active = LiveProbe.turn_active(&session);
    tracker().register(
        Tracked {
            schedule_id: schedule_id.to_string(),
            key: item.id.clone(),
            session_id: session,
            since: now,
            turn_seen: false,
            ended: false,
            token_cap: cap,
            stopping_since: None,
        },
        active,
    );
    now
}

/// The delivery did not reach acknowledgement: stop tracking it.
pub(crate) fn forget_dispatch(key: &str) {
    if key.starts_with("cron:") {
        tracker().forget(key);
    }
}

/// A recurring occurrence was delivered and acknowledged: persist `Running`.
/// A retry of a finished occurrence resets its record. Registration happens
/// inside the store transaction (store, then tracker), so a concurrent tick can
/// never reconcile between the two and drop the run.
pub(crate) fn note_delivered(key: &str, session: &str, since: DateTime<Utc>) -> Result<()> {
    let Some((schedule_id, _)) = key
        .strip_prefix("cron:")
        .and_then(|rest| rest.split_once(':'))
    else {
        return Ok(());
    };
    if session == "ambient" {
        return Ok(());
    }
    let store = RecurringStore::open_default()?;
    let active = LiveProbe.turn_active(session);
    store.transact(|all| {
        let Some(schedule) = all.iter_mut().find(|s| s.id == schedule_id) else {
            return Ok(());
        };
        tracker().register(
            Tracked {
                schedule_id: schedule_id.to_string(),
                key: key.to_string(),
                session_id: session.to_string(),
                since,
                turn_seen: false,
                ended: false,
                token_cap: schedule.token_cap,
                stopping_since: None,
            },
            active,
        );
        let now = Utc::now();
        match schedule.runs.iter_mut().find(|r| r.key == key) {
            Some(record) if !record.state.active() => {
                // A manual retry of an earlier attempt: a new run.
                record.state = RunState::Running;
                record.session_id = session.to_string();
                record.started_at = now;
                record.finished_at = None;
                record.note = Some("retried".into());
            }
            Some(_) => {}
            None => schedule.runs.push(RunRecord {
                key: key.to_string(),
                session_id: session.to_string(),
                state: RunState::Running,
                started_at: now,
                finished_at: None,
                note: None,
            }),
        }
        prune_runs(schedule);
        Ok(())
    })
}

/// Stream recorder hook: one provider response finished for `session`. Counts
/// toward a capped tracked run (the session itself or one it owns) once per
/// `event_id`, and stops the run when its cap is reached.
pub(crate) fn observe_usage(event_id: &str, session: &str, counters: [Option<u64>; 4]) {
    if !tracker().has_capped_runs() {
        return;
    }
    let [input, output, _cache_read, cache_creation] = counters;
    // Same convention as overnight: cache reads are not billed as new tokens.
    let tokens = input
        .unwrap_or(0)
        .saturating_add(output.unwrap_or(0))
        .saturating_add(cache_creation.unwrap_or(0));
    if tokens == 0 {
        return;
    }
    let Ok(store) = RecurringStore::open_default() else {
        return;
    };
    if let Err(error) = apply_usage_live(&store, &LiveProbe, event_id, session, tokens, Utc::now())
    {
        crate::logging::warn(&format!("Recurring usage not recorded: {error:#}"));
    }
}

/// Lock discipline: the tracker is the innermost lock and is never held over
/// I/O. Snapshot under it, do store and probe work without it, then relock
/// briefly to record the decision.
fn apply_usage_live(
    store: &RecurringStore,
    probe: &dyn Probe,
    event_id: &str,
    session: &str,
    tokens: u64,
    now: DateTime<Utc>,
) -> Result<()> {
    let chain = probe.owner_chain(session);
    let snapshot = tracker().clone();
    let outcome = usage_decision(store, &snapshot, probe, &chain, event_id, tokens)?;
    if let Some(key) = outcome.stop_key {
        tracker().mark_stopping(&key, now);
    }
    Ok(())
}

impl Tracker {
    fn mark_stopping(&mut self, key: &str, now: DateTime<Utc>) {
        if let Some(run) = self.runs.iter_mut().find(|r| r.key == key) {
            run.stopping_since.get_or_insert(now);
        }
    }
}

/// Test wrapper: one usage event against an owned tracker.
#[cfg(test)]
pub(crate) fn apply_usage(
    store: &RecurringStore,
    tracker: &mut Tracker,
    probe: &dyn Probe,
    event_id: &str,
    session: &str,
    tokens: u64,
    now: DateTime<Utc>,
) -> Result<()> {
    let chain = probe.owner_chain(session);
    let outcome = usage_decision(store, tracker, probe, &chain, event_id, tokens)?;
    if let Some(key) = outcome.stop_key {
        tracker.mark_stopping(&key, now);
    }
    Ok(())
}

pub(crate) struct UsageOutcome {
    /// Set when this response reached the cap and the run was cancelled.
    pub stop_key: Option<String>,
}

/// Record one usage event and, on reaching the cap, cancel the run's sessions.
/// Reads `tracker` only; the caller owns any locking.
pub(crate) fn usage_decision(
    store: &RecurringStore,
    tracker: &Tracker,
    probe: &dyn Probe,
    chain: &[String],
    event_id: &str,
    tokens: u64,
) -> Result<UsageOutcome> {
    let none = UsageOutcome { stop_key: None };
    let Some(run) = chain.iter().find_map(|id| {
        tracker
            .runs
            .iter()
            .find(|r| &r.session_id == id && r.token_cap.is_some() && !r.ended)
            .cloned()
    }) else {
        return Ok(none);
    };
    // A replay changes nothing and never cancels again.
    if !store.record_usage(&run.schedule_id, event_id, &run.key, tokens, 0.0)? {
        return Ok(none);
    }
    let total = store
        .get(&run.schedule_id)?
        .map(|s| s.tokens_used)
        .unwrap_or(0);
    let cap = run.token_cap.unwrap_or(u64::MAX);
    if total < cap || run.stopping_since.is_some() {
        return Ok(none);
    }
    for id in std::iter::once(run.session_id.clone()).chain(probe.owned(&run.session_id)) {
        probe.fire_cancel(&id);
    }
    // Persisted before the caller records the in-memory decision; if this
    // write fails, reconcile still sees `stopping_since` and settles it.
    store.transact(|all| {
        if let Some(record) = all
            .iter_mut()
            .flat_map(|s| s.runs.iter_mut())
            .find(|r| r.key == run.key && r.state == RunState::Running)
        {
            record.state = RunState::Stopping;
            record.note = Some(format!(
                "token cap {cap} reached ({total} used); cancelling"
            ));
        }
        Ok(())
    })?;
    Ok(UsageOutcome {
        stop_key: Some(run.key),
    })
}

/// Persist the scheduler's view of every tracked run. Called inside the
/// tick's store transaction (store lock held), so the tracker is only locked
/// briefly, twice, and never across the probe's file reads.
pub(crate) fn reconcile(all: &mut [Schedule], now: DateTime<Utc>) {
    let snapshot = tracker().clone();
    let mut local = snapshot.clone();
    reconcile_with(all, now, &mut local, &LiveProbe);
    let kept: std::collections::HashSet<&str> = local.runs.iter().map(|r| r.key.as_str()).collect();
    let dropped: std::collections::HashSet<&str> = snapshot
        .runs
        .iter()
        .map(|r| r.key.as_str())
        .filter(|key| !kept.contains(key))
        .collect();
    // Runs registered after the snapshot are not in `dropped` and survive.
    if !dropped.is_empty() {
        tracker().runs.retain(|r| !dropped.contains(r.key.as_str()));
    }
}

/// Server start: this process tracks nothing, so every persisted active run is
/// from a process that is gone. It may or may not have finished, and it is
/// never redelivered. Usage already recorded stays counted.
pub(crate) fn recover_on_startup() -> Result<usize> {
    let store = RecurringStore::open_default()?;
    store.transact(|all| {
        let mut changed = 0;
        for schedule in all.iter_mut() {
            for run in schedule.runs.iter_mut().filter(|r| r.state.active()) {
                run.state = RunState::Unknown;
                run.finished_at = Some(Utc::now());
                run.note = Some("server restarted before a completion signal".into());
                changed += 1;
            }
        }
        Ok(changed)
    })
}

// ---------------------------------------------------------------------------
// Logic (pure over a tracker, a store and a probe)
// ---------------------------------------------------------------------------

pub(crate) fn reconcile_with(
    all: &mut [Schedule],
    now: DateTime<Utc>,
    tracker: &mut Tracker,
    probe: &dyn Probe,
) {
    for schedule in all.iter_mut() {
        for run in schedule.runs.iter_mut().filter(|r| r.state.active()) {
            let tracked = tracker.tracked(&run.key).cloned();
            // A clock stepped backward must not hold a slot until it catches up.
            let age = (now - run.started_at).max(Duration::zero());
            let finish = |run: &mut RunRecord, state: RunState, note: &str| {
                run.state = state;
                run.finished_at = Some(now);
                run.note = Some(note.to_string());
            };
            let Some(t) = tracked else {
                if age > UNTRACKED_GRACE {
                    finish(
                        run,
                        RunState::Unknown,
                        "no live tracker: the server that delivered it is gone",
                    );
                }
                continue;
            };
            // A cap decision made in memory counts even if its record write
            // failed: settle it as stopping.
            if run.state == RunState::Running && t.stopping_since.is_some() {
                run.state = RunState::Stopping;
            }
            match run.state {
                RunState::Stopping => {
                    let alive = std::iter::once(t.session_id.clone())
                        .chain(probe.owned(&t.session_id))
                        .any(|id| probe.turn_active(&id));
                    if !alive {
                        finish(
                            run,
                            RunState::LimitStopped,
                            "token cap reached; owned turns confirmed ended",
                        );
                    } else if t.stopping_since.is_some_and(|at| now - at > STOP_DEADLINE) {
                        finish(
                            run,
                            RunState::Unknown,
                            "token cap reached; stop not confirmed in time",
                        );
                    }
                }
                RunState::Running if t.ended => finish(run, RunState::Completed, "turn ended"),
                RunState::Running if age > RUN_TIMEOUT => finish(
                    run,
                    RunState::Unknown,
                    "no completion signal within the run timeout",
                ),
                _ => {}
            }
        }
        prune_runs(schedule);
    }
    // Forget a tracked run once its record finished. One with no record yet is
    // a delivery in progress and is kept, unless it is long abandoned.
    let states: HashMap<&str, RunState> = all
        .iter()
        .flat_map(|s| s.runs.iter())
        .map(|r| (r.key.as_str(), r.state))
        .collect();
    tracker.runs.retain(|r| match states.get(r.key.as_str()) {
        Some(state) => state.active(),
        None => now - r.since <= PENDING_DELIVERY_GRACE,
    });
}

fn prune_runs(schedule: &mut Schedule) {
    let finished = schedule.runs.iter().filter(|r| !r.state.active()).count();
    let mut excess = finished.saturating_sub(MAX_RUNS);
    schedule.runs.retain(|r| {
        if !r.state.active() && excess > 0 {
            excess -= 1;
            false
        } else {
            true
        }
    });
}

/// Refuse to leave a stuck run holding capacity forever. See [`reconcile`].
pub(crate) fn expire_stale_runs(all: &mut [Schedule], now: DateTime<Utc>) {
    reconcile(all, now);
}
