//! Mid-turn cancellation and limit enforcement for the overnight supervisor.
//!
//! Cancellation and limits are observed while a coordinator turn is running,
//! not only between turns. Stopping goes through durable session control:
//! admission is revoked first, owned work is drained, and the run is recorded
//! as `Cancelled` only when every owned session is confirmed `stopped`. A drain
//! deadline that elapses leaves the run `CancelRequested` with an explicit
//! unresolved settlement instead of claiming success.
//!
//! Token caps read the deduplicated usage ledger after responses. They can
//! stop a run that is over its bound; they never authorize dispatch, so
//! `ensure_task_dispatch` and provider-side monetary reservations still decide
//! whether a request may be sent.

use super::*;
use crate::server::session_control::{self, ControlState};
use jcode_overnight_core::{
    LimitBreach, RunTokenUsage, StopSettlement, aggregate_run_tokens, check_agent_spawn,
    check_run_tokens, check_turn_time,
};
use std::collections::{BTreeMap, HashSet};
use std::sync::LazyLock;

#[cfg(not(test))]
const STOP_POLL_INTERVAL: Duration = Duration::from_secs(1);
#[cfg(test)]
const STOP_POLL_INTERVAL: Duration = Duration::from_millis(25);
/// How long an aborted turn may wind down on its own before it is dropped.
#[cfg(not(test))]
const TURN_ABORT_GRACE: Duration = Duration::from_secs(5);
#[cfg(test)]
const TURN_ABORT_GRACE: Duration = Duration::from_millis(100);
/// Matches the daemon's `swarm stop` drain deadline.
#[cfg(not(test))]
const SETTLE_DEADLINE: Duration = Duration::from_secs(25);
#[cfg(test)]
const SETTLE_DEADLINE: Duration = Duration::from_millis(300);

/// Re-settle attempts after an unresolved stop, and the wait between them.
const RESETTLE_ATTEMPTS: usize = 5;
#[cfg(not(test))]
const RESETTLE_INTERVAL: Duration = Duration::from_secs(60);
#[cfg(test)]
const RESETTLE_INTERVAL: Duration = Duration::from_millis(50);
/// Consecutive unreadable-manifest polls before the failure is recorded.
const UNREADABLE_POLLS_BEFORE_RECORD: u32 = 5;
/// Real seconds counted per elapsed turn second. Tests compress a "minute" to
/// one second so a hung turn can be interrupted on the time limit quickly.
#[cfg(not(test))]
const TURN_TIME_SCALE: i64 = 1;
#[cfg(test)]
const TURN_TIME_SCALE: i64 = 60;

/// Visible current-session runs have no supervisor, so the turn-time and token
/// caps cannot be enforced there. Reject them instead of accepting a bound that
/// would be silently ignored. `max_agents` still applies through the spawn gate.
pub fn visible_mode_limit_rejection(limits: &OvernightLimits) -> Option<String> {
    (limits.max_turn_minutes.is_some() || limits.max_run_tokens.is_some()).then(|| {
        "--max-turn-minutes and --max-tokens are only enforced for supervised runs (started with `/overnight` through the server); the visible current-session run has no supervisor to stop a turn. Only --max-agents applies here.".to_string()
    })
}

/// Why a run is being stopped.
#[derive(Debug, Clone)]
pub(super) enum StopCause {
    Cancel,
    Limit(LimitBreach),
}

pub(super) enum TurnOutcome {
    Finished(String),
    Stopped(StopCause),
}

/// Aggregate run-owned usage from the deduplicated ledger.
pub(super) fn run_token_usage(manifest: &OvernightManifest) -> RunTokenUsage {
    let (observations, unreadable) = usage::read_ledger(&manifest.run_dir);
    aggregate_run_tokens(&manifest.run_id, &observations, unreadable)
}

pub(super) fn cancel_pending(manifest: &OvernightManifest) -> bool {
    matches!(manifest.status, OvernightRunStatus::CancelRequested)
        || manifest.cancel_requested_at.is_some()
}

/// A supervised run has a coordinator session separate from the session that
/// started it. A visible current-session run uses one session for both and has
/// no supervisor, so only spawn refusal applies to it.
pub fn is_supervised(manifest: &OvernightManifest) -> bool {
    manifest.parent_session_id != manifest.coordinator_session_id
}

/// The reason this run must stop now, if any. `turn_started` is `None` between
/// turns, where only cancellation and the token cap apply.
pub(super) fn pending_stop(
    manifest: &OvernightManifest,
    turn_started: Option<chrono::DateTime<Utc>>,
) -> Option<StopCause> {
    if cancel_pending(manifest) {
        return Some(StopCause::Cancel);
    }
    // A recorded breach is sticky: an unresolved limit stop keeps stopping.
    if let Some(breach) = &manifest.limit_breach {
        return Some(StopCause::Limit(breach.clone()));
    }
    let limits = &manifest.limits;
    if let Some(started) = turn_started {
        let elapsed = Utc::now().signed_duration_since(started).num_seconds() * TURN_TIME_SCALE;
        if let Some(breach) = check_turn_time(limits, elapsed.max(0) as u64) {
            return Some(StopCause::Limit(breach));
        }
    }
    if limits.max_run_tokens.is_some()
        && let Some(breach) = check_run_tokens(limits, &run_token_usage(manifest))
    {
        return Some(StopCause::Limit(breach));
    }
    None
}

/// Revoke admission for the coordinator and every session it owns, and fire
/// their in-flight turn cancel signals. Returns the targeted session ids and
/// any that could not be stopped. Idempotent: only runnable sessions change.
fn revoke_admission(manifest: &OvernightManifest) -> (Vec<String>, Vec<String>) {
    let store = session_control::store();
    let coordinator = manifest.coordinator_session_id.as_str();
    let request = format!("overnight:{}", manifest.run_id);
    let mut ids = vec![coordinator.to_string()];
    ids.extend(
        store
            .owned_records(coordinator)
            .into_iter()
            .map(|record| record.session_id),
    );
    let mut problems = Vec::new();
    let mut controlled = Vec::new();
    for id in ids {
        match store.inspect(&id) {
            Ok(None) => continue,
            Ok(Some(record)) => {
                if record.state == ControlState::Runnable
                    && let Err(error) = store.request_stop(coordinator, &id, false, &request)
                {
                    problems.push(format!("{id}: stop not requested: {error:#}"));
                }
            }
            Err(error) => problems.push(format!("{id}: unreadable control record: {error:#}")),
        }
        for signal in crate::turn_cancel_registry::active_turn_signals(&id) {
            signal.fire();
        }
        controlled.push(id);
    }
    (controlled, problems)
}

/// Drain every owned session and report whether all were confirmed stopped.
/// Owned sessions are re-enumerated until no new one appears, so a worker that
/// registered after the first pass is still revoked and drained.
pub(super) async fn settle_owned_work(manifest: &OvernightManifest) -> StopSettlement {
    // Governed spawns already past the gate finish registering before the
    // final enumeration; later ones are refused because the run is stopping.
    drop(SPAWN_GATE.lock().await);
    let store = session_control::store();
    let mut known: HashSet<String> = HashSet::new();
    let mut problems: BTreeMap<String, String> = BTreeMap::new();
    loop {
        let (ids, revoke_problems) = revoke_admission(manifest);
        for problem in revoke_problems {
            let (id, detail) = problem.split_once(": ").unwrap_or((problem.as_str(), ""));
            problems.insert(id.to_string(), detail.to_string());
        }
        let fresh: Vec<String> = ids
            .into_iter()
            .filter(|id| known.insert(id.clone()))
            .collect();
        if fresh.is_empty() {
            break;
        }
        let drains = fresh.iter().map(|id| {
            let store = store.clone();
            async move { (id.clone(), store.drain(id, SETTLE_DEADLINE).await) }
        });
        for (id, outcome) in futures::future::join_all(drains).await {
            match outcome {
                Ok(record) if record.state == ControlState::Stopped => {
                    problems.remove(&id);
                }
                Ok(record) => {
                    problems.insert(id, record.state.as_str().to_string());
                }
                Err(error) => {
                    problems.insert(id, format!("{error:#}"));
                }
            }
        }
    }
    let at = Utc::now();
    if problems.is_empty() {
        StopSettlement::Settled { at }
    } else {
        StopSettlement::Unresolved {
            at,
            sessions: problems
                .into_iter()
                .map(|(id, detail)| format!("{id}: {detail}"))
                .collect(),
            reason: format!(
                "owned work was not confirmed stopped within {}s",
                SETTLE_DEADLINE.as_secs()
            ),
        }
    }
}

/// Record a limit breach on the manifest before admission is revoked, so the
/// spawn gate and later polls see the stop. Idempotent; the `limit_reached`
/// event is emitted once.
fn persist_limit_breach(manifest: &OvernightManifest, breach: &LimitBreach) -> Result<()> {
    let mut updated = load_manifest(&manifest.run_id).unwrap_or_else(|_| manifest.clone());
    if updated.limit_breach.is_some() {
        return Ok(());
    }
    updated.limit_breach = Some(breach.clone());
    updated.last_activity_at = Utc::now();
    save_manifest(&updated)?;
    record_event(
        &updated,
        "limit_reached",
        breach.message.clone(),
        json!({ "limits": updated.limits, "limit_breach": breach }),
        true,
    )
}

/// Stop the run for `cause`, settle owned work, and record the truthful end
/// state. Returns whether owned work was confirmed stopped.
pub(super) async fn stop_run(manifest: &OvernightManifest, cause: StopCause) -> Result<bool> {
    match &cause {
        StopCause::Cancel => record_event(
            manifest,
            "run_cancel_acknowledged",
            "Cancellation observed; revoking admission and draining owned work".to_string(),
            json!({ "limits": manifest.limits }),
            true,
        )?,
        StopCause::Limit(breach) => persist_limit_breach(manifest, breach)?,
    }
    settle_and_finish(manifest, &cause).await
}

async fn settle_and_finish(manifest: &OvernightManifest, cause: &StopCause) -> Result<bool> {
    let settlement = settle_owned_work(manifest).await;
    let settled = settlement.is_settled();
    finish_stopped(manifest, cause, settlement).await?;
    Ok(settled)
}

/// `stop_run`, then bounded re-settles while owned work is unresolved. A run
/// that never settles stays non-terminal and can be cancelled again later.
pub(super) async fn stop_until_settled(
    manifest: &OvernightManifest,
    mut cause: StopCause,
) -> Result<()> {
    let mut settled = stop_run(manifest, cause.clone()).await?;
    for _ in 0..RESETTLE_ATTEMPTS {
        if settled {
            break;
        }
        tokio::time::sleep(RESETTLE_INTERVAL).await;
        let current = load_manifest(&manifest.run_id)?;
        if current.status.is_terminal() {
            break;
        }
        // A user cancel issued while a limit stop was unresolved wins.
        if cancel_pending(&current) {
            cause = StopCause::Cancel;
        }
        settled = settle_and_finish(&current, &cause).await?;
    }
    Ok(())
}

static RESETTLING: LazyLock<std::sync::Mutex<HashSet<String>>> = LazyLock::new(Default::default);

/// Re-run cancellation settlement for a supervised run whose supervisor is no
/// longer active in this process (for example after an unresolved stop and a
/// repeated `/overnight cancel`). Returns false when nothing was started.
pub fn resettle_in_background(manifest: &OvernightManifest) -> bool {
    if manifest.status.is_terminal()
        || !is_supervised(manifest)
        || (manifest.process_id != std::process::id() && supervisor_is_alive(manifest))
        || usage::active_run_limits(&manifest.coordinator_session_id).is_some()
        || tokio::runtime::Handle::try_current().is_err()
    {
        return false;
    }
    let inserted = RESETTLING
        .lock()
        .map(|mut set| set.insert(manifest.run_id.clone()))
        .unwrap_or(false);
    if !inserted {
        return false;
    }
    let manifest = manifest.clone();
    tokio::spawn(async move {
        if let Err(error) = stop_run(&manifest, StopCause::Cancel).await {
            crate::logging::warn(&format!("Overnight re-settle failed: {error:#}"));
        }
        if let Ok(mut set) = RESETTLING.lock() {
            set.remove(&manifest.run_id);
        }
    });
    true
}

async fn finish_stopped(
    manifest: &OvernightManifest,
    cause: &StopCause,
    settlement: StopSettlement,
) -> Result<()> {
    let mut updated = load_manifest(&manifest.run_id).unwrap_or_else(|_| manifest.clone());
    let unresolved = match &settlement {
        StopSettlement::Settled { .. } => None,
        StopSettlement::Unresolved {
            sessions, reason, ..
        } => Some(format!("{reason}: {}", sessions.join(", "))),
    };
    let (kind, summary) = match (cause, &unresolved) {
        (StopCause::Cancel, None) => {
            updated.status = OvernightRunStatus::Cancelled;
            updated.completed_at = Some(Utc::now());
            (
                "run_cancelled",
                "Cancelled; owned work confirmed stopped".to_string(),
            )
        }
        // No terminal claim while owned work may still be running.
        (StopCause::Cancel, Some(detail)) => (
            "run_cancel_unresolved",
            format!("Cancellation requested but not settled: {detail}"),
        ),
        (StopCause::Limit(breach), None) => {
            updated.status = OvernightRunStatus::Failed;
            updated.completed_at = Some(Utc::now());
            updated.limit_breach = Some(breach.clone());
            (
                "run_limit_stopped",
                format!("Stopped by limit: {}", breach.message),
            )
        }
        (StopCause::Limit(breach), Some(detail)) => {
            updated.limit_breach = Some(breach.clone());
            (
                "run_limit_unresolved",
                format!(
                    "Stopped by limit ({}) but owned work is not confirmed stopped: {detail}",
                    breach.message
                ),
            )
        }
    };
    updated.stop_settlement = Some(settlement.clone());
    updated.last_activity_at = Utc::now();
    save_manifest(&updated)?;
    record_event(
        &updated,
        kind,
        summary,
        json!({
            "settlement": settlement,
            "limit_breach": updated.limit_breach,
            "usage_reconciled": unresolved.is_none(),
        }),
        true,
    )?;
    // Usage is reconciled only once owned work is settled; reconciling earlier
    // would cache a total that later worker usage could not update.
    if unresolved.is_none() {
        usage::finalize(&updated).await;
    }
    render_review_html(&updated)?;
    Ok(())
}

/// Run one coordinator turn, sampling resources and watching for cancellation
/// and limits. On a stop the turn is revoked, given a short grace, then dropped.
pub(super) async fn run_turn_monitored(
    agent: &mut Agent,
    manifest: &OvernightManifest,
    prompt: &str,
) -> Result<TurnOutcome> {
    let started = Utc::now();
    // Never dispatch a provider request for a run that is already stopping.
    if let Some(cause) = pending_stop(&load_manifest(&manifest.run_id)?, None) {
        return Ok(TurnOutcome::Stopped(cause));
    }
    let shutdown = agent.graceful_shutdown_signal();
    let mut sample_interval = tokio::time::interval_at(
        tokio::time::Instant::now() + RESOURCE_SAMPLE_INTERVAL,
        RESOURCE_SAMPLE_INTERVAL,
    );
    sample_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let mut long_notice_interval = tokio::time::interval_at(
        tokio::time::Instant::now() + LONG_TURN_NOTICE_INTERVAL,
        LONG_TURN_NOTICE_INTERVAL,
    );
    long_notice_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let mut stop_poll = tokio::time::interval(STOP_POLL_INTERVAL);
    stop_poll.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let mut unreadable_polls = 0u32;

    let run_future = agent.run_once_capture(prompt);
    tokio::pin!(run_future);

    loop {
        tokio::select! {
            result = &mut run_future => return result.map(TurnOutcome::Finished),
            _ = stop_poll.tick() => {
                let current = match load_manifest(&manifest.run_id) {
                    Ok(current) => {
                        unreadable_polls = 0;
                        current
                    }
                    Err(error) => {
                        // Cancellation and limits cannot be observed while the
                        // manifest is unreadable; say so instead of staying silent.
                        unreadable_polls += 1;
                        if unreadable_polls == UNREADABLE_POLLS_BEFORE_RECORD {
                            let _ = record_event(
                                manifest,
                                "manifest_unreadable",
                                format!(
                                    "Run manifest unreadable for {unreadable_polls} polls; cancellation and limits cannot be observed: {error:#}"
                                ),
                                json!({ "polls": unreadable_polls }),
                                true,
                            );
                        }
                        continue;
                    }
                };
                let Some(cause) = pending_stop(&current, Some(started)) else { continue };
                // Persist a limit breach first so no worker spawns while the
                // run is being revoked.
                if let StopCause::Limit(breach) = &cause {
                    let _ = persist_limit_breach(&current, breach);
                }
                revoke_admission(&current);
                shutdown.fire();
                // Let the turn wind down on its own briefly, then drop it so the
                // provider stream and any tool futures are aborted.
                let _ = tokio::time::timeout(TURN_ABORT_GRACE, &mut run_future).await;
                return Ok(TurnOutcome::Stopped(cause));
            }
            _ = sample_interval.tick() => {
                let snapshot = gather_resource_snapshot(manifest.working_dir.as_deref().map(Path::new));
                let _ = record_event(
                    manifest,
                    "resource_sample",
                    resource_summary(&snapshot),
                    serde_json::to_value(&snapshot).unwrap_or_else(|_| json!({})),
                    false,
                );
                let _ = render_review_html(manifest);
            }
            _ = long_notice_interval.tick() => {
                let elapsed = Utc::now().signed_duration_since(started).num_minutes().max(0);
                let _ = record_event(
                    manifest,
                    "coordinator_turn_still_running",
                    format!("Coordinator turn still running after {}m", elapsed),
                    json!({ "elapsed_minutes": elapsed }),
                    true,
                );
                let _ = render_review_html(manifest);
            }
        }
    }
}

/// Serializes limited worker spawns so two concurrent spawn requests cannot
/// both observe room under `max_agents`.
static SPAWN_GATE: LazyLock<Arc<tokio::sync::Mutex<()>>> = LazyLock::new(Default::default);

/// The live, non-terminal run `requester` coordinates. Its spawns are always
/// serialized through the gate; refusal depends on the cap or a pending stop.
/// A cancelled visible run has no supervisor to settle it, so it stays
/// `CancelRequested` forever; it stops governing (cap and refusal) once
/// cancelled instead of blocking the session's spawns until the process exits.
fn governing_run(requester: &str) -> Option<OvernightManifest> {
    let active =
        usage::active_run_limits(requester).and_then(|(run_id, _)| load_manifest(&run_id).ok());
    // Visible-session runs have no supervisor registered in this process.
    let manifest = active.or_else(|| {
        session_manifests_newest_first(requester)
            .ok()?
            .into_iter()
            .find(|manifest| {
                !manifest.status.is_terminal()
                    && supervisor_is_alive(manifest)
                    && (is_supervised(manifest) || !cancel_pending(manifest))
            })
    })?;
    (!manifest.status.is_terminal()).then_some(manifest)
}

/// `Some(guard)` when `requester` coordinates a live run. Hold the guard while
/// counting live workers and spawning.
pub async fn worker_spawn_gate(requester: &str) -> Option<tokio::sync::OwnedMutexGuard<()>> {
    governing_run(requester)?;
    Some(Arc::clone(&SPAWN_GATE).lock_owned().await)
}

/// Refuse a spawn beyond the run's `max_agents`, or after cancellation was
/// requested, with a visible reason.
pub fn ensure_worker_spawn_allowed(requester: &str, live_workers: usize) -> Result<()> {
    let Some(manifest) = governing_run(requester) else {
        return Ok(());
    };
    let reason = if cancel_pending(&manifest) {
        "Overnight run is being cancelled; no new workers may start.".to_string()
    } else if let Some(breach) = &manifest.limit_breach {
        format!(
            "Overnight run was stopped by a limit ({}); no new workers may start.",
            breach.message
        )
    } else if let Some(breach) = check_agent_spawn(&manifest.limits, live_workers) {
        breach.message
    } else {
        return Ok(());
    };
    let _ = record_event(
        &manifest,
        "worker_spawn_refused",
        reason.clone(),
        json!({ "live_workers": live_workers, "limits": manifest.limits }),
        true,
    );
    anyhow::bail!(reason)
}

#[cfg(test)]
#[path = "enforce_tests.rs"]
pub(super) mod tests;
