use super::*;
use crate::board::audit::AuditEvent;
use crate::board::provider::{BoardIssue, WorkflowState};
use crate::board::test_scope;
use async_trait::async_trait;
use jcode_config_types::BoardBackend;
use std::collections::VecDeque;
use std::sync::Mutex;

#[derive(Default)]
struct Fake {
    live: Mutex<VecDeque<Result<String, BoardError>>>,
    write: Mutex<VecDeque<Result<(), BoardError>>>,
    reads: Mutex<usize>,
    writes: Mutex<Vec<(String, String)>>,
    other_calls: Mutex<usize>,
}

impl Fake {
    fn with(live: &[&str], write: Vec<Result<(), BoardError>>) -> Self {
        let fake = Self::default();
        fake.live
            .lock()
            .unwrap()
            .extend(live.iter().map(|s| Ok(s.to_string())));
        fake.write.lock().unwrap().extend(write);
        fake
    }
    fn reads(&self) -> usize {
        *self.reads.lock().unwrap()
    }
    fn writes(&self) -> Vec<(String, String)> {
        self.writes.lock().unwrap().clone()
    }
    fn total_calls(&self) -> usize {
        self.reads() + self.writes().len() + *self.other_calls.lock().unwrap()
    }
}

// The trait has exactly these four methods: no method can carry a label,
// assignee or comment, so none is reachable through this API shape.
#[async_trait]
impl BoardProvider for Fake {
    async fn workflow_states(&self, _: &BoardScope) -> Result<Vec<WorkflowState>, BoardError> {
        *self.other_calls.lock().unwrap() += 1;
        Ok(vec![])
    }
    async fn issues(&self, _: &BoardScope) -> Result<Vec<BoardIssue>, BoardError> {
        *self.other_calls.lock().unwrap() += 1;
        Ok(vec![])
    }
    async fn issue_state(&self, _: &BoardScope, _: &str) -> Result<String, BoardError> {
        *self.reads.lock().unwrap() += 1;
        self.live
            .lock()
            .unwrap()
            .pop_front()
            .expect("unscripted read")
    }
    async fn set_issue_state(
        &self,
        _: &BoardScope,
        issue: &str,
        state: &str,
    ) -> Result<(), BoardError> {
        self.writes
            .lock()
            .unwrap()
            .push((issue.into(), state.into()));
        self.write
            .lock()
            .unwrap()
            .pop_front()
            .expect("unscripted write")
    }
}

fn request() -> StatusChangeRequest {
    StatusChangeRequest {
        scope: test_scope(BoardBackend::Linear),
        issue_id: "i-1".into(),
        expected_prior_state: "s-todo".into(),
        requested_state: "s-done".into(),
    }
}

fn same_scope() -> Result<BoardScope, BoardUnavailable> {
    Ok(test_scope(BoardBackend::Linear))
}

fn lines(audit: &AuditLog) -> Vec<AuditEvent> {
    std::fs::read_to_string(audit.path())
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

fn audit() -> (tempfile::TempDir, AuditLog) {
    let dir = tempfile::tempdir().unwrap();
    let log = AuditLog::open(&dir.path().join("a.jsonl")).unwrap();
    (dir, log)
}

#[tokio::test]
async fn confirmed_change_audits_intent_then_writes_then_audits_outcome() {
    let (_d, log) = audit();
    let fake = Fake::with(&["s-todo"], vec![Ok(())]);
    let req = request();
    let c = Confirmation::confirm(&req).unwrap();
    let op = c.op_id().to_string();
    let out = execute(&req, c, &fake, &log, &same_scope).await;
    assert_eq!(
        out,
        ChangeOutcome::Applied {
            op_id: op.clone(),
            outcome_unrecorded: None
        }
    );
    assert_eq!(
        fake.writes(),
        vec![("i-1".to_string(), "s-done".to_string())]
    );
    let events = lines(&log);
    assert_eq!(events.len(), 2);
    match (&events[0], &events[1]) {
        (AuditEvent::Intent(i), AuditEvent::Outcome(o)) => {
            assert_eq!(
                (i.op_id.as_str(), o.op_id.as_str()),
                (op.as_str(), op.as_str())
            );
            assert_eq!(i.world, "w");
            assert_eq!(i.backend, "linear");
            assert_eq!(i.team, "team-1");
            assert_eq!(i.expected_prior_state, "s-todo");
            assert_eq!(i.requested_state, "s-done");
            assert_eq!(o.result, AuditResult::Applied);
        }
        other => panic!("{other:?}"),
    }
}

#[tokio::test]
async fn declined_or_dropped_confirmation_does_nothing() {
    let (_d, log) = audit();
    let fake = Fake::default();
    // The user declines: `confirm` is never called and `execute` never runs.
    let req = request();
    drop(req);
    assert_eq!(fake.total_calls(), 0);
    assert!(lines(&log).is_empty());
    // A confirmation dropped unused performs no write and no audit either.
    let req = request();
    let c = Confirmation::confirm(&req).unwrap();
    drop(c);
    assert_eq!(fake.total_calls(), 0);
    assert!(lines(&log).is_empty());
}

#[tokio::test]
async fn ado_is_refused_before_confirmation_with_no_audit_or_provider_call() {
    let (_d, log) = audit();
    let fake = Fake::default();
    let mut req = request();
    req.scope = test_scope(BoardBackend::Ado);
    assert_eq!(Confirmation::confirm(&req), Err(Refusal::ReadOnly));
    // Even a confirmation minted for a Linear request cannot be replayed
    // against an ADO request: ADO is refused first.
    let linear = request();
    let c = Confirmation::confirm(&linear).unwrap();
    let out = execute(&req, c, &fake, &log, &|| Ok(test_scope(BoardBackend::Ado))).await;
    assert_eq!(out, ChangeOutcome::Refused(Refusal::ReadOnly));
    assert_eq!(fake.total_calls(), 0);
    assert!(lines(&log).is_empty());
}

#[tokio::test]
async fn confirmation_must_match_every_bound_field() {
    let base = request();
    type Mutation = Box<dyn Fn(&mut StatusChangeRequest)>;
    let mutate: Vec<(&str, Mutation)> = vec![
        ("issue", Box::new(|r| r.issue_id = "i-2".into())),
        (
            "prior",
            Box::new(|r| r.expected_prior_state = "s-other".into()),
        ),
        ("requested", Box::new(|r| r.requested_state = "s-x".into())),
        ("world", Box::new(|r| r.scope.world = "other".into())),
        ("team", Box::new(|r| r.scope.team = "team-2".into())),
        ("project", Box::new(|r| r.scope.project = Some("p".into()))),
        (
            "root",
            Box::new(|r| r.scope.canonical_root = Some("/elsewhere".into())),
        ),
    ];
    for (name, change) in mutate {
        let (_d, log) = audit();
        let fake = Fake::default();
        let c = Confirmation::confirm(&base).unwrap();
        let mut altered = base.clone();
        change(&mut altered);
        let scope = altered.scope.clone();
        let out = execute(&altered, c, &fake, &log, &move || Ok(scope.clone())).await;
        assert_eq!(
            out,
            ChangeOutcome::Refused(Refusal::ConfirmationMismatch),
            "{name}"
        );
        assert_eq!(fake.total_calls(), 0, "{name}");
        assert!(lines(&log).is_empty(), "{name}");
    }
}

#[tokio::test]
async fn no_op_change_is_refused() {
    let mut req = request();
    req.requested_state = req.expected_prior_state.clone();
    assert_eq!(Confirmation::confirm(&req), Err(Refusal::NoChange));
}

#[tokio::test]
async fn world_changed_refuses_without_contacting_provider() {
    for current in [
        Err(BoardUnavailable::UnknownWorld),
        Ok(BoardScope {
            world: "other".into(),
            ..test_scope(BoardBackend::Linear)
        }),
        Ok(BoardScope {
            canonical_root: Some("/moved".into()),
            ..test_scope(BoardBackend::Linear)
        }),
        Ok(BoardScope {
            team: "t2".into(),
            ..test_scope(BoardBackend::Linear)
        }),
    ] {
        let (_d, log) = audit();
        let fake = Fake::default();
        let req = request();
        let c = Confirmation::confirm(&req).unwrap();
        let out = execute(&req, c, &fake, &log, &|| current.clone()).await;
        assert!(
            matches!(out, ChangeOutcome::Refused(Refusal::WorldChanged(_))),
            "{out:?}"
        );
        assert_eq!(fake.total_calls(), 0);
        assert!(lines(&log).is_empty());
    }
}

#[tokio::test]
async fn conflict_writes_nothing_and_needs_no_audit() {
    let (_d, log) = audit();
    let fake = Fake::with(&["s-moved"], vec![]);
    let req = request();
    let c = Confirmation::confirm(&req).unwrap();
    let out = execute(&req, c, &fake, &log, &same_scope).await;
    assert_eq!(
        out,
        ChangeOutcome::Conflict {
            live_state: "s-moved".into()
        }
    );
    assert!(fake.writes().is_empty());
    assert!(lines(&log).is_empty());
}

#[tokio::test]
async fn live_read_failure_blocks_write_and_audit() {
    let (_d, log) = audit();
    let fake = Fake::default();
    fake.live
        .lock()
        .unwrap()
        .push_back(Err(BoardError::Unauthorized("401".into())));
    let req = request();
    let c = Confirmation::confirm(&req).unwrap();
    let out = execute(&req, c, &fake, &log, &same_scope).await;
    assert!(matches!(
        out,
        ChangeOutcome::ReadFailed(BoardError::Unauthorized(_))
    ));
    assert!(fake.writes().is_empty());
    assert!(lines(&log).is_empty());
}

#[tokio::test]
async fn uncertain_result_stops_without_retry_then_reconciles() {
    let (_d, log) = audit();
    let fake = Fake::with(
        &["s-todo", "s-done"],
        vec![Err(BoardError::Uncertain("timeout".into()))],
    );
    let req = request();
    let c = Confirmation::confirm(&req).unwrap();
    let op = c.op_id().to_string();
    let out = execute(&req, c, &fake, &log, &same_scope).await;
    let ChangeOutcome::Uncertain { pending, .. } = out else {
        panic!("{out:?}")
    };
    // Exactly one write attempt: no automatic retry.
    assert_eq!(fake.writes().len(), 1);
    assert_eq!(pending.op_id, op);
    assert_eq!(pending.scope, req.scope);

    let (outcome, unrecorded) = reconcile(&pending, &fake, &log).await;
    assert_eq!((outcome, unrecorded), (ReconcileOutcome::Applied, None));
    assert_eq!(fake.writes().len(), 1, "reconcile never writes");
    let events = lines(&log);
    assert_eq!(events.len(), 3);
    match &events[1] {
        AuditEvent::Outcome(o) => assert_eq!(o.result, AuditResult::Uncertain),
        other => panic!("{other:?}"),
    }
    match &events[2] {
        AuditEvent::Outcome(o) => {
            assert_eq!(o.op_id, op);
            assert_eq!(o.result, AuditResult::Reconciled { applied: true });
        }
        other => panic!("{other:?}"),
    }
}

#[tokio::test]
async fn reconcile_not_applied_diverged_and_unreadable() {
    let (_d, log) = audit();
    let pending = PendingOp {
        op_id: "op".into(),
        scope: test_scope(BoardBackend::Linear),
        issue_id: "i-1".into(),
        expected_prior_state: "s-todo".into(),
        requested_state: "s-done".into(),
    };
    let fake = Fake::with(&["s-todo", "s-third"], vec![]);
    fake.live
        .lock()
        .unwrap()
        .push_back(Err(BoardError::Unavailable("down".into())));
    assert_eq!(
        reconcile(&pending, &fake, &log).await.0,
        ReconcileOutcome::NotApplied
    );
    assert_eq!(
        reconcile(&pending, &fake, &log).await.0,
        ReconcileOutcome::Diverged {
            live_state: "s-third".into()
        }
    );
    // Unreadable: no record, still uncertain.
    let (outcome, _) = reconcile(&pending, &fake, &log).await;
    assert!(matches!(outcome, ReconcileOutcome::Unknown(_)));
    let events = lines(&log);
    assert_eq!(events.len(), 2);
    for e in events {
        match e {
            AuditEvent::Outcome(o) => {
                assert_eq!(o.result, AuditResult::Reconciled { applied: false })
            }
            other => panic!("{other:?}"),
        }
    }
    assert!(fake.writes().is_empty());
}

#[tokio::test]
async fn provider_error_is_recorded_failed() {
    let (_d, log) = audit();
    let fake = Fake::with(
        &["s-todo"],
        vec![Err(BoardError::Provider("bad state".into()))],
    );
    let req = request();
    let c = Confirmation::confirm(&req).unwrap();
    let out = execute(&req, c, &fake, &log, &same_scope).await;
    assert!(matches!(
        out,
        ChangeOutcome::Failed {
            error: BoardError::Provider(_),
            ..
        }
    ));
    assert_eq!(fake.writes().len(), 1);
    match &lines(&log)[1] {
        AuditEvent::Outcome(o) => assert_eq!(o.result, AuditResult::Failed),
        other => panic!("{other:?}"),
    }
}

#[tokio::test]
async fn duplicate_confirmation_is_a_new_operation() {
    let (_d, log) = audit();
    let fake = Fake::with(&["s-todo", "s-todo"], vec![Ok(()), Ok(())]);
    let req = request();
    let first = Confirmation::confirm(&req).unwrap();
    let second = Confirmation::confirm(&req).unwrap();
    assert_ne!(first.op_id(), second.op_id());
    let (a, b) = (first.op_id().to_string(), second.op_id().to_string());
    execute(&req, first, &fake, &log, &same_scope).await;
    execute(&req, second, &fake, &log, &same_scope).await;
    let ops: Vec<String> = lines(&log)
        .into_iter()
        .filter_map(|e| match e {
            AuditEvent::Intent(i) => Some(i.op_id),
            _ => None,
        })
        .collect();
    assert_eq!(ops, [a, b]);
    // Each did its own expected-prior-state read.
    assert_eq!(fake.reads(), 2);
}

#[cfg(target_os = "linux")]
#[tokio::test]
async fn audit_io_failure_blocks_dispatch() {
    // /dev/full opens fine and fails every write.
    let log = AuditLog::open(std::path::Path::new("/dev/full")).expect("/dev/full opens");
    let fake = Fake::with(&["s-todo"], vec![]);
    let req = request();
    let c = Confirmation::confirm(&req).unwrap();
    let out = execute(&req, c, &fake, &log, &same_scope).await;
    assert!(matches!(out, ChangeOutcome::AuditFailed(_)), "{out:?}");
    assert!(
        fake.writes().is_empty(),
        "no dispatch without an audit intent"
    );
}

#[cfg(target_os = "linux")]
#[tokio::test]
async fn outcome_audit_failure_is_reported_not_hidden() {
    // /dev/full fails every write, so the helper must report the failure.
    let log = AuditLog::open(std::path::Path::new("/dev/full")).expect("/dev/full opens");
    assert!(record_outcome(&log, "op", AuditResult::Applied).is_some());
}

#[tokio::test]
async fn world_change_between_audit_and_dispatch_refuses_and_records_the_outcome() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    let (_d, log) = audit();
    let fake = Fake::with(&["s-todo"], vec![Ok(())]);
    let req = request();
    let c = Confirmation::confirm(&req).unwrap();
    // The first re-resolution (before the live read) still matches; the second
    // (right before dispatch) reports a different world.
    let calls = AtomicUsize::new(0);
    let current = || {
        if calls.fetch_add(1, Ordering::SeqCst) == 0 {
            same_scope()
        } else {
            Ok(BoardScope {
                team: "another-team".into(),
                ..test_scope(BoardBackend::Linear)
            })
        }
    };
    let out = execute(&req, c, &fake, &log, &current).await;
    assert!(
        matches!(out, ChangeOutcome::Refused(Refusal::WorldChanged(None))),
        "{out:?}"
    );
    assert!(fake.writes().is_empty(), "nothing was dispatched");
    assert_eq!(calls.load(Ordering::SeqCst), 2, "rechecked twice");
    assert_eq!(lines(&log).len(), 2, "intent and a failed outcome");
}

#[tokio::test]
async fn discovered_scope_is_refused_before_confirmation_with_no_audit_or_provider_call() {
    use super::super::binding::ScopeOrigin;
    let (_d, log) = audit();
    let fake = Fake::default();
    let mut req = request();
    req.scope.origin = ScopeOrigin::Discovered;
    req.scope.world = String::new();
    assert_eq!(Confirmation::confirm(&req), Err(Refusal::NotConfigured));
    let c = Confirmation::confirm(&request()).unwrap();
    let scope = req.scope.clone();
    let out = execute(&req, c, &fake, &log, &|| Ok(scope.clone())).await;
    assert_eq!(out, ChangeOutcome::Refused(Refusal::NotConfigured));
    assert!(Refusal::NotConfigured.to_string().contains("read-only"));
    assert_eq!(fake.total_calls(), 0);
    assert!(lines(&log).is_empty());
}
