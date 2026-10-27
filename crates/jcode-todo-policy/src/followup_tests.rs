use super::*;
use crate::{
    ConfidenceState, DeliveryState, GateObservationKind, IntentUnderstanding, is_auto_poke_message,
};

// Reviewed synthetic baseline for the pure todo-policy engine. Expected outcomes below are
// current `FollowUpPolicy::decide` behavior, not proof of runtime acceptance or a loop fix.
// Expected: identical open work repeats quietly; details-only change emits another poke;
// added work pokes again; speculative completion requests validation, verified completion
// advances to one final response. Gate order is digest (clear observations), ownership,
// validation, final response; MAX_GATE_ATTEMPTS permits five gate turns then notice-only stop.
// Disable stays quiet; new open work re-arms after the breaker. These may be undesirable
// baseline outcomes, not accepted behavior improvements.
// This input has a complete TodoSnapshot/controller state, but no native ToolDone/errors or
// route provenance: those are not inputs to this pure policy. Do not synthesize them from logs.
// Absent gateway traffic, claimed confidence, or successful tool completion alone cannot prove
// that a continuation occurred or that work was accepted; identity gaps remain unattributed.

fn todo(content: &str, status: &str) -> TodoItem {
    TodoItem {
        content: content.into(),
        status: status.into(),
        priority: "high".into(),
        id: content.into(),
        ..Default::default()
    }
}

fn done(content: &str, confidence: ConfidenceState) -> TodoItem {
    TodoItem {
        completion_confidence: Some(confidence),
        ..todo(content, "completed")
    }
}

fn snapshot(todos: Vec<TodoItem>) -> TodoSnapshot {
    TodoSnapshot {
        todos,
        ..Default::default()
    }
}

fn kind(decision: &Decision) -> Option<FollowUpKind> {
    decision.follow_up.as_ref().map(|f| f.kind)
}

#[test]
fn disabled_policy_never_follows_up() {
    let mut policy = FollowUpPolicy::new(false);
    assert_eq!(
        policy.decide(&snapshot(vec![todo("a", "pending")])),
        Decision::default()
    );
    policy.enable();
    assert_eq!(
        kind(&policy.decide(&snapshot(vec![todo("a", "pending")]))),
        Some(FollowUpKind::IncompleteTodos)
    );
}

#[test]
fn pokes_open_work_once_per_distinct_state() {
    let mut policy = FollowUpPolicy::new(true);
    let open = snapshot(vec![todo("a", "pending"), todo("b", "in_progress")]);
    let first = policy.decide(&open);
    let follow_up = first.follow_up.expect("poke");
    assert_eq!(follow_up.kind, FollowUpKind::IncompleteTodos);
    assert!(is_auto_poke_message(&follow_up.message));
    assert!(follow_up.notice.contains("2 incomplete todos"));
    // Unchanged open work means the model is stalled: no loop.
    assert_eq!(policy.decide(&open), Decision::default());
    // Progress re-arms the poke.
    let progressed = snapshot(vec![todo("a", "completed"), todo("b", "in_progress")]);
    assert_eq!(
        kind(&policy.decide(&progressed)),
        Some(FollowUpKind::IncompleteTodos)
    );
}

#[test]
fn continuation_status_alias_does_not_rearm_but_required_work_does() {
    let mut policy = FollowUpPolicy::new(true);
    let mut state = snapshot(vec![todo("compile", "in_progress")]);
    assert_eq!(
        kind(&policy.decide(&state)),
        Some(FollowUpKind::IncompleteTodos)
    );
    assert_eq!(policy.decide(&state), Decision::default());
    state.todos[0].status = " IN-PROGRESS ".into();
    // Equivalent spelling carries no new work, unlike a real status transition.
    assert_eq!(policy.decide(&state), Decision::default());
    for details in [
        "same work, more context",
        "Also test crates/changed.rs before acceptance",
    ] {
        state.todos[0].details = Some(details.into());
        assert_eq!(
            kind(&policy.decide(&state)),
            Some(FollowUpKind::IncompleteTodos)
        );
    }
    state.todos.push(todo("test", "pending"));
    assert_eq!(
        kind(&policy.decide(&state)),
        Some(FollowUpKind::IncompleteTodos)
    );
    state.todos[0].status = "pending".into();
    assert_eq!(
        kind(&policy.decide(&state)),
        Some(FollowUpKind::IncompleteTodos)
    );
    // Claimed verification becomes stale: the existing gate must run again.
    let mut evidence = FollowUpPolicy::new(true);
    let mut checked = snapshot(vec![done("test", ConfidenceState::Verified)]);
    assert_eq!(
        kind(&evidence.decide(&checked)),
        Some(FollowUpKind::FinalResponse)
    );
    checked.todos[0].completion_confidence = Some(ConfidenceState::Speculative);
    assert_eq!(
        kind(&evidence.decide(&checked)),
        Some(FollowUpKind::CompletionValidation)
    );
}

#[test]
fn continuation_fingerprint_preserves_unknown_status_order_and_requirements() {
    let original = vec![todo("compile", "in_progress"), todo("test", "pending")];
    let fingerprint = crate::incomplete_todo_fingerprint(&original).unwrap();
    assert_eq!(fingerprint, serde_json::to_string(&original).unwrap());
    for (field, value) in [
        ("status", serde_json::json!("unknown-state")),
        (
            "details",
            serde_json::json!("Must verify the acceptance path"),
        ),
        ("priority", serde_json::json!("low")),
        ("assigned_to", serde_json::json!("worker-2")),
        ("blocked_by", serde_json::json!(["dependency"])),
        ("stage", serde_json::json!(2)),
        (
            "confidence_history",
            serde_json::json!(["speculative", "verified"]),
        ),
    ] {
        let mut changed = serde_json::to_value(&original).unwrap();
        changed[0][field] = value;
        let changed: Vec<TodoItem> = serde_json::from_value(changed).unwrap();
        assert_ne!(
            fingerprint,
            crate::incomplete_todo_fingerprint(&changed).unwrap(),
            "{field}"
        );
    }
    let mut reversed = original.clone();
    reversed.reverse();
    assert_ne!(
        fingerprint,
        crate::incomplete_todo_fingerprint(&reversed).unwrap()
    );
    let mut unknown = original;
    unknown[0].status = "unknown-state".into();
    let before = crate::incomplete_todo_fingerprint(&unknown).unwrap();
    unknown[0].status = " UNKNOWN-STATE ".into();
    assert_ne!(
        before,
        crate::incomplete_todo_fingerprint(&unknown).unwrap()
    );
}

#[test]
fn no_todos_stays_armed_and_quiet() {
    let mut policy = FollowUpPolicy::new(true);
    assert_eq!(policy.decide(&TodoSnapshot::default()), Decision::default());
    assert!(policy.is_enabled());
    assert_eq!(
        kind(&policy.decide(&snapshot(vec![todo("a", "pending")]))),
        Some(FollowUpKind::IncompleteTodos)
    );
}

#[test]
fn completed_work_runs_digest_then_validation_then_final_response_once() {
    let mut policy = FollowUpPolicy::new(true);
    let mut state = TodoSnapshot {
        todos: vec![done("ship", ConfidenceState::Plausible)],
        plan: TodoPlan {
            understands_user_intent: Some(IntentUnderstanding::Clear),
            ..Default::default()
        },
        gate_observations: vec![GateObservation {
            kind: GateObservationKind::IntentUnderstanding,
            group: None,
            state: Some("partial".into()),
        }],
        ..Default::default()
    };

    let digest = policy.decide(&state);
    assert_eq!(kind(&digest), Some(FollowUpKind::GateDigest));
    assert!(digest.effects.clear_gate_observations);
    assert!(is_auto_poke_message(&digest.follow_up.unwrap().message));
    state.gate_observations.clear();

    let validation = policy.decide(&state);
    assert_eq!(kind(&validation), Some(FollowUpKind::CompletionValidation));
    assert!(validation.follow_up.unwrap().message.contains("\"ship\""));

    state.todos = vec![done("ship", ConfidenceState::Verified)];
    let final_response = policy.decide(&state);
    assert_eq!(kind(&final_response), Some(FollowUpKind::FinalResponse));
    assert!(
        final_response
            .follow_up
            .unwrap()
            .notice
            .contains("verified")
    );
    // The final-response turn itself must not trigger another cycle.
    assert_eq!(policy.decide(&state), Decision::default());
    // New work starts a fresh cycle.
    state.todos.push(todo("follow-up", "pending"));
    assert_eq!(
        kind(&policy.decide(&state)),
        Some(FollowUpKind::IncompleteTodos)
    );
}

#[test]
fn empty_digest_still_consumes_the_log() {
    let mut policy = FollowUpPolicy::new(true);
    let state = TodoSnapshot {
        todos: vec![done("ship", ConfidenceState::Verified)],
        gate_observations: Vec::new(),
        ..Default::default()
    };
    // No observations: nothing to clear.
    assert!(!policy.decide(&state).effects.clear_gate_observations);
}

#[test]
fn ownership_gate_asks_once_per_distinct_gap() {
    let mut policy = FollowUpPolicy::new(true);
    let state = TodoSnapshot {
        todos: vec![done("ship", ConfidenceState::Verified)],
        goals: vec![TodoGoal {
            delivery_state: Some(DeliveryState::ChangeMade),
            ..Default::default()
        }],
        ..Default::default()
    };
    assert_eq!(kind(&policy.decide(&state)), Some(FollowUpKind::Ownership));
    // Same assessment: stop instead of looping or claiming success.
    assert_eq!(policy.decide(&state), Decision::default());
}

#[test]
fn confidence_spike_is_challenged_once() {
    let mut policy = FollowUpPolicy::new(true);
    let mut spiked = done("ship", ConfidenceState::Verified);
    spiked.confidence_history = vec![ConfidenceState::Speculative, ConfidenceState::Verified];
    let state = snapshot(vec![spiked]);
    assert_eq!(
        kind(&policy.decide(&state)),
        Some(FollowUpKind::ConfidenceSpike)
    );
    assert_eq!(
        kind(&policy.decide(&state)),
        Some(FollowUpKind::FinalResponse)
    );
}

#[test]
fn long_session_review_precedes_poke_and_is_marked() {
    let mut policy = FollowUpPolicy::new(true);
    let state = TodoSnapshot {
        todos: vec![todo("a", "in_progress")],
        long_session_review_due: true,
        ..Default::default()
    };
    let decision = policy.decide(&state);
    assert_eq!(kind(&decision), Some(FollowUpKind::LongSessionReview));
    assert!(decision.effects.mark_long_session_review_delivered);
}

#[test]
fn exhausted_gate_budget_disarms_with_notice_and_rearms_for_new_work() {
    let mut policy = FollowUpPolicy::new(true);
    let mut sent = 0;
    let mut stopped = false;
    for round in 0..(MAX_GATE_ATTEMPTS as usize + 3) {
        // Keep changing wording so ownership de-dup does not stop it first,
        // while validation never passes.
        let state = snapshot(vec![done(
            &format!("ship {round}"),
            ConfidenceState::Speculative,
        )]);
        let decision = policy.decide(&state);
        if decision.follow_up.is_some() {
            sent += 1;
        }
        if decision.notice.is_some() {
            stopped = true;
            break;
        }
    }
    assert!(stopped, "breaker must trip");
    assert_eq!(sent, MAX_GATE_ATTEMPTS as usize);
    assert_eq!(
        policy.decide(&snapshot(vec![done(
            "ship x",
            ConfidenceState::Speculative
        )])),
        Decision::default()
    );
    assert_eq!(
        kind(&policy.decide(&snapshot(vec![todo("new", "pending")]))),
        Some(FollowUpKind::IncompleteTodos)
    );
}

#[test]
fn disable_sticks_until_enabled() {
    let mut policy = FollowUpPolicy::new(true);
    policy.disable();
    assert!(!policy.is_enabled());
    assert_eq!(
        policy.decide(&snapshot(vec![todo("a", "pending")])),
        Decision::default()
    );
}

#[test]
fn continuation_baseline_repeated_and_changed_work() {
    let mut policy = FollowUpPolicy::new(true);
    let open = snapshot(vec![todo("compile", "pending")]);

    let first = policy.decide(&open);
    assert_eq!(kind(&first), Some(FollowUpKind::IncompleteTodos));
    assert!(first.effects.is_empty());
    assert!(first.notice.is_none());
    assert!(is_auto_poke_message(&first.follow_up.unwrap().message));
    assert_eq!(policy.decide(&open), Decision::default());

    // Repeated identical metadata stays quiet. A metadata-only change (details) currently
    // changes the full todo fingerprint and emits another poke: this is baseline behavior,
    // not evidence that the extra continuation is useful.
    let mut metadata_changed = todo("compile", "pending");
    metadata_changed.details = Some("same work, more context".into());
    let metadata_only = snapshot(vec![metadata_changed]);
    let metadata_decision = policy.decide(&metadata_only);
    assert_eq!(
        kind(&metadata_decision),
        Some(FollowUpKind::IncompleteTodos)
    );
    assert!(metadata_decision.effects.is_empty());
    assert!(metadata_decision.notice.is_none());
    assert!(
        metadata_decision
            .follow_up
            .as_ref()
            .unwrap()
            .notice
            .contains("1 incomplete todo")
    );
    assert_eq!(policy.decide(&metadata_only), Decision::default());

    // New required work changes the todo set and re-arms a poke.
    let newly_required = snapshot(vec![todo("compile", "pending"), todo("test", "pending")]);
    let changed_work = policy.decide(&newly_required);
    assert_eq!(kind(&changed_work), Some(FollowUpKind::IncompleteTodos));
    assert!(
        changed_work
            .follow_up
            .unwrap()
            .notice
            .contains("2 incomplete todos")
    );

    // Changed confidence claims are consumed by the gate, not proof of accepted completion.
    let mut evidence_policy = FollowUpPolicy::new(true);
    let speculative = snapshot(vec![done("compile", ConfidenceState::Speculative)]);
    let validation = evidence_policy.decide(&speculative);
    assert_eq!(kind(&validation), Some(FollowUpKind::CompletionValidation));
    assert!(validation.effects.is_empty());
    let verified = snapshot(vec![done("compile", ConfidenceState::Verified)]);
    let final_response = evidence_policy.decide(&verified);
    assert_eq!(kind(&final_response), Some(FollowUpKind::FinalResponse));
    assert!(final_response.notice.is_none());
    assert!(
        final_response
            .follow_up
            .as_ref()
            .unwrap()
            .notice
            .contains("verified")
    );
    assert_eq!(evidence_policy.decide(&verified), Decision::default());
}

#[test]
fn continuation_baseline_required_checks_and_limits() {
    let mut policy = FollowUpPolicy::new(true);
    let mut state = TodoSnapshot {
        todos: vec![done("ship", ConfidenceState::Speculative)],
        goals: vec![TodoGoal {
            delivery_state: Some(DeliveryState::ChangeMade),
            ..Default::default()
        }],
        gate_observations: vec![GateObservation {
            kind: GateObservationKind::IntentUnderstanding,
            group: None,
            state: Some("partial".into()),
        }],
        ..Default::default()
    };

    // Deferred evidence is handled before ownership, then ownership before validation.
    let digest = policy.decide(&state);
    assert_eq!(kind(&digest), Some(FollowUpKind::GateDigest));
    assert!(digest.effects.clear_gate_observations);
    assert!(!digest.effects.mark_long_session_review_delivered);
    state.gate_observations.clear();
    let ownership = policy.decide(&state);
    assert_eq!(kind(&ownership), Some(FollowUpKind::Ownership));
    assert!(ownership.effects.is_empty());

    // Once delivery is sufficient, low completion confidence gets validation, then verified
    // confidence claims advance the policy. The same final fingerprint is quiet afterwards.
    state.goals.clear();
    let validation = policy.decide(&state);
    assert_eq!(kind(&validation), Some(FollowUpKind::CompletionValidation));
    state.todos = vec![done("ship", ConfidenceState::Verified)];
    let final_response = policy.decide(&state);
    assert_eq!(kind(&final_response), Some(FollowUpKind::FinalResponse));
    assert_eq!(policy.decide(&state), Decision::default());

    // Gate budget allows exactly MAX_GATE_ATTEMPTS validation continuations. The next
    // unresolved gate produces a notice-only stop and disarms until new open work appears.
    let mut budget_policy = FollowUpPolicy::new(true);
    for attempt in 0..MAX_GATE_ATTEMPTS {
        let unresolved = snapshot(vec![done(
            &format!("ship {attempt}"),
            ConfidenceState::Speculative,
        )]);
        let decision = budget_policy.decide(&unresolved);
        assert_eq!(kind(&decision), Some(FollowUpKind::CompletionValidation));
        assert!(decision.notice.is_none());
        assert!(decision.effects.is_empty());
    }
    let unresolved = snapshot(vec![done(
        "ship beyond budget",
        ConfidenceState::Speculative,
    )]);
    let stopped = budget_policy.decide(&unresolved);
    assert!(stopped.follow_up.is_none());
    assert!(stopped.notice.is_some());
    assert!(stopped.effects.is_empty());
    assert!(budget_policy.is_enabled());
    assert!(!budget_policy.armed);
    assert!(budget_policy.default_on);
    assert_eq!(budget_policy.gate_attempts, 0);
    assert_eq!(budget_policy.decide(&unresolved), Decision::default());
    let new_work = budget_policy.decide(&snapshot(vec![todo("new", "pending")]));
    assert_eq!(kind(&new_work), Some(FollowUpKind::IncompleteTodos));

    let mut disabled = FollowUpPolicy::new(true);
    disabled.disable();
    let disabled_decision = disabled.decide(&snapshot(vec![todo("pending", "pending")]));
    assert_eq!(disabled_decision, Decision::default());
    assert!(!disabled.is_enabled());
}

#[test]
fn snapshot_round_trips_through_json() {
    let state = TodoSnapshot {
        todos: vec![todo("a", "pending")],
        long_session_review_due: true,
        ..Default::default()
    };
    let json = serde_json::to_string(&state).unwrap();
    assert_eq!(serde_json::from_str::<TodoSnapshot>(&json).unwrap(), state);
    assert_eq!(
        serde_json::from_str::<TodoSnapshot>("{}").unwrap(),
        TodoSnapshot::default()
    );
}
