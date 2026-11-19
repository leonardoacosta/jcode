use super::*;
use crate::message::{Message, StreamEvent, ToolDefinition};
use async_trait::async_trait;
use jcode_overnight_core::UsageSource;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Provider whose response never completes, like a stuck network stream.
#[derive(Clone)]
struct HangingProvider {
    calls: Arc<AtomicUsize>,
}

#[async_trait]
impl Provider for HangingProvider {
    async fn complete(
        &self,
        _: &[Message],
        _: &[ToolDefinition],
        _: &str,
        _: Option<&str>,
    ) -> Result<crate::provider::EventStream> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(Box::pin(futures::stream::pending::<Result<StreamEvent>>()))
    }
    fn name(&self) -> &str {
        "hanging"
    }
    fn model(&self) -> String {
        "hanging-model".into()
    }
    fn fork(&self) -> Arc<dyn Provider> {
        Arc::new(self.clone())
    }
}

struct Fixture {
    _sandbox: crate::auth::test_sandbox::AuthTestSandbox,
}

fn fixture() -> Fixture {
    Fixture {
        _sandbox: crate::auth::test_sandbox::AuthTestSandbox::new().unwrap(),
    }
}

pub(crate) fn run_manifest(
    run_id: &str,
    coordinator: &str,
    limits: OvernightLimits,
) -> OvernightManifest {
    let root = storage::jcode_dir().unwrap();
    let mut manifest = crate::overnight::tests::test_manifest(&root, run_id);
    let dir = run_dir(run_id).unwrap();
    manifest.coordinator_session_id = coordinator.to_string();
    manifest.process_id = std::process::id();
    manifest.limits = limits;
    manifest.events_path = dir.join("events.jsonl");
    manifest.human_log_path = dir.join("run.log");
    manifest.review_path = dir.join("review.html");
    manifest.review_notes_path = dir.join("review-notes.md");
    manifest.preflight_path = dir.join("preflight.json");
    manifest.task_cards_dir = dir.join("task-cards");
    manifest.issue_drafts_dir = dir.join("issue-drafts");
    manifest.validation_dir = dir.join("validation");
    manifest.run_dir = dir;
    std::fs::create_dir_all(&manifest.run_dir).unwrap();
    save_manifest(&manifest).unwrap();
    manifest
}

fn request_cancel(run_id: &str) {
    let mut manifest = load_manifest(run_id).unwrap();
    manifest.status = OvernightRunStatus::CancelRequested;
    manifest.cancel_requested_at = Some(Utc::now());
    save_manifest(&manifest).unwrap();
}

fn request_cancel_outcome(run_id: &str) -> CancelOutcome {
    crate::overnight::request_cancel(load_manifest(run_id).unwrap()).unwrap()
}

fn event_kinds(manifest: &OvernightManifest) -> Vec<String> {
    read_events(manifest)
        .unwrap()
        .into_iter()
        .map(|event| event.kind)
        .collect()
}

async fn hanging_agent() -> (Agent, Arc<AtomicUsize>) {
    let calls = Arc::new(AtomicUsize::new(0));
    let provider: Arc<dyn Provider> = Arc::new(HangingProvider {
        calls: calls.clone(),
    });
    let registry = Registry::new(provider.clone()).await;
    let agent = Agent::new(provider, registry);
    (agent, calls)
}

#[tokio::test]
async fn cancel_interrupts_a_running_turn_then_settles_and_blocks_later_admission() {
    let _env = fixture();
    let (mut agent, calls) = hanging_agent().await;
    let coordinator = agent.session_id().to_string();
    let manifest = run_manifest("cancel-mid-turn", &coordinator, OvernightLimits::default());

    let run_id = manifest.run_id.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(150)).await;
        request_cancel(&run_id);
    });
    let began = std::time::Instant::now();
    let outcome = run_turn_monitored(&mut agent, &manifest, "work forever")
        .await
        .unwrap();
    assert!(
        matches!(outcome, TurnOutcome::Stopped(StopCause::Cancel)),
        "a hung provider stream must be interrupted by cancellation"
    );
    assert!(
        began.elapsed() < Duration::from_secs(5),
        "cancel was prompt"
    );
    assert_eq!(calls.load(Ordering::SeqCst), 1, "the turn really started");

    // Admission is revoked: no later turn, tool or worker may start.
    let store = session_control::store();
    assert!(store.admit(&coordinator).is_err());
    let current = load_manifest(&manifest.run_id).unwrap();
    stop_run(&current, StopCause::Cancel).await.unwrap();

    let done = load_manifest(&manifest.run_id).unwrap();
    assert_eq!(done.status, OvernightRunStatus::Cancelled);
    assert!(done.stop_settlement.as_ref().unwrap().is_settled());
    assert_eq!(
        store.inspect(&coordinator).unwrap().unwrap().state,
        ControlState::Stopped
    );
    let kinds = event_kinds(&done);
    assert!(kinds.contains(&"run_cancel_acknowledged".to_string()));
    assert!(kinds.contains(&"run_cancelled".to_string()));

    // A second turn on a cancelled run never reaches the provider.
    let (mut other, other_calls) = hanging_agent().await;
    let again = run_turn_monitored(&mut other, &current, "again")
        .await
        .unwrap();
    assert!(matches!(again, TurnOutcome::Stopped(StopCause::Cancel)));
    assert_eq!(other_calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn unsettled_owned_work_keeps_cancel_unresolved_not_cancelled() {
    let _env = fixture();
    let manifest = run_manifest("cancel-unresolved", "coord-u", OvernightLimits::default());
    let store = session_control::store();
    store.register("coord-u", None).unwrap();
    store.register("worker-held", Some("coord-u")).unwrap();
    store.register("worker-effect", Some("coord-u")).unwrap();
    // One worker still has work in flight past the drain deadline; another has
    // an unknown external effect that cannot be proven finished.
    let held = store.admit("worker-held").unwrap();
    let _work = held.reserve().unwrap();
    let effect = store.admit("worker-effect").unwrap();
    effect.mark_external_effect("remote_push").unwrap();
    request_cancel(&manifest.run_id);

    let current = load_manifest(&manifest.run_id).unwrap();
    stop_run(&current, StopCause::Cancel).await.unwrap();

    let after = load_manifest(&manifest.run_id).unwrap();
    assert_eq!(
        after.status,
        OvernightRunStatus::CancelRequested,
        "an elapsed drain deadline must not report cancelled"
    );
    assert!(after.completed_at.is_none());
    let StopSettlement::Unresolved { sessions, .. } = after.stop_settlement.clone().unwrap() else {
        panic!("settlement must be unresolved");
    };
    assert!(
        sessions.iter().any(|s| s.starts_with("worker-held")),
        "{sessions:?}"
    );
    assert!(
        sessions.iter().any(|s| s.starts_with("worker-effect")),
        "{sessions:?}"
    );
    let kinds = event_kinds(&after);
    assert!(kinds.contains(&"run_cancel_unresolved".to_string()));
    assert!(!kinds.contains(&"run_cancelled".to_string()));
    // Admission stays revoked while unresolved.
    assert!(store.admit("worker-held").is_err());
    assert!(!after.status.is_terminal());
}

#[tokio::test]
async fn worker_spawn_beyond_cap_is_refused_with_a_visible_reason() {
    let _env = fixture();
    let capped = OvernightLimits {
        max_agents: Some(1),
        ..Default::default()
    };
    let manifest = run_manifest("spawn-cap", "coord-spawn", capped);
    ensure_worker_spawn_allowed("coord-spawn", 0).unwrap();
    let error = ensure_worker_spawn_allowed("coord-spawn", 1).unwrap_err();
    assert!(error.to_string().contains("max 1"), "{error}");
    assert!(event_kinds(&manifest).contains(&"worker_spawn_refused".to_string()));
    assert!(worker_spawn_gate("coord-spawn").await.is_some());

    // Unrelated sessions, and runs without a cap, are never restricted.
    ensure_worker_spawn_allowed("someone-else", 50).unwrap();
    assert!(worker_spawn_gate("someone-else").await.is_none());
    run_manifest("spawn-uncapped", "coord-free", OvernightLimits::default());
    ensure_worker_spawn_allowed("coord-free", 50).unwrap();

    // A cancelling run starts no new workers even under the cap.
    request_cancel("spawn-cap");
    let error = ensure_worker_spawn_allowed("coord-spawn", 0).unwrap_err();
    assert!(error.to_string().contains("being cancelled"), "{error}");
}

#[tokio::test]
async fn token_cap_counts_deduplicated_child_and_resumed_usage_and_labels_estimates() {
    let _env = fixture();
    let limits = OvernightLimits {
        max_run_tokens: Some(100),
        ..Default::default()
    };
    let manifest = run_manifest("token-cap", "coord-t", limits);
    let _active = usage::register_run(&manifest);
    let store = session_control::store();
    store.register("coord-t", None).unwrap();
    store.register("worker-t", Some("coord-t")).unwrap();

    let observe = |event: &str, counters: [Option<u64>; 4]| {
        usage::observe_stream_usage(event, "worker-t", "openai", "m", counters)
    };
    let first = store.admit("worker-t").unwrap();
    session_control::scope(Some(first), async {
        observe("e1", [Some(60), Some(5), None, None]);
        // Replayed/mirrored copy of the same response must not count twice.
        observe("e1", [Some(60), Some(5), None, None]);
    })
    .await;
    assert!(
        pending_stop(&load_manifest("token-cap").unwrap(), None).is_none(),
        "65 tokens is under the 100 cap"
    );

    // Resume the worker as a new generation; its usage still belongs to the run.
    let stop = store
        .request_stop("coord-t", "worker-t", false, "pause")
        .unwrap();
    store.settle("worker-t", stop.revision).unwrap();
    store.admit_new_work("coord-t", "worker-t", false).unwrap();
    let second = store.admit("worker-t").unwrap();
    session_control::scope(Some(second), async {
        observe("e2", [Some(30), Some(10), None, None]);
    })
    .await;

    let usage = run_token_usage(&load_manifest("token-cap").unwrap());
    assert_eq!(usage.tokens, 105, "dedup: 65 + 40, not 170");
    assert_eq!(usage.requests, 2);
    assert_eq!(usage.source, UsageSource::Reported);
    let Some(StopCause::Limit(breach)) = pending_stop(&load_manifest("token-cap").unwrap(), None)
    else {
        panic!("105 tokens must breach a 100 token cap");
    };
    assert_eq!(breach.source, UsageSource::Reported);

    stop_run(
        &load_manifest("token-cap").unwrap(),
        StopCause::Limit(breach),
    )
    .await
    .unwrap();
    let done = load_manifest("token-cap").unwrap();
    assert_eq!(done.status, OvernightRunStatus::Failed);
    assert_eq!(
        done.limit_breach.as_ref().unwrap().kind,
        jcode_overnight_core::LimitKind::RunTokens
    );
    assert!(event_kinds(&done).contains(&"run_limit_stopped".to_string()));
}

#[tokio::test]
async fn estimated_token_usage_is_a_labelled_lower_bound() {
    let _env = fixture();
    let limits = OvernightLimits {
        max_run_tokens: Some(50),
        ..Default::default()
    };
    let manifest = run_manifest("token-estimated", "coord-e", limits);
    let _active = usage::register_run(&manifest);
    session_control::store().register("coord-e", None).unwrap();
    // Input reported, output never reported.
    usage::observe_stream_usage("e1", "coord-e", "p", "m", [Some(80), None, None, None]);
    let Some(StopCause::Limit(breach)) =
        pending_stop(&load_manifest("token-estimated").unwrap(), None)
    else {
        panic!("lower bound already over the cap");
    };
    assert_eq!(breach.source, UsageSource::Estimated);
    assert!(
        breach.message.contains("estimated lower bound"),
        "{}",
        breach.message
    );
}

#[tokio::test]
async fn unavailable_usage_never_claims_an_exact_breach() {
    let _env = fixture();
    let limits = OvernightLimits {
        max_run_tokens: Some(1),
        ..Default::default()
    };
    run_manifest("token-unavailable", "coord-n", limits);
    let manifest = load_manifest("token-unavailable").unwrap();
    assert_eq!(run_token_usage(&manifest).source, UsageSource::Unavailable);
    assert!(pending_stop(&manifest, None).is_none());
}

#[test]
fn turn_time_limit_stops_a_long_turn() {
    let _env = fixture();
    let limits = OvernightLimits {
        max_turn_minutes: Some(1),
        ..Default::default()
    };
    let manifest = run_manifest("turn-time", "coord-time", limits);
    let long_ago = Utc::now() - ChronoDuration::minutes(2);
    assert!(matches!(
        pending_stop(&manifest, Some(long_ago)),
        Some(StopCause::Limit(_))
    ));
    assert!(pending_stop(&manifest, Some(Utc::now())).is_none());
    assert!(
        pending_stop(&manifest, None).is_none(),
        "between turns only tokens apply"
    );
}

#[tokio::test]
async fn within_limits_run_has_no_limit_events_and_limits_never_authorize_dispatch() {
    let _env = fixture();
    let (mut agent, _calls) = hanging_agent().await;
    let coordinator = agent.session_id().to_string();
    let generous = OvernightLimits {
        max_agents: Some(8),
        max_turn_minutes: Some(600),
        max_run_tokens: Some(u64::MAX),
    };
    let mut manifest = run_manifest("within", &coordinator, generous);
    let _active = usage::register_run(&manifest);
    session_control::store()
        .register(&coordinator, None)
        .unwrap();
    usage::observe_stream_usage(
        "e1",
        &coordinator,
        "p",
        "m",
        [Some(1_000), Some(10), None, None],
    );
    assert!(pending_stop(&manifest, Some(Utc::now())).is_none());

    // Let the real monitor loop poll several times over a live turn.
    let monitored = tokio::time::timeout(
        Duration::from_millis(400),
        run_turn_monitored(&mut agent, &manifest, "work"),
    )
    .await;
    assert!(monitored.is_err(), "the turn stays running within limits");
    let current = load_manifest("within").unwrap();
    assert_eq!(current.status, OvernightRunStatus::Running);
    assert!(current.limit_breach.is_none());
    let kinds = event_kinds(&current);
    assert!(
        !kinds
            .iter()
            .any(|kind| kind.starts_with("run_limit") || kind == "limit_reached"),
        "no limit events within limits: {kinds:?}"
    );

    // A token cap with unlimited headroom is not spend authority: the
    // monetary/attempt denial still fails closed.
    manifest.process_id = std::process::id();
    manifest.working_dir = Some(storage::jcode_dir().unwrap().to_string_lossy().into());
    manifest.task_admission = Some(crate::config::TaskAdmissionSnapshot {
        scope: "bounded task".into(),
        authorization: Some(crate::config::TaskAuthorization {
            scope: "bounded task".into(),
            max_attempts: 2,
            max_runtime_seconds: 100,
            max_spend_microusd: 100,
        }),
        ownership_verified: true,
        dependencies_ready: true,
        workspace_ready: true,
        ..Default::default()
    });
    let policy = crate::config::TasksConfig {
        unattended: true,
        ..Default::default()
    };
    assert!(pending_stop(&manifest, None).is_none());
    let denied = ensure_task_dispatch(&manifest, &policy).unwrap_err();
    assert!(denied.to_string().contains("spend enforcer"), "{denied}");
}

#[tokio::test]
async fn run_turn_monitored_interrupts_a_hung_turn_on_the_time_limit() {
    let _env = fixture();
    let (mut agent, calls) = hanging_agent().await;
    let coordinator = agent.session_id().to_string();
    // Under cfg(test) one limit "minute" is one real second.
    let limits = OvernightLimits {
        max_turn_minutes: Some(1),
        ..Default::default()
    };
    let manifest = run_manifest("turn-limit", &coordinator, limits);
    let outcome = tokio::time::timeout(
        Duration::from_secs(10),
        run_turn_monitored(&mut agent, &manifest, "hang"),
    )
    .await
    .expect("the time limit must interrupt a hung turn")
    .unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 1, "the turn really started");
    let TurnOutcome::Stopped(StopCause::Limit(breach)) = outcome else {
        panic!("expected a Limit stop");
    };
    assert_eq!(breach.kind, jcode_overnight_core::LimitKind::TurnTime);
    // The breach is persisted before the run is torn down.
    let persisted = load_manifest("turn-limit").unwrap();
    assert_eq!(persisted.limit_breach.as_ref().unwrap().kind, breach.kind);
    assert!(event_kinds(&persisted).contains(&"limit_reached".to_string()));

    stop_run(&persisted, StopCause::Limit(breach))
        .await
        .unwrap();
    let done = load_manifest("turn-limit").unwrap();
    assert_eq!(done.status, OvernightRunStatus::Failed);
    assert!(done.stop_settlement.as_ref().unwrap().is_settled());
    assert!(done.limit_breach.is_some());
    assert!(
        event_kinds(&done)
            .iter()
            .filter(|k| *k == "limit_reached")
            .count()
            == 1,
        "the breach event is recorded once"
    );
}

#[tokio::test]
async fn unreadable_manifest_is_recorded_after_repeated_poll_failures() {
    let _env = fixture();
    let (mut agent, _calls) = hanging_agent().await;
    let coordinator = agent.session_id().to_string();
    let manifest = run_manifest("unreadable", &coordinator, OvernightLimits::default());
    let path = manifest_path("unreadable").unwrap();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(60)).await;
        std::fs::write(&path, b"{not json").unwrap();
    });
    let monitored = tokio::time::timeout(
        Duration::from_millis(600),
        run_turn_monitored(&mut agent, &manifest, "work"),
    )
    .await;
    assert!(
        monitored.is_err(),
        "an unreadable manifest must not stop the turn"
    );
    assert!(event_kinds(&manifest).contains(&"manifest_unreadable".to_string()));
}

#[tokio::test]
async fn settle_re_enumerates_owned_work_registered_during_the_drain() {
    let _env = fixture();
    let manifest = run_manifest("settle-fixpoint", "coord-f", OvernightLimits::default());
    let store = session_control::store();
    store.register("coord-f", None).unwrap();
    store.register("worker-a", Some("coord-f")).unwrap();
    let held = store.admit("worker-a").unwrap();
    let work = held.reserve().unwrap();
    // A spawn already past the gate registers a worker while worker-a drains.
    let late = tokio::spawn(async {
        tokio::time::sleep(Duration::from_millis(40)).await;
        session_control::store()
            .register("worker-late", Some("coord-f"))
            .unwrap();
        tokio::time::sleep(Duration::from_millis(60)).await;
        drop(work);
    });
    let settlement = settle_owned_work(&manifest).await;
    late.await.unwrap();
    assert!(settlement.is_settled(), "{settlement:?}");
    for id in ["coord-f", "worker-a", "worker-late"] {
        assert_eq!(
            store.inspect(id).unwrap().unwrap().state,
            ControlState::Stopped,
            "{id}"
        );
    }
    assert!(
        store.admit("worker-late").is_err(),
        "late worker is revoked"
    );
}

#[tokio::test]
async fn limit_stop_refuses_new_spawns_and_unresolved_work_stays_non_terminal() {
    let _env = fixture();
    let capped = OvernightLimits {
        max_agents: Some(5),
        max_run_tokens: Some(1),
        ..Default::default()
    };
    let manifest = run_manifest("limit-unresolved", "coord-l", capped);
    let store = session_control::store();
    store.register("coord-l", None).unwrap();
    store.register("worker-l", Some("coord-l")).unwrap();
    let held = store.admit("worker-l").unwrap();
    let _work = held.reserve().unwrap();
    let breach = jcode_overnight_core::check_turn_time(
        &OvernightLimits {
            max_turn_minutes: Some(1),
            ..Default::default()
        },
        600,
    )
    .unwrap();

    let settled = stop_run(&manifest, StopCause::Limit(breach)).await.unwrap();
    assert!(!settled);
    let after = load_manifest("limit-unresolved").unwrap();
    assert_eq!(
        after.status,
        OvernightRunStatus::Running,
        "no terminal claim"
    );
    assert!(after.completed_at.is_none());
    assert!(after.limit_breach.is_some());
    assert!(!after.stop_settlement.as_ref().unwrap().is_settled());
    assert!(event_kinds(&after).contains(&"run_limit_unresolved".to_string()));
    assert!(
        read_usage_reconciled(&after).is_none(),
        "no usage finalized yet"
    );
    // The breach is sticky and new workers are refused even below the cap.
    assert!(matches!(
        pending_stop(&after, None),
        Some(StopCause::Limit(_))
    ));
    let error = ensure_worker_spawn_allowed("coord-l", 0).unwrap_err();
    assert!(error.to_string().contains("stopped by a limit"), "{error}");
}

#[tokio::test]
async fn unresolved_cancel_is_re_settled_by_the_supervisor_then_finalizes_usage() {
    let _env = fixture();
    let manifest = run_manifest("resettle", "coord-r", OvernightLimits::default());
    let store = session_control::store();
    store.register("coord-r", None).unwrap();
    store.register("worker-r", Some("coord-r")).unwrap();
    let held = store.admit("worker-r").unwrap();
    let work = held.reserve().unwrap();
    request_cancel("resettle");
    // The worker finishes after the first drain deadline has elapsed.
    let release = tokio::spawn(async move {
        tokio::time::sleep(SETTLE_DEADLINE + Duration::from_millis(100)).await;
        drop(work);
    });
    let current = load_manifest("resettle").unwrap();
    stop_until_settled(&current, StopCause::Cancel)
        .await
        .unwrap();
    release.await.unwrap();
    let done = load_manifest("resettle").unwrap();
    assert_eq!(done.status, OvernightRunStatus::Cancelled);
    assert!(done.stop_settlement.as_ref().unwrap().is_settled());
    let kinds = event_kinds(&done);
    assert!(
        kinds.contains(&"run_cancel_unresolved".to_string()),
        "{kinds:?}"
    );
    assert!(kinds.contains(&"run_cancelled".to_string()));
    assert!(
        read_usage_reconciled(&done).is_some(),
        "usage finalized once settled"
    );
    drop(manifest);
}

#[tokio::test]
async fn repeated_cancel_of_an_unresolved_run_re_runs_settlement() {
    let _env = fixture();
    run_manifest("recancel", "coord-c", OvernightLimits::default());
    let store = session_control::store();
    store.register("coord-c", None).unwrap();
    store.register("worker-c", Some("coord-c")).unwrap();
    let held = store.admit("worker-c").unwrap();
    let work = held.reserve().unwrap();
    request_cancel("recancel");
    stop_run(&load_manifest("recancel").unwrap(), StopCause::Cancel)
        .await
        .unwrap();
    assert_eq!(
        load_manifest("recancel").unwrap().status,
        OvernightRunStatus::CancelRequested
    );
    // Work finishes, the user cancels again: settlement runs and completes.
    drop(work);
    let outcome = request_cancel_outcome("recancel");
    assert!(matches!(outcome, CancelOutcome::Requested(_)));
    let mut status = OvernightRunStatus::CancelRequested;
    for _ in 0..100 {
        status = load_manifest("recancel").unwrap().status;
        if status == OvernightRunStatus::Cancelled {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert_eq!(status, OvernightRunStatus::Cancelled);
}

#[test]
fn visible_mode_rejects_unenforceable_limits_but_allows_max_agents() {
    let none = OvernightLimits::default();
    assert!(visible_mode_limit_rejection(&none).is_none());
    let agents = OvernightLimits {
        max_agents: Some(2),
        ..Default::default()
    };
    assert!(visible_mode_limit_rejection(&agents).is_none());
    for limits in [
        OvernightLimits {
            max_turn_minutes: Some(5),
            ..Default::default()
        },
        OvernightLimits {
            max_run_tokens: Some(5),
            ..Default::default()
        },
    ] {
        let reason = visible_mode_limit_rejection(&limits).unwrap();
        assert!(reason.contains("supervised"), "{reason}");
    }
}

#[tokio::test]
async fn starting_a_visible_run_with_unenforceable_limits_fails_before_creating_it() {
    let _env = fixture();
    let (agent, _calls) = hanging_agent().await;
    let provider: Arc<dyn Provider> = Arc::new(HangingProvider {
        calls: Arc::new(AtomicUsize::new(0)),
    });
    let registry = Registry::new(provider.clone()).await;
    let error = match start_overnight_run(OvernightStartOptions {
        duration: OvernightDuration { minutes: 60 },
        mission: None,
        parent_session: Session::create(None, None),
        provider,
        registry,
        working_dir: None,
        use_current_session: true,
        limits: OvernightLimits {
            max_run_tokens: Some(10),
            ..Default::default()
        },
    }) {
        Ok(_) => panic!("visible run must reject a token cap"),
        Err(error) => error,
    };
    assert!(
        error.to_string().contains("only enforced for supervised"),
        "{error}"
    );
    assert!(latest_manifest().unwrap().is_none(), "no run was created");
    drop(agent);
}

#[tokio::test]
async fn cancelled_visible_run_no_longer_governs_or_blocks_worker_spawns() {
    let _env = fixture();
    let capped = OvernightLimits {
        max_agents: Some(1),
        ..Default::default()
    };
    let mut manifest = run_manifest("visible-cancel", "visible-session", capped);
    // A visible run uses one session as both parent and coordinator.
    manifest.parent_session_id = "visible-session".to_string();
    save_manifest(&manifest).unwrap();
    assert!(!is_supervised(&manifest));
    // Before cancel the cap still governs the visible run.
    assert!(ensure_worker_spawn_allowed("visible-session", 1).is_err());

    let outcome = request_cancel_outcome("visible-cancel");
    assert!(matches!(outcome, CancelOutcome::Requested(_)));
    // Nothing supervises it, so it stays cancel-requested, drains nothing...
    let after = load_manifest("visible-cancel").unwrap();
    assert_eq!(after.status, OvernightRunStatus::CancelRequested);
    assert!(after.stop_settlement.is_none());
    // ...and no longer refuses spawns for the session.
    ensure_worker_spawn_allowed("visible-session", 0).unwrap();
    ensure_worker_spawn_allowed("visible-session", 5).unwrap();
    assert!(worker_spawn_gate("visible-session").await.is_none());
}

#[tokio::test]
async fn user_cancel_during_an_unresolved_limit_stop_switches_to_cancel() {
    let _env = fixture();
    let manifest = run_manifest("limit-then-cancel", "coord-lc", OvernightLimits::default());
    let store = session_control::store();
    store.register("coord-lc", None).unwrap();
    store.register("worker-lc", Some("coord-lc")).unwrap();
    let held = store.admit("worker-lc").unwrap();
    let work = held.reserve().unwrap();
    let breach = jcode_overnight_core::check_turn_time(
        &OvernightLimits {
            max_turn_minutes: Some(1),
            ..Default::default()
        },
        600,
    )
    .unwrap();
    // The first drain times out (unresolved); the user cancels during it and the
    // worker finishes only during the first re-settle.
    let canceller = tokio::spawn(async move {
        tokio::time::sleep(SETTLE_DEADLINE / 3).await;
        request_cancel("limit-then-cancel");
        tokio::time::sleep(SETTLE_DEADLINE + Duration::from_millis(100)).await;
        drop(work);
    });
    stop_until_settled(&manifest, StopCause::Limit(breach))
        .await
        .unwrap();
    canceller.await.unwrap();
    let done = load_manifest("limit-then-cancel").unwrap();
    assert_eq!(done.status, OvernightRunStatus::Cancelled);
    assert!(event_kinds(&done).contains(&"run_cancelled".to_string()));
}

#[tokio::test]
async fn supervisor_error_with_unresolved_owned_work_leaves_the_run_non_terminal() {
    let _env = fixture();
    let mut unresolved = run_manifest("fail-unresolved", "coord-fu", OvernightLimits::default());
    unresolved.stop_settlement = Some(StopSettlement::Unresolved {
        at: Utc::now(),
        sessions: vec!["worker: draining".to_string()],
        reason: "owned work was not confirmed stopped".to_string(),
    });
    save_manifest(&unresolved).unwrap();
    let error = anyhow::anyhow!("disk full");
    crate::overnight::record_supervisor_failure(&unresolved, &error).await;
    let after = load_manifest("fail-unresolved").unwrap();
    assert_eq!(after.status, OvernightRunStatus::Running);
    assert!(after.completed_at.is_none());
    assert!(event_kinds(&after).contains(&"run_failed_unresolved".to_string()));

    // With nothing unresolved the error still closes the run as Failed.
    let clean = run_manifest("fail-clean", "coord-fc", OvernightLimits::default());
    crate::overnight::record_supervisor_failure(&clean, &error).await;
    let after = load_manifest("fail-clean").unwrap();
    assert_eq!(after.status, OvernightRunStatus::Failed);
    assert!(after.completed_at.is_some());
}
