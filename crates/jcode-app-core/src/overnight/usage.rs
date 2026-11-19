//! Per-run usage ledger and run-end reconciliation.
//!
//! Usage is observed after each provider response. It never reserves spend and
//! never gates dispatch; that stays with the admission and enforcement paths.
//! Attribution follows the durable session-control owner chain, so resumed and
//! run-owned swarm workers count toward the run that owns them, tagged with
//! the execution generation that produced them.

use super::*;
use crate::server::session_control::{self, ControlRecord, ControlState};
use jcode_overnight_core::{
    ReconcileInput, RunSessionState, UsageObservation, UsageReconciled, reconcile_usage,
    usage_reconciled_summary,
};
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

const LEDGER_FILE: &str = "usage.jsonl";
const RECONCILED_FILE: &str = "usage-reconciled.json";
const END_LIMIT_FETCH_TIMEOUT: Duration = Duration::from_secs(20);
const MAX_OWNER_HOPS: usize = 16;

#[derive(Clone)]
struct ActiveRun {
    run_id: String,
    run_dir: PathBuf,
    limits: jcode_overnight_core::OvernightLimits,
}

/// Coordinator session id to its run, for supervisors alive in this process.
static ACTIVE: LazyLock<Mutex<HashMap<String, ActiveRun>>> = LazyLock::new(Default::default);
/// Serving OAuth account per session, published only while a run is active.
static ACCOUNTS: LazyLock<Mutex<HashMap<String, String>>> = LazyLock::new(Default::default);

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|e| e.into_inner())
}

pub(super) struct ActiveRunGuard(String);

impl Drop for ActiveRunGuard {
    fn drop(&mut self) {
        let mut active = lock(&ACTIVE);
        active.remove(&self.0);
        if active.is_empty() {
            lock(&ACCOUNTS).clear();
        }
    }
}

pub(super) fn register_run(manifest: &OvernightManifest) -> ActiveRunGuard {
    lock(&ACTIVE).insert(
        manifest.coordinator_session_id.clone(),
        ActiveRun {
            run_id: manifest.run_id.clone(),
            run_dir: manifest.run_dir.clone(),
            limits: manifest.limits,
        },
    );
    ActiveRunGuard(manifest.coordinator_session_id.clone())
}

/// Record which saved account serves `session_id`. No-op without an active run.
pub(crate) fn note_session_account(session_id: &str, label: Option<&str>) {
    if lock(&ACTIVE).is_empty() {
        return;
    }
    let mut accounts = lock(&ACCOUNTS);
    match label {
        Some(label) => accounts.insert(session_id.to_string(), label.to_string()),
        None => accounts.remove(session_id),
    };
}

/// Run id and limits of the run `session_id` coordinates in this process.
pub(super) fn active_run_limits(
    session_id: &str,
) -> Option<(String, jcode_overnight_core::OvernightLimits)> {
    lock(&ACTIVE)
        .get(session_id)
        .map(|run| (run.run_id.clone(), run.limits))
}

fn owning_run(session_id: &str) -> Option<ActiveRun> {
    {
        let active = lock(&ACTIVE);
        if active.is_empty() {
            return None;
        }
        if let Some(run) = active.get(session_id) {
            return Some(run.clone());
        }
    }
    let store = session_control::store();
    let mut current = session_id.to_string();
    for _ in 0..MAX_OWNER_HOPS {
        let owner = store.inspect(&current).ok().flatten()?.owner_session_id?;
        if let Some(run) = lock(&ACTIVE).get(&owner) {
            return Some(run.clone());
        }
        current = owner;
    }
    None
}

/// Called once per observed provider response by the agent stream recorder.
/// `event_id` is unique per response attempt (cumulative snapshots share it).
/// Providers do not surface a stable response id on `StreamEvent::TokenUsage`,
/// so only a repeated append of the same id dedupes; see design.md.
pub(crate) fn observe_stream_usage(
    event_id: &str,
    session_id: &str,
    provider: &str,
    model: &str,
    [input, output, cache_read, cache_creation]: [Option<u64>; 4],
) {
    if input.is_none() && output.is_none() && cache_read.is_none() && cache_creation.is_none() {
        return;
    }
    let Some(run) = owning_run(session_id) else {
        return;
    };
    let observation = UsageObservation {
        event_id: event_id.to_string(),
        run_id: run.run_id,
        session_id: session_id.to_string(),
        generation: session_control::current_generation(session_id),
        provider: provider.to_string(),
        account: lock(&ACCOUNTS).get(session_id).cloned(),
        model: model.to_string(),
        input_tokens: input,
        output_tokens: output,
        cache_read_input_tokens: cache_read,
        cache_creation_input_tokens: cache_creation,
        observed_at: Utc::now(),
    };
    if let Err(error) = append_observation(&run.run_dir, &observation) {
        crate::logging::warn(&format!("Overnight usage not recorded: {error:#}"));
    }
}

fn append_observation(run_dir: &Path, observation: &UsageObservation) -> Result<()> {
    std::fs::create_dir_all(run_dir)?;
    let mut line = serde_json::to_string(observation)?;
    line.push('\n');
    // One O_APPEND write per record keeps concurrent workers' lines whole.
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(run_dir.join(LEDGER_FILE))?
        .write_all(line.as_bytes())?;
    Ok(())
}

/// Parsed observations plus the count of non-empty lines that could not be read.
pub(super) fn read_ledger(run_dir: &Path) -> (Vec<UsageObservation>, usize) {
    let text = std::fs::read_to_string(run_dir.join(LEDGER_FILE)).unwrap_or_default();
    let mut skipped = 0;
    let mut observations = Vec::new();
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        match serde_json::from_str(line) {
            Ok(observation) => observations.push(observation),
            Err(_) => skipped += 1,
        }
    }
    (observations, skipped)
}

pub fn reconciled_path(manifest: &OvernightManifest) -> PathBuf {
    manifest.run_dir.join(RECONCILED_FILE)
}

pub fn read_usage_reconciled(manifest: &OvernightManifest) -> Option<UsageReconciled> {
    storage::read_json(&reconciled_path(manifest)).ok()
}

fn session_state(record: &ControlRecord) -> RunSessionState {
    RunSessionState {
        session_id: record.session_id.clone(),
        control_state: record.state.as_str().to_string(),
        unsettled: matches!(
            record.state,
            ControlState::StopRequested | ControlState::Draining | ControlState::Unresolved
        ) || !record.unresolved_categories.is_empty(),
    }
}

/// Build and persist `usage_reconciled` for a finished run. Idempotent.
pub(super) fn reconcile_run(
    manifest: &OvernightManifest,
    end_limits: &[UsageProviderSnapshot],
) -> Result<UsageReconciled> {
    if let Some(existing) = read_usage_reconciled(manifest) {
        return Ok(existing);
    }
    let store = session_control::store();
    let mut sessions = Vec::new();
    match store.inspect(&manifest.coordinator_session_id) {
        Ok(Some(record)) => sessions.push(session_state(&record)),
        // Without a readable record the coordinator's state is unknown, so its
        // work may still be consuming: fail closed to a lower bound.
        Ok(None) | Err(_) => sessions.push(RunSessionState {
            session_id: manifest.coordinator_session_id.clone(),
            control_state: "unknown".to_string(),
            unsettled: true,
        }),
    }
    sessions.extend(
        store
            .owned_records(&manifest.coordinator_session_id)
            .iter()
            .map(session_state),
    );
    // A record this build cannot read cannot prove who owns it, so any of them
    // may be a worker of this run whose usage was dropped. Surface each one.
    let unreadable = store.incompatible_records();
    sessions.extend(
        unreadable
            .iter()
            .filter(|(id, _)| *id != manifest.coordinator_session_id)
            .map(|(id, _)| RunSessionState {
                session_id: id.clone(),
                control_state: "unreadable".to_string(),
                unsettled: true,
            }),
    );
    let (observations, skipped_lines) = read_ledger(&manifest.run_dir);
    let preflight = read_preflight(manifest);
    let mut reconciled = reconcile_usage(ReconcileInput {
        run_id: &manifest.run_id,
        expected_provider: &manifest.provider_name,
        preflight: preflight.as_ref().map(|p| &p.usage),
        observations: &observations,
        sessions: &sessions,
        end_limits,
        reconciled_at: Utc::now(),
    });
    if !unreadable.is_empty() {
        reconciled.notes.push(format!(
            "{} session control record(s) were unreadable and may belong to this run; their usage is not attributed, so totals are a lower bound.",
            unreadable.len()
        ));
    }
    if skipped_lines > 0 {
        reconciled.notes.push(format!(
            "{skipped_lines} usage ledger line(s) were unreadable and skipped; usage is a lower bound."
        ));
    }
    storage::write_json(&reconciled_path(manifest), &reconciled)?;
    record_event(
        manifest,
        "usage_reconciled",
        usage_reconciled_summary(&reconciled),
        serde_json::to_value(&reconciled)?,
        true,
    )?;
    Ok(reconciled)
}

/// Run-end hook. Never fails the run: reconciliation is reporting only.
pub(super) async fn finalize(manifest: &OvernightManifest) {
    if read_usage_reconciled(manifest).is_some() {
        return;
    }
    let end_limits = match tokio::time::timeout(
        END_LIMIT_FETCH_TIMEOUT,
        crate::usage::fetch_all_provider_usage(),
    )
    .await
    {
        Ok(reports) => build_usage_projection(&reports, manifest).providers,
        Err(_) => Vec::new(),
    };
    if let Err(error) = reconcile_run(manifest, &end_limits) {
        crate::logging::warn(&format!("Overnight usage reconciliation failed: {error:#}"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jcode_overnight_core::UsageSource;

    struct Env {
        _lock: std::sync::MutexGuard<'static, ()>,
        _runtime: tempfile::TempDir,
        previous: Option<std::ffi::OsString>,
    }

    impl Env {
        fn new() -> Self {
            let lock = storage::lock_test_env();
            let runtime = tempfile::tempdir().unwrap();
            let previous = std::env::var_os("JCODE_RUNTIME_DIR");
            crate::env::set_var("JCODE_RUNTIME_DIR", runtime.path());
            Self {
                _lock: lock,
                _runtime: runtime,
                previous,
            }
        }
    }

    impl Drop for Env {
        fn drop(&mut self) {
            match self.previous.take() {
                Some(value) => crate::env::set_var("JCODE_RUNTIME_DIR", value),
                None => crate::env::remove_var("JCODE_RUNTIME_DIR"),
            }
            lock(&ACTIVE).clear();
            lock(&ACCOUNTS).clear();
        }
    }

    fn manifest(temp: &Path) -> OvernightManifest {
        let mut manifest = crate::overnight::tests::test_manifest(temp, "usage-run");
        manifest.coordinator_session_id = "coord".into();
        manifest.provider_name = "openai".into();
        std::fs::create_dir_all(&manifest.run_dir).unwrap();
        manifest
    }

    #[tokio::test]
    async fn worker_and_resumed_generations_attribute_to_run_and_replays_dedupe() {
        let _env = Env::new();
        let temp = tempfile::tempdir().unwrap();
        let manifest = manifest(temp.path());
        let _guard = register_run(&manifest);
        let store = session_control::store();
        store.register("coord", None).unwrap();
        store.register("worker", Some("coord")).unwrap();
        store.register("stranger", Some("elsewhere")).unwrap();

        let first = store.admit("worker").unwrap();
        note_session_account("worker", Some("acct-a"));
        session_control::scope(Some(first), async {
            observe_stream_usage(
                "e1",
                "worker",
                "openai",
                "m",
                [Some(10), Some(1), None, None],
            );
            // Replayed/mirrored copy of the same response.
            observe_stream_usage(
                "e1",
                "worker",
                "openai",
                "m",
                [Some(10), Some(1), None, None],
            );
        })
        .await;
        let stop = store
            .request_stop("coord", "worker", false, "stop")
            .unwrap();
        store.settle("worker", stop.revision).unwrap();
        store.admit_new_work("coord", "worker", false).unwrap();
        let second = store.admit("worker").unwrap();
        session_control::scope(Some(second), async {
            observe_stream_usage(
                "e2",
                "worker",
                "openai",
                "m",
                [Some(20), Some(2), None, None],
            );
        })
        .await;
        // Not owned by this run, and a nested non-run worker's usage is ignored.
        let other = store.admit("stranger").unwrap();
        session_control::scope(Some(other), async {
            observe_stream_usage(
                "e3",
                "stranger",
                "openai",
                "m",
                [Some(99), Some(9), None, None],
            );
        })
        .await;
        // Unattributable session id (no control record): never invented as run usage.
        observe_stream_usage("e4", "ghost", "openai", "m", [Some(1), Some(1), None, None]);

        let reconciled = reconcile_run(&manifest, &[]).unwrap();
        let worker = reconciled
            .sessions
            .iter()
            .find(|s| s.session_id == "worker")
            .unwrap();
        let gens: Vec<_> = worker
            .generations
            .iter()
            .map(|g| (g.generation, g.totals.requests, g.totals.input_tokens))
            .collect();
        assert_eq!(gens, vec![(Some(1), 1, Some(10)), (Some(2), 1, Some(20))]);
        assert!(
            reconciled
                .sessions
                .iter()
                .all(|s| s.session_id != "stranger")
        );
        assert_eq!(reconciled.providers.len(), 1);
        assert_eq!(reconciled.providers[0].account.as_deref(), Some("acct-a"));
        assert_eq!(
            reconciled.providers[0]
                .totals
                .as_ref()
                .unwrap()
                .input_tokens,
            Some(30)
        );
        let events = read_events(&manifest).unwrap();
        assert!(events.iter().any(|e| e.kind == "usage_reconciled"));
        // Idempotent: a second finalize keeps the first record and emits no second event.
        reconcile_run(&manifest, &[]).unwrap();
        let count = read_events(&manifest)
            .unwrap()
            .iter()
            .filter(|e| e.kind == "usage_reconciled")
            .count();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn cancelled_worker_stays_attributable_and_never_reads_as_zero() {
        let _env = Env::new();
        let temp = tempfile::tempdir().unwrap();
        let manifest = manifest(temp.path());
        let _guard = register_run(&manifest);
        let store = session_control::store();
        store.register("coord", None).unwrap();
        store.register("silent", Some("coord")).unwrap();
        store.register("stopping", Some("coord")).unwrap();
        let token = store.admit("stopping").unwrap();
        session_control::scope(Some(token), async {
            observe_stream_usage("s1", "stopping", "openai", "m", [Some(5), None, None, None]);
        })
        .await;
        store.admit("silent").unwrap();
        store
            .request_stop("coord", "stopping", false, "cancel")
            .unwrap();
        store
            .request_stop("coord", "silent", false, "cancel")
            .unwrap();

        let reconciled = reconcile_run(&manifest, &[]).unwrap();
        let by = |id: &str| {
            reconciled
                .sessions
                .iter()
                .find(|s| s.session_id == id)
                .unwrap()
        };
        assert!(by("stopping").unsettled);
        assert_eq!(by("stopping").source, UsageSource::Reported);
        assert_eq!(by("stopping").generations[0].totals.output_tokens, None);
        assert!(by("silent").unsettled);
        assert_eq!(by("silent").source, UsageSource::Unavailable);
        assert!(by("silent").generations.is_empty());
        assert!(reconciled.notes.iter().any(|n| n.contains("lower bound")));
    }

    #[tokio::test]
    async fn no_usage_is_estimated_not_zero_and_report_renders_it() {
        let _env = Env::new();
        let temp = tempfile::tempdir().unwrap();
        let manifest = manifest(temp.path());
        let reconciled = reconcile_run(&manifest, &[]).unwrap();
        assert_eq!(reconciled.providers[0].source, UsageSource::Unavailable);
        assert!(reconciled.providers[0].totals.is_none());
        render_review_html(&manifest).unwrap();
        let html = std::fs::read_to_string(&manifest.review_path).unwrap();
        assert!(html.contains("Usage: projected versus observed"));
        assert!(
            html.contains("openai: ") && html.contains("<span class=\"badge\">unavailable</span>")
        );
        assert!(html.contains("Not a spend reservation"));
    }

    #[test]
    fn accounting_does_not_change_dispatch_admission() {
        // Structural: reconciliation has no reserve/authorize entry point and
        // its basis says it is post-request. Behavioural: recording usage and
        // reconciling leaves the dispatch decision for a task-admission
        // manifest exactly as it was.
        let _env = Env::new();
        let temp = tempfile::tempdir().unwrap();
        let mut gated = manifest(temp.path());
        gated.task_admission = Some(Default::default());
        let before = ensure_task_dispatch(&gated, &Default::default())
            .unwrap_err()
            .to_string();
        let _guard = register_run(&gated);
        observe_stream_usage("a1", "coord", "openai", "m", [Some(1), Some(1), None, None]);
        let reconciled = reconcile_run(&gated, &[]).unwrap();
        assert_eq!(reconciled.basis, jcode_overnight_core::USAGE_BASIS);
        assert!(!reconciled.providers.is_empty());
        let after = ensure_task_dispatch(&gated, &Default::default())
            .unwrap_err()
            .to_string();
        assert_eq!(before, after);
    }

    #[tokio::test]
    async fn corrupt_control_record_and_garbage_ledger_line_fail_closed() {
        let _env = Env::new();
        let temp = tempfile::tempdir().unwrap();
        let manifest = manifest(temp.path());
        let _guard = register_run(&manifest);
        let store = session_control::store();
        store.register("coord", None).unwrap();
        store.register("broken", Some("coord")).unwrap();
        std::fs::write(
            crate::storage::durable_state_dir().join("session-control/broken.json"),
            b"{not json",
        )
        .unwrap();
        observe_stream_usage("g1", "coord", "openai", "m", [Some(7), Some(1), None, None]);
        let ledger = manifest.run_dir.join(LEDGER_FILE);
        let mut text = std::fs::read_to_string(&ledger).unwrap();
        text.push_str("garbage line\n\n");
        std::fs::write(&ledger, text).unwrap();

        let reconciled = reconcile_run(&manifest, &[]).unwrap();
        let broken = reconciled
            .sessions
            .iter()
            .find(|s| s.session_id == "broken")
            .unwrap();
        assert!(broken.unsettled);
        assert_eq!(broken.source, UsageSource::Unavailable);
        assert!(
            reconciled
                .notes
                .iter()
                .any(|n| n.contains("unreadable and may belong"))
        );
        assert!(
            reconciled
                .notes
                .iter()
                .any(|n| n.contains("1 usage ledger line(s)"))
        );
        assert_eq!(
            reconciled.providers[0]
                .totals
                .as_ref()
                .unwrap()
                .input_tokens,
            Some(7)
        );
    }

    #[tokio::test]
    async fn missing_coordinator_record_is_unsettled_lower_bound() {
        let _env = Env::new();
        let temp = tempfile::tempdir().unwrap();
        let manifest = manifest(temp.path());
        let reconciled = reconcile_run(&manifest, &[]).unwrap();
        let coord = reconciled
            .sessions
            .iter()
            .find(|s| s.session_id == "coord")
            .unwrap();
        assert!(coord.unsettled);
        assert!(reconciled.notes.iter().any(|n| n.contains("lower bound")));
    }

    #[tokio::test]
    async fn reported_provider_renders_tokens_and_reported_badge_end_to_end() {
        let _env = Env::new();
        let temp = tempfile::tempdir().unwrap();
        let manifest = manifest(temp.path());
        let _guard = register_run(&manifest);
        session_control::store().register("coord", None).unwrap();
        observe_stream_usage(
            "r1",
            "coord",
            "openai",
            "m",
            [Some(1234), Some(56), None, None],
        );
        reconcile_run(&manifest, &[]).unwrap();
        render_review_html(&manifest).unwrap();
        let html = std::fs::read_to_string(&manifest.review_path).unwrap();
        assert!(html.contains("1234"), "{html}");
        assert!(html.contains("openai: "), "{html}");
        assert!(
            html.contains("<span class=\"badge\">reported</span>"),
            "{html}"
        );
        assert!(html.contains("stream responses only"), "{html}");
    }
}
