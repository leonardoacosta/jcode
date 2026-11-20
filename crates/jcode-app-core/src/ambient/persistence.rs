use anyhow::Result;
use chrono::{DateTime, Utc};
use std::path::PathBuf;

use super::paths::{lock_path, state_path};
use super::{
    AmbientCycleResult, AmbientState, AmbientStatus, CycleStatus, DeliveryAttempt, DeliveryRecord,
    ItemState, ScheduledItem,
};
use crate::storage;

#[cfg(test)]
#[path = "queue_transaction_tests.rs"]
mod queue_transaction_tests;

#[cfg(test)]
#[path = "scheduled_recovery_tests.rs"]
mod scheduled_recovery_tests;

// ---------------------------------------------------------------------------
// AmbientState persistence
// ---------------------------------------------------------------------------

impl AmbientState {
    pub fn load() -> Result<Self> {
        let path = state_path()?;
        if path.exists() {
            storage::read_json(&path)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        storage::write_json(&state_path()?, self)
    }

    pub fn record_cycle(&mut self, result: &AmbientCycleResult) {
        self.last_run = Some(result.ended_at);
        self.last_summary = Some(result.summary.clone());
        self.last_compactions = Some(result.compactions);
        self.last_memories_modified = Some(result.memories_modified);
        self.total_cycles += 1;

        match result.status {
            CycleStatus::Complete => {
                if let Some(ref req) = result.next_schedule {
                    let next = req.wake_at.unwrap_or_else(|| {
                        Utc::now()
                            + chrono::Duration::minutes(req.wake_in_minutes.unwrap_or(30) as i64)
                    });
                    self.status = AmbientStatus::Scheduled { next_wake: next };
                } else {
                    self.status = AmbientStatus::Idle;
                }
            }
            CycleStatus::Interrupted | CycleStatus::Incomplete => {
                self.status = AmbientStatus::Idle;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// ScheduledQueue
// ---------------------------------------------------------------------------

/// Read the queue entry by entry. Timed todo reviews (`todo_review` targets)
/// were removed with item-level blockers, and one unreadable legacy entry must
/// not cost the user every other scheduled task. A file that is not a JSON
/// list is still an error, so corruption never reads as an empty queue.
fn read_queue(path: &std::path::Path) -> Result<Vec<ScheduledItem>> {
    let entries: Vec<serde_json::Value> = storage::read_json(path)?;
    Ok(entries
        .into_iter()
        .filter_map(|entry| match serde_json::from_value(entry) {
            Ok(item) => Some(item),
            Err(error) => {
                crate::logging::warn(&format!(
                    "Dropping unreadable scheduled item from {}: {error}",
                    path.display()
                ));
                None
            }
        })
        .collect())
}

/// A `Dispatched` claim older than this is treated as abandoned whatever its
/// owner looks like, so no item can stay `dispatched` forever.
const CLAIM_STALE_AFTER: chrono::Duration = chrono::Duration::hours(1);

/// Identity of this process for claim ownership: `pid:start:nonce`. `start` is
/// the OS process start time (empty where unavailable). The pid and start time
/// together tell this process from a later one that reused the pid; the nonce
/// only tells this process from an earlier one with the same pid and start.
fn process_owner() -> &'static str {
    static OWNER: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
        let pid = std::process::id();
        let start = crate::platform::process_start_time(pid)
            .map(|start| start.to_string())
            .unwrap_or_default();
        format!("{pid}:{start}:{:016x}", rand::random::<u64>())
    });
    &OWNER
}

/// Whether a claim's owner can still settle it. Empty or unparsable owners
/// (legacy data) count as gone. An owner without a recorded start time cannot
/// be told from a pid-reusing process, so it only counts as alive while its
/// claim is younger than [`CLAIM_STALE_AFTER`].
fn owner_alive(owner: &str, dispatched_at: DateTime<Utc>) -> bool {
    if owner == process_owner() {
        return true;
    }
    let mut parts = owner.split(':');
    let Some(pid) = parts.next().and_then(|pid| pid.parse::<u32>().ok()) else {
        return false;
    };
    let start = match (parts.next(), parts.next()) {
        (Some(start), Some(_nonce)) => start.parse::<u64>().ok(),
        _ => None,
    };
    if pid == std::process::id() || !is_pid_alive(pid) {
        return false;
    }
    match start {
        Some(start) => crate::platform::process_start_time(pid) == Some(start),
        None => Utc::now() - dispatched_at < CLAIM_STALE_AFTER,
    }
}

/// True if no live process can be expected to settle this `Dispatched` item.
fn claim_stuck(record: &DeliveryRecord) -> bool {
    let Some(attempt) = record.attempts.last() else {
        return true;
    };
    Utc::now() - attempt.dispatched_at >= CLAIM_STALE_AFTER
        || !owner_alive(&attempt.owner, attempt.dispatched_at)
}

/// Turn every stuck `Dispatched` item with this id into `Uncertain`.
fn release_stuck(items: &mut [ScheduledItem], id: &str) {
    for item in items.iter_mut().filter(|item| {
        item.id == id && item.delivery.state == ItemState::Dispatched && claim_stuck(&item.delivery)
    }) {
        mark_uncertain(
            &mut item.delivery,
            "claim was abandoned without an acknowledgement",
        );
    }
}

fn mark_uncertain(record: &mut DeliveryRecord, note: &str) {
    record.state = ItemState::Uncertain;
    record.resolved_at = Some(Utc::now());
    if let Some(attempt) = record.attempts.last_mut() {
        attempt.note = Some(note.to_string());
    }
}

/// Terminal items kept in the queue file. `Uncertain` items are never pruned
/// because they still await a manual retry.
const HISTORY_LIMIT: usize = 100;

/// How a claimed (`Dispatched`) item ended. See [`ScheduledQueue::settle`].
#[derive(Debug, Clone, PartialEq)]
pub enum Settlement {
    /// The intended target accepted the message. This is not evidence that the
    /// work ran or succeeded.
    Acked {
        target: String,
        generation: Option<u64>,
    },
    /// Delivery was attempted but acceptance is unproven. May have run.
    Uncertain { note: String },
    /// The target was cancelled or revoked, so the wake was suppressed.
    Cancelled { note: String },
}

/// Counts from [`ScheduledQueue::recover_startup`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RecoveryReport {
    pub uncertain: usize,
    pub missed: usize,
}

/// Durable schedule queue.
///
/// One JSON file holds every item and its [`ItemState`]; the accessors split
/// it by state. Lifecycle (each step is one locked, atomic transaction):
///
/// ```text
/// Pending --claim--> Dispatched --settle--> Delivered | Uncertain | Cancelled
/// Pending | Uncertain | Missed --cancel--> Cancelled
/// Pending --recover_startup (overdue, direct target)--> Missed
/// Dispatched --recover_startup (claiming process gone)--> Uncertain
/// Uncertain | Missed --retry--> Pending (new attempt on next claim)
/// Dispatched (claim abandoned) --cancel or retry--> Uncertain first
/// ```
///
/// Claiming never removes an item, so a crash between claim and delivery
/// leaves a record. Nothing here delivers anything or redelivers on its own.
pub struct ScheduledQueue {
    pending: Vec<ScheduledItem>,
    in_flight: Vec<ScheduledItem>,
    history: Vec<ScheduledItem>,
    path: PathBuf,
}

impl ScheduledQueue {
    pub fn load(path: PathBuf) -> Self {
        let items: Vec<ScheduledItem> = if path.exists() {
            read_queue(&path).unwrap_or_default()
        } else {
            Vec::new()
        };
        let mut queue = Self {
            pending: Vec::new(),
            in_flight: Vec::new(),
            history: Vec::new(),
            path,
        };
        queue.set(items);
        queue
    }

    fn set(&mut self, items: Vec<ScheduledItem>) {
        self.pending.clear();
        self.in_flight.clear();
        self.history.clear();
        for item in items {
            match item.delivery.state {
                ItemState::Pending => self.pending.push(item),
                ItemState::Dispatched => self.in_flight.push(item),
                _ => self.history.push(item),
            }
        }
    }

    /// Lock a separate stable inode: atomic JSON replacement must not replace
    /// the inode carrying the lock. Reload under lock, and publish only after
    /// the entire mutation was durably saved. Never write a cached snapshot.
    /// `update` sees every item, whatever its state.
    pub(super) fn transact<T>(
        &mut self,
        update: impl FnOnce(&mut Vec<ScheduledItem>) -> Result<T>,
    ) -> Result<T> {
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
        let mut latest = if self.path.exists() {
            read_queue(&self.path)?
        } else {
            Vec::new()
        };
        let before = latest.clone();
        let result = update(&mut latest)?;
        prune_history(&mut latest);
        if latest != before {
            storage::write_json(&self.path, &latest)?;
        }
        self.set(latest);
        Ok(result)
    }

    pub fn try_push(&mut self, item: ScheduledItem) -> Result<()> {
        self.transact(|items| {
            items.push(item);
            Ok(())
        })
    }

    /// Push unless an item with this id exists in any state. Checked under the
    /// queue lock, so two passes can never both add the same occurrence.
    /// Returns whether the item was added.
    pub fn try_push_unique(&mut self, item: ScheduledItem) -> Result<bool> {
        self.transact(|items| {
            if items.iter().any(|existing| existing.id == item.id) {
                return Ok(false);
            }
            items.push(item);
            Ok(true)
        })
    }

    pub fn push(&mut self, item: ScheduledItem) {
        if let Err(error) = self.try_push(item) {
            crate::logging::warn(&format!("Failed to persist scheduled item: {error:#}"));
        }
    }

    /// Cancel a `Pending`, `Uncertain` or `Missed` item: it moves to history as
    /// `cancelled` and keeps its attempts. Returns `None` if there is no such
    /// item. A `Dispatched` item still being delivered cannot be cancelled,
    /// but one whose claim is abandoned (owner gone, or older than an hour)
    /// first becomes `Uncertain` and is then cancelled.
    pub fn cancel(&mut self, id: &str) -> Result<Option<ScheduledItem>> {
        self.transact(|items| {
            release_stuck(items, id);
            if let Some(item) = items.iter_mut().find(|item| {
                item.id == id
                    && matches!(
                        item.delivery.state,
                        ItemState::Pending | ItemState::Uncertain | ItemState::Missed
                    )
            }) {
                item.delivery.state = ItemState::Cancelled;
                item.delivery.resolved_at = Some(Utc::now());
                return Ok(Some(item.clone()));
            }
            anyhow::ensure!(
                !items
                    .iter()
                    .any(|item| item.id == id && item.delivery.state == ItemState::Dispatched),
                "Scheduled task '{id}' is already being delivered and cannot be cancelled"
            );
            Ok(None)
        })
    }

    /// Mark ready `Pending` items `Dispatched` and return them, highest
    /// priority first. The items stay persisted until [`Self::settle`].
    fn claim_ready(&mut self, direct_only: bool) -> Result<Vec<ScheduledItem>> {
        self.transact(|items| {
            let now = Utc::now();
            let mut ready = Vec::new();
            for item in items.iter_mut().filter(|item| {
                item.delivery.state == ItemState::Pending
                    && item.scheduled_for <= now
                    && (!direct_only || item.target.is_direct_delivery())
            }) {
                let record = &mut item.delivery;
                record.state = ItemState::Dispatched;
                record.attempts.push(DeliveryAttempt {
                    number: record.attempts.len() as u32 + 1,
                    dispatched_at: now,
                    manual_retry: std::mem::take(&mut record.retry_pending),
                    acked_at: None,
                    acked_target: None,
                    generation: None,
                    owner: process_owner().to_string(),
                    note: None,
                });
                ready.push(item.clone());
            }
            ready.sort_by(|a, b| {
                b.priority
                    .cmp(&a.priority)
                    .then_with(|| a.scheduled_for.cmp(&b.scheduled_for))
            });
            Ok(ready)
        })
    }

    /// Claim every ready item. Callers must [`Self::settle`] each one.
    pub fn pop_ready(&mut self) -> Vec<ScheduledItem> {
        self.claim_ready(false).unwrap_or_else(|error| {
            crate::logging::warn(&format!("Failed to claim scheduled items: {error:#}"));
            Vec::new()
        })
    }

    /// Claim ready items that target a session directly. Callers must
    /// [`Self::settle`] each one.
    pub fn take_ready_direct_items(&mut self) -> Vec<ScheduledItem> {
        self.claim_ready(true).unwrap_or_else(|error| {
            crate::logging::warn(&format!(
                "Failed to claim direct scheduled items: {error:#}"
            ));
            Vec::new()
        })
    }

    /// Record how a `Dispatched` item ended and move it to history. Returns
    /// `false` if no such dispatched item exists (already settled).
    pub fn settle(&mut self, id: &str, outcome: Settlement) -> Result<bool> {
        self.transact(|items| {
            let Some(item) = items
                .iter_mut()
                .find(|item| item.id == id && item.delivery.state == ItemState::Dispatched)
            else {
                return Ok(false);
            };
            let now = Utc::now();
            let record = &mut item.delivery;
            let attempt = record.attempts.last_mut();
            match outcome {
                Settlement::Acked { target, generation } => {
                    record.state = ItemState::Delivered;
                    if let Some(attempt) = attempt {
                        attempt.acked_at = Some(now);
                        attempt.acked_target = Some(target);
                        attempt.generation = generation;
                    }
                }
                Settlement::Uncertain { note } => {
                    record.state = ItemState::Uncertain;
                    if let Some(attempt) = attempt {
                        attempt.note = Some(note);
                    }
                }
                Settlement::Cancelled { note } => {
                    record.state = ItemState::Cancelled;
                    if let Some(attempt) = attempt {
                        attempt.note = Some(note);
                    }
                }
            }
            record.resolved_at = Some(now);
            Ok(true)
        })
    }

    /// Run once when a runner starts. Never delivers anything.
    ///
    /// - `Dispatched` items whose claiming process is gone (or whose claim is
    ///   older than an hour) become `Uncertain`: the target may or may not
    ///   have run them. Claims owned by this process or another live one are
    ///   left to their owner to settle.
    /// - `Pending` direct-delivery items overdue by more than `grace` become
    ///   `Missed`. Ambient-target items are left alone: the ambient agent
    ///   reads them from the queue rather than receiving a delivery.
    pub fn recover_startup(&mut self, grace: chrono::Duration) -> Result<RecoveryReport> {
        self.transact(|items| {
            let now = Utc::now();
            let mut report = RecoveryReport::default();
            for item in items.iter_mut() {
                let record = &mut item.delivery;
                match record.state {
                    ItemState::Dispatched => {
                        if claim_stuck(record) {
                            mark_uncertain(
                                record,
                                "no acknowledgement was recorded before the runner stopped",
                            );
                            report.uncertain += 1;
                        }
                    }
                    ItemState::Pending
                        if !record.retry_pending
                            && item.target.is_direct_delivery()
                            && now - item.scheduled_for > grace =>
                    {
                        record.state = ItemState::Missed;
                        record.resolved_at = Some(now);
                        record.missed_delay_secs = Some((now - item.scheduled_for).num_seconds());
                        report.missed += 1;
                    }
                    _ => {}
                }
            }
            Ok(report)
        })
    }

    /// Explicit manual retry of an `Uncertain` or `Missed` item (or an
    /// abandoned `Dispatched` claim, see [`Self::cancel`]): it becomes
    /// `Pending` again and its next claim records a new attempt. Earlier
    /// attempts are kept. Returns `None` if there is nothing to retry. The
    /// caller must warn that the earlier attempt may already have had effects.
    pub fn retry(&mut self, id: &str) -> Result<Option<ScheduledItem>> {
        self.transact(|items| {
            release_stuck(items, id);
            let Some(item) = items.iter_mut().rev().find(|item| {
                item.id == id
                    && matches!(
                        item.delivery.state,
                        ItemState::Uncertain | ItemState::Missed
                    )
            }) else {
                return Ok(None);
            };
            item.delivery.state = ItemState::Pending;
            item.delivery.retry_pending = true;
            item.delivery.resolved_at = None;
            Ok(Some(item.clone()))
        })
    }

    pub fn peek_next(&self) -> Option<&ScheduledItem> {
        self.pending.iter().min_by_key(|i| i.scheduled_for)
    }

    /// Number of `Pending` items.
    pub fn len(&self) -> usize {
        self.pending.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    /// `Pending` items only.
    pub fn items(&self) -> &[ScheduledItem] {
        &self.pending
    }

    /// `Dispatched` items awaiting acknowledgement.
    pub fn in_flight(&self) -> &[ScheduledItem] {
        &self.in_flight
    }

    /// The `limit` most recently resolved terminal items, newest first, plus
    /// any older `Uncertain` item: it still awaits a manual retry.
    pub fn recent_history(&self, limit: usize) -> Vec<&ScheduledItem> {
        let mut recent: Vec<_> = self.history.iter().collect();
        recent.sort_by_key(|item| std::cmp::Reverse(item.delivery.resolved_at));
        let older = recent.split_off(limit.min(recent.len()));
        recent.extend(
            older
                .into_iter()
                .filter(|item| item.delivery.state == ItemState::Uncertain),
        );
        recent
    }
}

/// Drop the oldest terminal items beyond [`HISTORY_LIMIT`], sparing `Uncertain`.
fn prune_history(items: &mut Vec<ScheduledItem>) {
    let mut prunable: Vec<_> = items
        .iter()
        .enumerate()
        .filter(|(_, item)| {
            matches!(
                item.delivery.state,
                ItemState::Delivered | ItemState::Missed | ItemState::Cancelled
            )
        })
        .map(|(index, item)| (item.delivery.resolved_at, index))
        .collect();
    if prunable.len() <= HISTORY_LIMIT {
        return;
    }
    prunable.sort();
    let excess = prunable.len() - HISTORY_LIMIT;
    let drop: std::collections::HashSet<usize> =
        prunable.into_iter().take(excess).map(|(_, i)| i).collect();
    let mut index = 0;
    items.retain(|_| {
        index += 1;
        !drop.contains(&(index - 1))
    });
}

// ---------------------------------------------------------------------------
// AmbientLock  (single-instance guard)
// ---------------------------------------------------------------------------

pub struct AmbientLock {
    pub(crate) lock_path: PathBuf,
}

impl AmbientLock {
    /// Try to acquire the ambient lock.
    /// Returns `Ok(Some(lock))` if acquired, `Ok(None)` if another instance
    /// already holds it, or `Err` on I/O failure.
    pub fn try_acquire() -> Result<Option<Self>> {
        let path = lock_path()?;

        // Check existing lock
        if path.exists() {
            if let Ok(contents) = std::fs::read_to_string(&path)
                && let Ok(pid) = contents.trim().parse::<u32>()
                && is_pid_alive(pid)
            {
                return Ok(None); // Another instance is running
            }
            let _ = std::fs::remove_file(&path);
        }

        // Write our PID
        let pid = std::process::id();
        if let Some(parent) = path.parent() {
            storage::ensure_dir(parent)?;
        }
        std::fs::write(&path, pid.to_string())?;

        Ok(Some(Self { lock_path: path }))
    }

    pub fn release(self) -> Result<()> {
        let _ = std::fs::remove_file(&self.lock_path);
        // Drop runs, but we already cleaned up
        std::mem::forget(self);
        Ok(())
    }
}

impl Drop for AmbientLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.lock_path);
    }
}

fn is_pid_alive(pid: u32) -> bool {
    crate::platform::is_process_running(pid)
}
