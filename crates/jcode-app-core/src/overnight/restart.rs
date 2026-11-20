//! Startup reconciliation of overnight runs whose supervisor is gone.
//!
//! A restart or crash leaves manifests `Running`/`CancelRequested` with no
//! supervisor. Reconciliation never widens authority: it only cancels, reports
//! a live owner, marks the run `Interrupted` with a reason, or (when every
//! persisted bound still admits the task) hands the run back for resumption.
//! The overnight wake time is never consulted.

use super::*;
use crate::server::session_control::{self, ControlState};

/// What reconciliation did with one stale run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reconciled {
    /// Not an active supervised run, or already handled. Nothing changed.
    Ignored,
    /// A matching live process still owns the run. Nothing changed.
    LiveOwner(u32),
    /// `CancelRequested` was settled and closed; the run never restarts.
    Cancelled,
    /// The run stays down; the reason is recorded in the event log.
    Interrupted(String),
    /// Every bound still admits the task; ownership moved to this process and
    /// `run_resumed` was recorded. The caller starts the supervisor.
    Resume,
}

/// Runs this process resumed and still supervises. Covers the gap before the
/// supervisor registers with the usage tracker.
static RESUMED: std::sync::LazyLock<std::sync::Mutex<std::collections::HashSet<String>>> =
    std::sync::LazyLock::new(Default::default);

/// Whether a live process still owns the run. This process counts only when it
/// really supervises the run now: after an exec reload it keeps its pid but the
/// supervisor task is gone, so an empty registry means no owner.
fn live_owner(manifest: &OvernightManifest) -> Option<u32> {
    let pid = manifest.process_id;
    if pid == std::process::id() {
        let supervising = usage::active_run_limits(&manifest.coordinator_session_id).is_some()
            || RESUMED
                .lock()
                .is_ok_and(|set| set.contains(&manifest.run_id));
        return supervising.then_some(pid);
    }
    if !supervisor_is_alive(manifest) {
        return None;
    }
    match manifest.supervisor_start_time {
        // Only a readable, different start time proves the pid was reused;
        // an unreadable one cannot clear a live process, so it stays an owner.
        Some(start) if crate::platform::process_start_time(pid).is_some_and(|now| now != start) => {
            None
        }
        _ => Some(pid),
    }
}

fn interrupt(manifest: &OvernightManifest, reason: String) -> Result<Reconciled> {
    let mut updated = load_manifest(&manifest.run_id).unwrap_or_else(|_| manifest.clone());
    updated.status = OvernightRunStatus::Interrupted;
    updated.completed_at = Some(Utc::now());
    updated.last_activity_at = Utc::now();
    save_manifest(&updated)?;
    if load_manifest(&manifest.run_id)?.status != OvernightRunStatus::Interrupted {
        // A concurrent cancel won; it settles the run, so do not claim otherwise.
        return Ok(Reconciled::Ignored);
    }
    record_event(
        &updated,
        "run_interrupted",
        format!("Run not resumed after restart: {reason}"),
        json!({ "reason": reason, "previous_supervisor_pid": manifest.process_id }),
        true,
    )?;
    render_review_html(&updated)?;
    Ok(Reconciled::Interrupted(reason))
}

/// Owned work of the dead supervisor that was not confirmed stopped.
fn unsettled_owned_work(manifest: &OvernightManifest) -> Vec<String> {
    session_control::store()
        .owned_records(&manifest.coordinator_session_id)
        .into_iter()
        .filter(|record| record.state != ControlState::Stopped)
        .map(|record| format!("{}: {}", record.session_id, record.state.as_str()))
        .collect()
}

/// Reconcile one run. `policy` is the current task policy (an `Err` means it
/// could not be loaded, which fails closed). `dispatch` is the verdict of the
/// execution gate for chargeable work (`ensure_task_dispatch`).
pub async fn reconcile_run(
    manifest: &OvernightManifest,
    policy: &std::result::Result<crate::config::TasksConfig, String>,
    dispatch: impl FnOnce(&OvernightManifest) -> Result<()>,
) -> Result<Reconciled> {
    if manifest.status.is_terminal() || !enforce::is_supervised(manifest) {
        return Ok(Reconciled::Ignored);
    }
    if let Some(pid) = live_owner(manifest) {
        return Ok(Reconciled::LiveOwner(pid));
    }
    if enforce::cancel_pending(manifest) {
        // Never resumes. Settlement drains whatever the dead supervisor owned.
        enforce::stop_until_settled(manifest, enforce::StopCause::Cancel).await?;
        return Ok(match load_manifest(&manifest.run_id)?.status {
            OvernightRunStatus::Cancelled => Reconciled::Cancelled,
            _ => Reconciled::Interrupted("cancellation settlement unresolved".into()),
        });
    }
    if let Some(breach) = &manifest.limit_breach {
        return interrupt(
            manifest,
            format!("limit already reached: {}", breach.message),
        );
    }
    if manifest.task_admission.is_none() {
        return interrupt(
            manifest,
            "no persisted task authorization; start a new run to authorize work".into(),
        );
    }
    let policy = match policy {
        Ok(policy) => policy,
        Err(error) => return interrupt(manifest, format!("task policy unavailable: {error}")),
    };
    let unsettled = unsettled_owned_work(manifest);
    if !unsettled.is_empty() {
        return interrupt(
            manifest,
            format!("owned work is unresolved: {}", unsettled.join(", ")),
        );
    }
    // Re-own on a candidate only; nothing is saved unless every gate passes.
    let mut candidate = manifest.clone();
    candidate.process_id = std::process::id();
    candidate.supervisor_start_time = crate::platform::process_start_time(candidate.process_id);
    candidate.heartbeat_at = Some(Utc::now());
    let Some(decision) = revalidate_task_admission(&candidate, policy) else {
        return interrupt(manifest, "task authorization missing".into());
    };
    if decision.state != crate::config::AdmissionState::Ready {
        return interrupt(
            manifest,
            format!(
                "admission is {:?}: {}",
                decision.state,
                decision.reasons.join("; ")
            ),
        );
    }
    if let Err(error) = dispatch(&candidate) {
        return interrupt(manifest, format!("{error:#}"));
    }
    save_manifest(&candidate)?;
    // A cancel that landed after the snapshot survives the save merge: it wins.
    let saved = load_manifest(&manifest.run_id)?;
    if saved.status != OvernightRunStatus::Running || saved.cancel_requested_at.is_some() {
        return Ok(Reconciled::Ignored);
    }
    record_event(
        &candidate,
        "run_resumed",
        "Resumed from the persisted manifest after restart".to_string(),
        json!({
            "previous_supervisor_pid": manifest.process_id,
            "remaining_attempts": decision.remaining_attempts,
            "remaining_runtime_seconds": decision.remaining_runtime_seconds,
            "remaining_spend_microusd": decision.remaining_spend_microusd,
        }),
        true,
    )?;
    render_review_html(&candidate)?;
    Ok(Reconciled::Resume)
}

static RECONCILING: std::sync::LazyLock<std::sync::Mutex<std::collections::HashSet<String>>> =
    std::sync::LazyLock::new(Default::default);

struct ReconcileClaim(String);

impl ReconcileClaim {
    fn take(run_id: &str) -> Option<Self> {
        RECONCILING
            .lock()
            .ok()?
            .insert(run_id.to_string())
            .then(|| Self(run_id.to_string()))
    }
}

impl Drop for ReconcileClaim {
    fn drop(&mut self) {
        if let Ok(mut set) = RECONCILING.lock() {
            set.remove(&self.0);
        }
    }
}

/// Reconcile every stale run at server start.
pub async fn reconcile_on_startup(provider: Arc<dyn Provider>) {
    let Ok(dir) = runs_dir() else { return };
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let policy = crate::config::Config::load_strict()
        .map(|config| config.tasks)
        .map_err(|error| format!("{error:#}"));
    for entry in entries.flatten() {
        let path = entry.path().join("manifest.json");
        let Ok(manifest) = admission::read_manifest_at(&path) else {
            continue;
        };
        if manifest.status.is_terminal() {
            continue;
        }
        // One reconciliation per run in this process, even if boot runs twice.
        let Some(_claim) = ReconcileClaim::take(&manifest.run_id) else {
            continue;
        };
        let provider = provider.clone();
        let policy = policy.clone();
        tokio::spawn(async move {
            let _claim = _claim;
            let outcome = reconcile_run(&manifest, &policy, |candidate| {
                let tasks = policy.as_ref().map_err(|e| anyhow::anyhow!("{e}"))?;
                ensure_task_dispatch(candidate, tasks)
            })
            .await;
            match outcome {
                Ok(Reconciled::Resume) => {
                    if let Ok(mut set) = RESUMED.lock() {
                        set.insert(manifest.run_id.clone());
                    }
                    resume(&manifest, provider).await
                }
                Ok(Reconciled::Ignored) | Ok(Reconciled::LiveOwner(_)) => {}
                Ok(other) => crate::logging::info(&format!(
                    "Overnight run {} after restart: {other:?}",
                    manifest.run_id
                )),
                Err(error) => {
                    crate::logging::warn(&format!(
                        "Overnight run {} reconciliation failed: {error:#}",
                        manifest.run_id
                    ));
                    // Never leave a dead run `Running` because a step errored.
                    let _ = interrupt(&manifest, format!("reconciliation failed: {error:#}"));
                }
            }
        });
    }
}

async fn resume(manifest: &OvernightManifest, provider: Arc<dyn Provider>) {
    let started = async {
        let current = load_manifest(&manifest.run_id)?;
        anyhow::ensure!(
            current.status == OvernightRunStatus::Running && current.cancel_requested_at.is_none(),
            "run is no longer resumable ({})",
            current.status.label()
        );
        // No route substitution: the run resumes on the route it was admitted on.
        anyhow::ensure!(
            provider.name() == current.provider_name,
            "the persisted route `{}` is not the server's active provider `{}`",
            current.provider_name,
            provider.name()
        );
        let child = Session::load(&current.coordinator_session_id)?;
        let canary = child.is_canary;
        let registry = Registry::new(provider.fork()).await;
        spawn_supervisor(current, child, provider.fork(), registry, canary);
        Result::<()>::Ok(())
    };
    if let Err(error) = started.await {
        let _ = interrupt(manifest, format!("cannot restart supervisor: {error:#}"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::overnight::enforce::tests::run_manifest;

    fn sandbox() -> crate::auth::test_sandbox::AuthTestSandbox {
        crate::auth::test_sandbox::AuthTestSandbox::new().unwrap()
    }

    fn policy() -> std::result::Result<crate::config::TasksConfig, String> {
        Ok(crate::config::TasksConfig {
            unattended: true,
            ..Default::default()
        })
    }

    /// A supervised, task-authorized run whose supervisor (`u32::MAX`) is gone
    /// and whose overnight wake time is long past.
    fn stale_run(run_id: &str) -> OvernightManifest {
        let mut manifest = run_manifest(run_id, &format!("coord-{run_id}"), Default::default());
        manifest.parent_session_id = format!("parent-{run_id}");
        manifest.process_id = u32::MAX;
        manifest.target_wake_at = Utc::now() - ChronoDuration::hours(30);
        manifest.working_dir = Some(storage::jcode_dir().unwrap().to_string_lossy().into());
        manifest.task_admission = Some(crate::config::TaskAdmissionSnapshot {
            scope: "bounded task".into(),
            authorization: Some(crate::config::TaskAuthorization {
                scope: "bounded task".into(),
                max_attempts: 2,
                max_runtime_seconds: u64::MAX / 2,
                max_spend_microusd: 100,
            }),
            ownership_verified: true,
            dependencies_ready: true,
            workspace_ready: true,
            ..Default::default()
        });
        storage::write_json(&manifest_path(run_id).unwrap(), &manifest).unwrap();
        manifest
    }

    fn kinds(manifest: &OvernightManifest) -> Vec<String> {
        read_events(manifest)
            .unwrap()
            .into_iter()
            .map(|event| event.kind)
            .collect()
    }

    #[tokio::test]
    async fn eligible_authorized_task_resumes_without_an_overnight_window() {
        let _sandbox = sandbox();
        let manifest = stale_run("rst-eligible");
        assert!(manifest.target_wake_at < Utc::now());
        let outcome = reconcile_run(&manifest, &policy(), |_| Ok(()))
            .await
            .unwrap();
        assert_eq!(outcome, Reconciled::Resume);
        let current = load_manifest("rst-eligible").unwrap();
        assert_eq!(current.status, OvernightRunStatus::Running);
        assert_eq!(current.process_id, std::process::id());
        assert!(current.heartbeat_at.is_some());
        assert!(kinds(&current).contains(&"run_resumed".to_string()));
    }

    #[tokio::test]
    async fn cancel_requested_is_cancelled_and_never_resumed() {
        let _sandbox = sandbox();
        let mut manifest = stale_run("rst-cancel");
        manifest.status = OvernightRunStatus::CancelRequested;
        manifest.cancel_requested_at = Some(Utc::now());
        save_manifest(&manifest).unwrap();
        let outcome = reconcile_run(&manifest, &policy(), |_| {
            panic!("a cancelled run must never reach dispatch")
        })
        .await
        .unwrap();
        assert_eq!(outcome, Reconciled::Cancelled);
        let current = load_manifest("rst-cancel").unwrap();
        assert_eq!(current.status, OvernightRunStatus::Cancelled);
        assert!(!kinds(&current).contains(&"run_resumed".to_string()));
    }

    #[tokio::test]
    async fn live_owner_is_reported_and_never_duplicated() {
        let _sandbox = sandbox();
        let mut manifest = stale_run("rst-live");
        // Pid 1 is alive and is not this process.
        manifest.process_id = 1;
        manifest.supervisor_start_time = crate::platform::process_start_time(1);
        save_manifest(&manifest).unwrap();
        let before = std::fs::read(manifest_path("rst-live").unwrap()).unwrap();
        let outcome = reconcile_run(&manifest, &policy(), |_| {
            panic!("a live owner must not be duplicated")
        })
        .await
        .unwrap();
        assert_eq!(outcome, Reconciled::LiveOwner(1));
        assert_eq!(
            std::fs::read(manifest_path("rst-live").unwrap()).unwrap(),
            before
        );

        // A pid reused by an unrelated process (start time differs) is not the owner.
        if manifest.supervisor_start_time.is_some() {
            manifest.supervisor_start_time = manifest.supervisor_start_time.map(|t| t + 1);
            save_manifest(&manifest).unwrap();
            let outcome = reconcile_run(&manifest, &policy(), |_| Ok(()))
                .await
                .unwrap();
            assert_eq!(outcome, Reconciled::Resume);
        }
    }

    #[tokio::test]
    async fn policy_rejection_stays_interrupted_with_a_reason() {
        let _sandbox = sandbox();
        let manifest = stale_run("rst-policy");
        let disabled = Ok(crate::config::TasksConfig::default());
        let outcome = reconcile_run(&manifest, &disabled, |_| Ok(()))
            .await
            .unwrap();
        let Reconciled::Interrupted(reason) = outcome else {
            panic!("expected interrupted, got {outcome:?}");
        };
        assert!(!reason.is_empty());
        let current = load_manifest("rst-policy").unwrap();
        assert_eq!(current.status, OvernightRunStatus::Interrupted);
        assert!(kinds(&current).contains(&"run_interrupted".to_string()));
        assert!(!kinds(&current).contains(&"run_resumed".to_string()));

        // An unloadable policy and a denied dispatch gate also fail closed.
        let manifest = stale_run("rst-policy-err");
        let outcome = reconcile_run(&manifest, &Err("bad config".into()), |_| Ok(()))
            .await
            .unwrap();
        assert!(matches!(outcome, Reconciled::Interrupted(r) if r.contains("bad config")));
        let manifest = stale_run("rst-gate");
        let outcome = reconcile_run(&manifest, &policy(), |_| {
            anyhow::bail!("no monetary capability")
        })
        .await
        .unwrap();
        assert!(matches!(outcome, Reconciled::Interrupted(r) if r.contains("monetary")));
        assert_eq!(
            load_manifest("rst-gate").unwrap().status,
            OvernightRunStatus::Interrupted
        );
    }

    #[tokio::test]
    async fn broader_config_cannot_widen_prior_authorization() {
        let _sandbox = sandbox();
        let mut manifest = stale_run("rst-widen");
        let snapshot = manifest.task_admission.as_mut().unwrap();
        snapshot.attempts = 2; // the persisted authorization allows 2
        save_manifest(&manifest).unwrap();
        let wide = Ok(crate::config::TasksConfig {
            unattended: true,
            max_attempts: 1000,
            max_spend_microusd: u64::MAX / 2,
            ..Default::default()
        });
        let outcome = reconcile_run(&manifest, &wide, |_| Ok(())).await.unwrap();
        assert!(matches!(outcome, Reconciled::Interrupted(_)), "{outcome:?}");
        let current = load_manifest("rst-widen").unwrap();
        assert_eq!(current.status, OvernightRunStatus::Interrupted);
        assert_eq!(
            current
                .task_admission
                .unwrap()
                .authorization
                .unwrap()
                .max_attempts,
            2,
            "the persisted authorization is never rewritten"
        );
    }

    #[tokio::test]
    async fn legacy_run_without_task_authority_is_interrupted_not_resumed() {
        let _sandbox = sandbox();
        let mut manifest = stale_run("rst-legacy");
        manifest.task_admission = None;
        // A save cannot drop persisted authority, so write the legacy shape directly.
        storage::write_json(&manifest_path("rst-legacy").unwrap(), &manifest).unwrap();
        let outcome = reconcile_run(&manifest, &policy(), |_| Ok(()))
            .await
            .unwrap();
        assert!(matches!(outcome, Reconciled::Interrupted(_)), "{outcome:?}");
    }

    #[tokio::test]
    async fn interrupted_run_cannot_be_revived_by_a_stale_save() {
        let _sandbox = sandbox();
        let manifest = stale_run("rst-revive");
        let disabled = Ok(crate::config::TasksConfig::default());
        reconcile_run(&manifest, &disabled, |_| Ok(()))
            .await
            .unwrap();
        assert_eq!(
            load_manifest("rst-revive").unwrap().status,
            OvernightRunStatus::Interrupted
        );
        // A supervisor still holding the pre-restart `Running` snapshot saves.
        save_manifest(&manifest).unwrap();
        assert_eq!(
            load_manifest("rst-revive").unwrap().status,
            OvernightRunStatus::Interrupted,
            "a stale Running save must not revive an interrupted run"
        );
    }

    #[tokio::test]
    async fn a_concurrent_cancel_wins_over_the_interrupt() {
        let _sandbox = sandbox();
        let manifest = stale_run("rst-race");
        let mut cancelled = manifest.clone();
        cancelled.status = OvernightRunStatus::CancelRequested;
        cancelled.cancel_requested_at = Some(Utc::now());
        save_manifest(&cancelled).unwrap();
        // `manifest` is the stale pre-cancel snapshot reconciliation started from.
        let outcome = interrupt(&manifest, "test".into()).unwrap();
        assert_eq!(outcome, Reconciled::Ignored);
        let current = load_manifest("rst-race").unwrap();
        assert_eq!(current.status, OvernightRunStatus::CancelRequested);
        assert!(!kinds(&current).contains(&"run_interrupted".to_string()));
    }

    #[tokio::test]
    async fn only_one_reconcile_claim_per_run() {
        let first = ReconcileClaim::take("rst-claim").expect("first claim");
        assert!(ReconcileClaim::take("rst-claim").is_none());
        drop(first);
        assert!(ReconcileClaim::take("rst-claim").is_some());
    }

    #[test]
    fn own_pid_owns_only_while_it_really_supervises_and_unreadable_start_stays_owner() {
        let mut manifest = run_manifest_for_owner();
        manifest.process_id = std::process::id();
        // Exec reload: same pid, no supervisor task registered.
        assert_eq!(live_owner(&manifest), None);
        RESUMED.lock().unwrap().insert(manifest.run_id.clone());
        assert_eq!(live_owner(&manifest), Some(std::process::id()));
        RESUMED.lock().unwrap().remove(&manifest.run_id);
        manifest.process_id = 1;
        manifest.supervisor_start_time = None;
        assert_eq!(live_owner(&manifest), Some(1));
    }

    #[tokio::test]
    async fn a_cancel_landing_during_resume_wins_and_records_no_resume() {
        let _sandbox = sandbox();
        let manifest = stale_run("rst-resume-cancel");
        let outcome = reconcile_run(&manifest, &policy(), |_| {
            // Cancel arrives between the snapshot and the ownership save.
            let mut cancelled = load_manifest("rst-resume-cancel").unwrap();
            cancelled.status = OvernightRunStatus::CancelRequested;
            cancelled.cancel_requested_at = Some(Utc::now());
            save_manifest(&cancelled).unwrap();
            Ok(())
        })
        .await
        .unwrap();
        assert_eq!(outcome, Reconciled::Ignored);
        let current = load_manifest("rst-resume-cancel").unwrap();
        assert_eq!(current.status, OvernightRunStatus::CancelRequested);
        assert!(!kinds(&current).contains(&"run_resumed".to_string()));
    }

    fn run_manifest_for_owner() -> OvernightManifest {
        let _sandbox = sandbox();
        stale_run("rst-owner")
    }
}
