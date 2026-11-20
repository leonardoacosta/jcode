//! Durable tracking of MCP tasks (Tasks extension, spec 2026-07-28).
//!
//! The tracker reuses the background-task record: every tracked remote task is
//! one `TaskStatusFile` with `mcp_task` set (see `crate::background`). There is
//! no second registry. The tracker owns only the polling loops, which are
//! derived state: after a restart [`McpTaskTracker::resume_all`] rebuilds them
//! from the persisted records.
//!
//! Rules the code below enforces:
//! * A task handle is persisted before it is returned ([`McpTaskTracker::register`]).
//! * `tasks/get` is the only source of remote state. `notifications/tasks`
//!   arrive as [`McpTaskTracker::hint`] and only wake the poll loop.
//! * A saved task id is only ever sent to the server identity that created it.
//! * Transport errors keep the last known state and back off. They never
//!   complete or fail a task.
//! * The originating `tools/call` is never replayed (this module cannot send
//!   one: [`TaskRemote`] has no such method).
//! * `input_required` is surfaced and never answered.

use super::tasks::{RemoteStatus, RemoteTask, ResolveError, TaskPoll, TaskSource};
use crate::background::{
    BackgroundTaskManager, MCP_TASK_INPUT_CAP, MCP_TASK_RESULT_CAP, McpRemoteState, McpTaskRecord,
    TaskStatusFile, bound_json,
};
use crate::bus::BackgroundTaskStatus;
use anyhow::{Result, anyhow, bail};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;
use tokio::sync::Notify;
use tokio::task::JoinHandle;

/// Poll interval when the server gives no `pollIntervalMs`.
pub const DEFAULT_POLL_INTERVAL: Duration = Duration::from_secs(5);
/// Floor for any poll interval, including server-suggested ones.
pub const MIN_POLL_INTERVAL: Duration = Duration::from_secs(1);
/// Ceiling for poll intervals and for the transport-error backoff.
pub const MAX_POLL_DELAY: Duration = Duration::from_secs(300);
/// Longest error reason stored in a record.
const MAX_REASON_BYTES: usize = 200;

/// Wall clock and sleep, injectable so tests control polling cadence and
/// expiry without waiting.
#[async_trait]
pub trait TrackerClock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
    async fn sleep(&self, duration: Duration);
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

#[async_trait]
impl TrackerClock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }

    async fn sleep(&self, duration: Duration) {
        tokio::time::sleep(duration).await;
    }
}

impl super::tasks::TaskHintSink for McpTaskTracker {
    fn tracked_task_ids(&self, server: &str) -> Vec<String> {
        let mut ids: Vec<String> = self
            .inner
            .loops()
            .values()
            .filter(|entry| entry.server == server)
            .map(|entry| entry.remote_task_id.clone())
            .collect();
        ids.sort();
        ids.dedup();
        ids
    }

    fn hint(&self, server: &str, task_id: &str) {
        McpTaskTracker::hint(self, server, task_id);
    }

    fn changed(&self) -> Arc<Notify> {
        Arc::clone(&self.inner.changed)
    }
}

/// Result of [`McpTaskTracker::cancel`]. None of these means "the task was
/// stopped": only [`CancelOutcome::Confirmed`] reports the server's own
/// `cancelled` state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CancelOutcome {
    /// `tasks/cancel` was acknowledged and `tasks/get` reports `cancelled`.
    Confirmed,
    /// The server reached this terminal state first. Its outcome is kept and
    /// the cancellation did not take effect.
    ServerFinished(RemoteStatus),
    /// Acknowledged, but the task is not (yet) reported as terminal. Polling
    /// continues.
    Requested,
    /// The request failed or its reply was lost. Cancellation is unconfirmed
    /// and polling continues.
    Unacknowledged(String),
    /// The task could not be matched to a server (unknown or changed). Nothing
    /// was sent; the local record was closed as lost and the remote task, if
    /// any, keeps running.
    AbandonedLocally,
}

struct LoopEntry {
    generation: u64,
    server: String,
    remote_task_id: String,
    wake: Arc<Notify>,
    join: JoinHandle<()>,
}

struct Inner {
    source: Arc<dyn TaskSource>,
    manager: Arc<BackgroundTaskManager>,
    clock: Arc<dyn TrackerClock>,
    /// Polling loops by background task id.
    loops: Mutex<HashMap<String, LoopEntry>>,
    next_generation: AtomicU64,
    /// Fires when a polling loop starts or ends, so the listen stream can
    /// reopen with the new set of task ids.
    changed: Arc<Notify>,
}

/// Durable tracker for MCP tasks. Cheap to clone.
#[derive(Clone)]
pub struct McpTaskTracker {
    inner: Arc<Inner>,
}

/// What one poll decided about the loop.
enum Step {
    /// Server answered. Poll again after the task's interval.
    Next,
    /// Transport or persistence problem. Poll again after a backoff.
    Backoff,
    /// Tracking ended or is suspended (terminal, not found, unresolved).
    Stop,
}

static GLOBAL: OnceLock<McpTaskTracker> = OnceLock::new();

impl McpTaskTracker {
    pub fn new(source: Arc<dyn TaskSource>, manager: Arc<BackgroundTaskManager>) -> Self {
        Self::with_clock(source, manager, Arc::new(SystemClock))
    }

    pub fn with_clock(
        source: Arc<dyn TaskSource>,
        manager: Arc<BackgroundTaskManager>,
        clock: Arc<dyn TrackerClock>,
    ) -> Self {
        Self {
            inner: Arc::new(Inner {
                source,
                manager,
                clock,
                loops: Mutex::new(HashMap::new()),
                next_generation: AtomicU64::new(1),
                changed: Arc::new(Notify::new()),
            }),
        }
    }

    /// Install the process-wide tracker used by the `bg` tool for cancellation.
    /// First call wins; returns whether this call installed it.
    pub fn install_global(tracker: McpTaskTracker) -> bool {
        GLOBAL.set(tracker).is_ok()
    }

    pub fn global() -> Option<McpTaskTracker> {
        GLOBAL.get().cloned()
    }

    /// Persist a task handle received from `tools/call` and start tracking it.
    ///
    /// Persistence comes first. On any error nothing is tracked and the caller
    /// must not hand the handle out. Returns the background task id.
    pub async fn register(
        &self,
        handle: RemoteTask,
        server: &str,
        identity: &str,
        tool: &str,
        session_id: &str,
        tool_call_id: &str,
    ) -> Result<String> {
        let now = self.inner.clock.now();
        let mut record = McpTaskRecord {
            server: server.to_string(),
            identity: identity.to_string(),
            remote_task_id: handle.task_id.clone(),
            tool: tool.to_string(),
            tool_call_id: tool_call_id.to_string(),
            remote_status: handle.status,
            status_message: None,
            created_at: handle.created_at.clone(),
            last_updated_at: handle.last_updated_at.clone(),
            ttl_ms: None,
            poll_interval_ms: None,
            expires_at: None,
            cancel_requested: false,
            cancel_error: None,
            result: None,
            result_truncated: false,
            error: None,
            input_requests: None,
            remote_state: McpRemoteState::Tracking,
        };
        take_snapshot_fields(&mut record, &handle, now);
        let terminal = record.remote_status.is_terminal();
        let info = self
            .inner
            .manager
            .register_mcp_task(record, session_id, true, false)
            .await
            .map_err(|error| anyhow!("MCP task was not tracked: cannot persist it: {error}"))?;
        if !terminal {
            self.inner
                .ensure_loop(&info.task_id, server, &handle.task_id, false);
        }
        Ok(info.task_id)
    }

    /// Resume polling for every persisted nonterminal MCP task, after a daemon
    /// restart or an MCP reconnect. Each task is resolved first; a task whose
    /// server is gone or has a different identity becomes `Unresolved` without
    /// any request reaching a server. Returns how many loops were started.
    pub async fn resume_all(&self) -> usize {
        let mut started = 0;
        for task in self.inner.manager.list().await {
            let Some(record) = task.mcp_task.as_ref() else {
                continue;
            };
            if task.status != BackgroundTaskStatus::Running {
                continue;
            }
            if self
                .inner
                .ensure_loop(&task.task_id, &record.server, &record.remote_task_id, true)
            {
                started += 1;
            }
        }
        started
    }

    /// `notifications/tasks` wake hint. Triggers an immediate `tasks/get` for
    /// the tracked task of exactly this server and remote id. The notification
    /// payload is never used as state. Returns whether a tracked task matched;
    /// unknown servers and ids, and ids that belong to another server, are
    /// ignored.
    pub fn hint(&self, server: &str, remote_task_id: &str) -> bool {
        let loops = self.inner.loops();
        let mut matched = false;
        for entry in loops.values() {
            if entry.server == server && entry.remote_task_id == remote_task_id {
                entry.wake.notify_one();
                matched = true;
            }
        }
        matched
    }

    /// Ask the original server to cancel a tracked, nonterminal task.
    ///
    /// Refuses terminal, expired and unresolved-identity tasks. Records
    /// `cancel_requested` before sending, never marks the task cancelled
    /// itself, and reconciles with `tasks/get` afterwards.
    pub async fn cancel(&self, task_id: &str) -> Result<CancelOutcome> {
        let status = self
            .inner
            .manager
            .status(task_id)
            .await
            .ok_or_else(|| anyhow!("Task not found: {task_id}"))?;
        let Some(record) = status.mcp_task.as_ref() else {
            bail!("Task {task_id} is not an MCP task");
        };
        if matches!(record.remote_state, McpRemoteState::Unresolved(_)) {
            // Nothing can be sent: the server is unknown or is a different
            // server now. Let the user dismiss the task locally, as Lost, and
            // never claim the remote task was cancelled.
            self.inner
                .manager
                .update_mcp_task(task_id, |record| {
                    record.remote_state = McpRemoteState::NotFound;
                    true
                })
                .await?;
            return Ok(CancelOutcome::AbandonedLocally);
        }
        if matches!(
            record.remote_state,
            McpRemoteState::Expired | McpRemoteState::NotFound
        ) {
            bail!("MCP task {task_id} is expired or unknown to its server; it cannot be cancelled");
        }
        if status.status != BackgroundTaskStatus::Running || record.remote_status.is_terminal() {
            bail!(
                "MCP task {task_id} already finished ({}); it cannot be cancelled",
                record.remote_status.as_str()
            );
        }
        let resolved =
            self.inner
                .source
                .resolve(&record.server)
                .await
                .map_err(|error| match error {
                    ResolveError::Unknown => {
                        anyhow!("MCP server '{}' is not configured", record.server)
                    }
                    ResolveError::Unavailable(reason) => {
                        anyhow!("MCP server '{}' is unavailable: {reason}", record.server)
                    }
                })?;
        if resolved.identity != record.identity {
            bail!(
                "MCP server '{}' no longer matches the server that created task {task_id}; not sending the cancel request",
                record.server
            );
        }
        let remote_task_id = record.remote_task_id.clone();
        let server = record.server.clone();

        // Persist intent first: an unpersisted request must not be sent.
        self.inner
            .manager
            .update_mcp_task(task_id, |record| {
                let changed = !record.cancel_requested || record.cancel_error.is_some();
                record.cancel_requested = true;
                record.cancel_error = None;
                changed
            })
            .await?;

        let ack = resolved.remote.cancel_task(&remote_task_id).await;
        if let Err(error) = &ack {
            let reason = bounded_reason(&error.to_string());
            let stored = reason.clone();
            self.inner
                .manager
                .update_mcp_task(task_id, move |record| {
                    record.cancel_error = Some(stored);
                    true
                })
                .await?;
            self.inner
                .ensure_loop(task_id, &server, &remote_task_id, false);
            return Ok(CancelOutcome::Unacknowledged(reason));
        }

        // Reconcile with the server's own state. Its terminal outcome wins.
        let snapshot = resolved.remote.get_task(&remote_task_id).await;
        let now = self.inner.clock.now();
        let status = match snapshot {
            Ok(TaskPoll::Task(snapshot)) if snapshot.task_id == remote_task_id => {
                self.inner
                    .manager
                    .update_mcp_task(task_id, |record| apply_snapshot(record, &snapshot, now))
                    .await?
            }
            Ok(TaskPoll::NotFound(_)) => {
                self.inner
                    .manager
                    .update_mcp_task(task_id, |record| {
                        record.remote_state = McpRemoteState::NotFound;
                        true
                    })
                    .await?
            }
            _ => None,
        };
        let remote_status = status
            .and_then(|status| status.mcp_task)
            .map(|record| record.remote_status);
        Ok(match remote_status {
            Some(RemoteStatus::Cancelled) => CancelOutcome::Confirmed,
            Some(finished @ (RemoteStatus::Completed | RemoteStatus::Failed)) => {
                CancelOutcome::ServerFinished(finished)
            }
            _ => {
                self.inner
                    .ensure_loop(task_id, &server, &remote_task_id, false);
                CancelOutcome::Requested
            }
        })
    }

    /// Stop every polling loop. Persisted records are untouched, so
    /// [`Self::resume_all`] picks them up again.
    pub fn shutdown(&self) {
        for (_, entry) in self.inner.loops().drain() {
            entry.join.abort();
        }
    }

    /// Whether a polling loop is active for this background task id.
    pub fn is_polling(&self, task_id: &str) -> bool {
        self.inner.loops().contains_key(task_id)
    }
}

impl Inner {
    fn loops(&self) -> std::sync::MutexGuard<'_, HashMap<String, LoopEntry>> {
        self.loops
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Start a polling loop unless one runs. Returns whether it started one.
    fn ensure_loop(
        self: &Arc<Self>,
        task_id: &str,
        server: &str,
        remote_task_id: &str,
        immediate: bool,
    ) -> bool {
        let mut loops = self.loops();
        if loops.contains_key(task_id) {
            return false;
        }
        let generation = self.next_generation.fetch_add(1, Ordering::Relaxed);
        let wake = Arc::new(Notify::new());
        let inner = Arc::clone(self);
        let id = task_id.to_string();
        let loop_wake = Arc::clone(&wake);
        let join = tokio::spawn(async move {
            inner.run_loop(&id, &loop_wake, immediate).await;
            // Removing needs the map lock, which the spawner holds until its
            // entry is inserted, so this cannot run before the insert.
            let mut loops = inner.loops();
            if loops
                .get(&id)
                .is_some_and(|entry| entry.generation == generation)
            {
                loops.remove(&id);
                inner.changed.notify_one();
            }
        });
        loops.insert(
            task_id.to_string(),
            LoopEntry {
                generation,
                server: server.to_string(),
                remote_task_id: remote_task_id.to_string(),
                wake,
                join,
            },
        );
        self.changed.notify_one();
        true
    }

    /// The task's record while it still needs polling.
    async fn active_record(&self, task_id: &str) -> Option<McpTaskRecord> {
        let status: TaskStatusFile = self.manager.status(task_id).await?;
        if status.status != BackgroundTaskStatus::Running {
            return None;
        }
        status
            .mcp_task
            .filter(|record| !record.remote_status.is_terminal())
    }

    async fn run_loop(self: &Arc<Self>, task_id: &str, wake: &Notify, mut poll_now: bool) {
        let mut failures: u32 = 0;
        let mut last_poll: Option<DateTime<Utc>> = None;
        loop {
            let Some(mut record) = self.active_record(task_id).await else {
                return;
            };
            if self.expire_if_due(task_id, &record).await {
                return;
            }
            if poll_now {
                last_poll = Some(self.clock.now());
                match self.poll_once(task_id, &record).await {
                    Step::Stop => return,
                    Step::Backoff => failures = failures.saturating_add(1),
                    Step::Next => failures = 0,
                }
                let Some(fresh) = self.active_record(task_id).await else {
                    return;
                };
                record = fresh;
            }
            poll_now = true;
            let mut delay = poll_delay(&record, failures);
            if let Some(until) = self.time_until_expiry(&record) {
                delay = delay.min(until);
            }
            tokio::select! {
                () = self.clock.sleep(delay) => {}
                () = wake.notified() => {
                    // A hint is a hint, not a license to hammer the server:
                    // keep the minimum spacing since the last poll even when a
                    // server floods notifications.
                    if let Some(last) = last_poll {
                        let since = (self.clock.now() - last).to_std().unwrap_or_default();
                        if let Some(wait) = MIN_POLL_INTERVAL.checked_sub(since) {
                            self.clock.sleep(wait).await;
                        }
                    }
                }
            }
        }
    }

    /// Mark the task expired when its TTL has passed. Returns whether it did
    /// (or found the record already finished), so the loop stops.
    async fn expire_if_due(&self, task_id: &str, record: &McpTaskRecord) -> bool {
        if self.time_until_expiry(record) != Some(Duration::ZERO) {
            return false;
        }
        if let Err(error) = self
            .manager
            .update_mcp_task(task_id, |record| {
                record.remote_state = McpRemoteState::Expired;
                true
            })
            .await
        {
            crate::logging::warn(&format!(
                "Cannot record MCP task expiry for {task_id}: {error}"
            ));
            return false;
        }
        true
    }

    fn time_until_expiry(&self, record: &McpTaskRecord) -> Option<Duration> {
        let expires = DateTime::parse_from_rfc3339(record.expires_at.as_deref()?).ok()?;
        let remaining = expires.with_timezone(&Utc) - self.clock.now();
        Some(remaining.to_std().unwrap_or(Duration::ZERO))
    }

    async fn set_state(&self, task_id: &str, state: McpRemoteState) -> bool {
        self.manager
            .update_mcp_task(task_id, move |record| {
                if record.remote_state == state {
                    return false;
                }
                record.remote_state = state;
                true
            })
            .await
            .map(|_| ())
            .map_err(|error| {
                crate::logging::warn(&format!(
                    "Cannot persist MCP task state for {task_id}: {error}"
                ));
            })
            .is_ok()
    }

    async fn poll_once(&self, task_id: &str, record: &McpTaskRecord) -> Step {
        let resolved = match self.source.resolve(&record.server).await {
            Ok(resolved) => resolved,
            Err(ResolveError::Unknown) => {
                self.set_state(
                    task_id,
                    McpRemoteState::Unresolved(format!(
                        "server '{}' is not configured",
                        record.server
                    )),
                )
                .await;
                return Step::Stop;
            }
            Err(ResolveError::Unavailable(reason)) => {
                self.set_state(
                    task_id,
                    McpRemoteState::RemoteUnavailable(bounded_reason(&reason)),
                )
                .await;
                return Step::Backoff;
            }
        };
        // Never send the saved task id to a different server or auth context.
        if resolved.identity != record.identity {
            self.set_state(
                task_id,
                McpRemoteState::Unresolved(format!(
                    "server '{}' now has a different endpoint or authorization context",
                    record.server
                )),
            )
            .await;
            return Step::Stop;
        }
        match resolved.remote.get_task(&record.remote_task_id).await {
            Err(error) => {
                self.set_state(
                    task_id,
                    McpRemoteState::RemoteUnavailable(bounded_reason(&error.to_string())),
                )
                .await;
                Step::Backoff
            }
            Ok(TaskPoll::NotFound(_)) => {
                self.set_state(task_id, McpRemoteState::NotFound).await;
                Step::Stop
            }
            Ok(TaskPoll::Task(snapshot)) => {
                if snapshot.task_id != record.remote_task_id {
                    self.set_state(
                        task_id,
                        McpRemoteState::RemoteUnavailable(
                            "server answered for a different task id".to_string(),
                        ),
                    )
                    .await;
                    return Step::Backoff;
                }
                let now = self.clock.now();
                match self
                    .manager
                    .update_mcp_task(task_id, |record| apply_snapshot(record, &snapshot, now))
                    .await
                {
                    Ok(_) => Step::Next,
                    Err(error) => {
                        crate::logging::warn(&format!(
                            "Cannot persist MCP task snapshot for {task_id}: {error}"
                        ));
                        Step::Backoff
                    }
                }
            }
        }
    }
}

/// Delay before the next poll: the task's interval (default 5 s, clamped to
/// 1 s..5 min), doubled per consecutive transport failure up to 5 min.
fn poll_delay(record: &McpTaskRecord, failures: u32) -> Duration {
    let base = record
        .poll_interval_ms
        .map_or(DEFAULT_POLL_INTERVAL, Duration::from_millis)
        .clamp(MIN_POLL_INTERVAL, MAX_POLL_DELAY);
    if failures == 0 {
        return base;
    }
    base.saturating_mul(1u32.checked_shl(failures.min(16)).unwrap_or(u32::MAX))
        .min(MAX_POLL_DELAY)
}

fn bounded_reason(reason: &str) -> String {
    let single_line = reason.split_whitespace().collect::<Vec<_>>().join(" ");
    crate::util::truncate_str(&single_line, MAX_REASON_BYTES).to_string()
}

/// Whether `incoming` supersedes the stored snapshot. Older snapshots are
/// stale. Timestamps may have one-second precision, so an EQUAL timestamp still
/// counts when it moves the task forward to a terminal status; a terminal
/// status is never replaced by a non-terminal one. Unparseable timestamps fall
/// back to string order, which is correct for ISO 8601 in one zone.
fn supersedes(incoming: &RemoteTask, stored: &McpTaskRecord) -> bool {
    if stored.remote_status.is_terminal() {
        return false;
    }
    let order = match (
        DateTime::parse_from_rfc3339(&incoming.last_updated_at),
        DateTime::parse_from_rfc3339(&stored.last_updated_at),
    ) {
        (Ok(incoming), Ok(stored)) => incoming.cmp(&stored),
        _ => incoming.last_updated_at.cmp(&stored.last_updated_at),
    };
    match order {
        std::cmp::Ordering::Greater => true,
        std::cmp::Ordering::Equal => incoming.status.is_terminal(),
        std::cmp::Ordering::Less => false,
    }
}

/// Copy a snapshot's server-reported fields into the record and recompute the
/// local expiry. Result and error are bounded; input requests are kept only
/// while the task is `input_required`.
fn take_snapshot_fields(record: &mut McpTaskRecord, snapshot: &RemoteTask, now: DateTime<Utc>) {
    record.remote_status = snapshot.status;
    record.status_message.clone_from(&snapshot.status_message);
    record.last_updated_at.clone_from(&snapshot.last_updated_at);
    record.ttl_ms = snapshot.ttl_ms;
    record.poll_interval_ms = snapshot.poll_interval_ms;
    record.expires_at = snapshot.ttl_ms.map(|ttl| {
        let created = DateTime::parse_from_rfc3339(&record.created_at)
            .map_or(now, |created| created.with_timezone(&Utc));
        let ttl = chrono::Duration::milliseconds(i64::try_from(ttl).unwrap_or(i64::MAX / 2));
        created
            .checked_add_signed(ttl)
            .unwrap_or(DateTime::<Utc>::MAX_UTC)
            .to_rfc3339()
    });
    record.input_requests = match (snapshot.status, snapshot.input_requests.as_ref()) {
        (RemoteStatus::InputRequired, Some(requests)) => {
            Some(bound_json(requests, MCP_TASK_INPUT_CAP).0)
        }
        _ => None,
    };
    let (result, truncated) = match (snapshot.status, snapshot.result.as_ref()) {
        (RemoteStatus::Completed, Some(result)) => {
            let (value, truncated) = bound_json(result, MCP_TASK_RESULT_CAP);
            (Some(value), truncated)
        }
        _ => (None, false),
    };
    record.result = result;
    record.result_truncated = truncated;
    record.error = match (snapshot.status, snapshot.error.as_ref()) {
        (RemoteStatus::Failed, Some(error)) => Some(bound_json(error, MCP_TASK_RESULT_CAP).0),
        _ => None,
    };
}

/// Merge a `tasks/get` snapshot into the stored record. A successful answer
/// clears an `RemoteUnavailable` condition, but only a strictly newer
/// `lastUpdatedAt` changes task state, so a stale or reordered snapshot can
/// never regress a newer one.
fn apply_snapshot(record: &mut McpTaskRecord, snapshot: &RemoteTask, now: DateTime<Utc>) -> bool {
    let mut changed = false;
    if record.remote_state != McpRemoteState::Tracking {
        record.remote_state = McpRemoteState::Tracking;
        changed = true;
    }
    if supersedes(snapshot, record) {
        take_snapshot_fields(record, snapshot, now);
        changed = true;
    }
    changed
}

#[cfg(test)]
#[path = "task_tracker_tests.rs"]
mod task_tracker_tests;
