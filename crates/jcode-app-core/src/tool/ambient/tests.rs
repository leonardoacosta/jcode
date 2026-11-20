use super::*;

#[test]
fn schedule_description_stays_one_short_sentence() {
    let tool = ScheduleTool::new();
    let description = tool.description();
    assert!(description.contains("independent future tasks"));
    assert!(!description.contains("blocker"));
    assert!(crate::util::estimate_tokens(description) <= 20);
    let schema = tool.parameters_schema();
    for action in ["create", "list", "cancel"] {
        assert!(
            schema["properties"]["action"]["enum"]
                .as_array()
                .expect("action choices")
                .contains(&json!(action))
        );
    }
}

#[test]
fn test_parse_priority() {
    assert_eq!(parse_priority(Some("low")), Priority::Low);
    assert_eq!(parse_priority(Some("normal")), Priority::Normal);
    assert_eq!(parse_priority(Some("high")), Priority::High);
    assert_eq!(parse_priority(None), Priority::Normal);
    assert_eq!(parse_priority(Some("unknown")), Priority::Normal);
}

#[test]
fn test_cycle_result_store_and_take() {
    let result = AmbientCycleResult {
        summary: "test".to_string(),
        memories_modified: 1,
        compactions: 0,
        proactive_work: None,
        next_schedule: None,
        started_at: Utc::now(),
        ended_at: Utc::now(),
        status: CycleStatus::Complete,
        conversation: None,
    };

    store_cycle_result(result);
    let taken = take_cycle_result();
    assert!(taken.is_some());
    assert_eq!(taken.unwrap().summary, "test");

    // Second take should be None
    assert!(take_cycle_result().is_none());
}

#[test]
fn test_end_cycle_input_deserialization() {
    let input = json!({
        "summary": "Merged 3 duplicates",
        "memories_modified": 5,
        "compactions": 1,
        "proactive_work": "Fixed typo in README",
        "next_schedule": {
            "wake_in_minutes": 20,
            "context": "Verify stale facts",
            "priority": "high"
        }
    });

    let parsed: EndCycleInput = serde_json::from_value(input).unwrap();
    assert_eq!(parsed.summary, "Merged 3 duplicates");
    assert_eq!(parsed.memories_modified, 5);
    assert_eq!(parsed.compactions, 1);
    assert_eq!(
        parsed.proactive_work.as_deref(),
        Some("Fixed typo in README")
    );
    let ns = parsed.next_schedule.unwrap();
    assert_eq!(ns.wake_in_minutes, Some(20));
    assert_eq!(ns.context.as_deref(), Some("Verify stale facts"));
    assert_eq!(ns.priority.as_deref(), Some("high"));
}

#[test]
fn test_end_cycle_input_minimal() {
    let input = json!({
        "summary": "Nothing to do",
        "memories_modified": 0,
        "compactions": 0
    });

    let parsed: EndCycleInput = serde_json::from_value(input).unwrap();
    assert_eq!(parsed.summary, "Nothing to do");
    assert!(parsed.proactive_work.is_none());
    assert!(parsed.next_schedule.is_none());
}

/// Regression for #106: Claude's tool calling emits numeric arguments as JSON
/// *strings* (e.g. `{"compactions": "0"}`). Before the fix, this failed with
/// `invalid type: string "0", expected u32`, breaking every ambient cycle.
#[test]
fn test_end_cycle_input_accepts_string_numbers() {
    let input = json!({
        "summary": "Stringified counts",
        "memories_modified": "5",
        "compactions": "0",
        "next_schedule": {
            "wake_in_minutes": "20",
            "context": "later",
            "priority": "high"
        }
    });

    let parsed: EndCycleInput = serde_json::from_value(input)
        .expect("string-encoded numbers must deserialize (regression #106)");
    assert_eq!(parsed.memories_modified, 5);
    assert_eq!(parsed.compactions, 0);
    assert_eq!(parsed.next_schedule.unwrap().wake_in_minutes, Some(20));
}

/// The `schedule_ambient` and `schedule` tools and the permission `wait` flag
/// must survive the same stringified-argument quirk.
#[test]
fn test_ambient_inputs_accept_string_numbers_and_bools() {
    let sched: ScheduleInput = serde_json::from_value(json!({
        "wake_in_minutes": "15",
        "context": "ctx"
    }))
    .expect("schedule_ambient must accept string wake_in_minutes (#106)");
    assert_eq!(sched.wake_in_minutes, Some(15));

    let perm: RequestPermissionInput = serde_json::from_value(json!({
        "action": "delete",
        "description": "remove file",
        "rationale": "cleanup",
        "wait": "true"
    }))
    .expect("request_permission must accept string wait flag (#106)");
    assert!(perm.wait);

    let tool: ScheduleToolInput = serde_json::from_value(json!({
        "task": "do thing",
        "wake_in_minutes": "30"
    }))
    .expect("schedule tool must accept string wake_in_minutes (#106)");
    assert_eq!(tool.wake_in_minutes, Some(30));
}

#[test]
fn test_schedule_input_deserialization() {
    let input = json!({
        "wake_in_minutes": 15,
        "context": "Check CI results",
        "priority": "normal"
    });

    let parsed: ScheduleInput = serde_json::from_value(input).unwrap();
    assert_eq!(parsed.wake_in_minutes, Some(15));
    assert!(parsed.wake_at.is_none());
    assert_eq!(parsed.context, "Check CI results");
    assert_eq!(parsed.priority.as_deref(), Some("normal"));
}

#[test]
fn test_permission_input_deserialization() {
    let input = json!({
        "action": "create_pull_request",
        "description": "Create PR for test fixes",
        "rationale": "Found failing tests that need attention",
        "urgency": "high",
        "wait": true
    });

    let parsed: RequestPermissionInput = serde_json::from_value(input).unwrap();
    assert_eq!(parsed.action, "create_pull_request");
    assert_eq!(parsed.description, "Create PR for test fixes");
    assert_eq!(parsed.rationale, "Found failing tests that need attention");
    assert_eq!(parsed.urgency.as_deref(), Some("high"));
    assert!(parsed.wait);
}

#[test]
fn test_permission_input_defaults() {
    let input = json!({
        "action": "edit",
        "description": "Fix typo",
        "rationale": "Obvious error"
    });

    let parsed: RequestPermissionInput = serde_json::from_value(input).unwrap();
    assert!(parsed.urgency.is_none());
    assert!(!parsed.wait);
}

#[test]
fn test_build_permission_review_context_defaults() {
    let review =
        build_permission_review_context("edit", "Fix typo in docs", "Needs write permission", None);

    assert_eq!(
        review
            .get("summary")
            .and_then(|v| v.as_str())
            .unwrap_or_default(),
        "Fix typo in docs"
    );
    assert_eq!(
        review
            .get("why_permission_needed")
            .and_then(|v| v.as_str())
            .unwrap_or_default(),
        "Needs write permission"
    );
    assert_eq!(
        review
            .get("requested_action")
            .and_then(|v| v.as_str())
            .unwrap_or_default(),
        "edit"
    );
}

#[test]
fn test_build_permission_review_context_uses_structured_fields() {
    let context = json!({
        "summary": "Preparing a focused refactor",
        "why_permission_needed": "Need to modify tracked files",
        "planned_steps": ["Update parser", "Run tests"],
        "files": ["src/parser.rs", "src/tests.rs"],
        "commands": ["cargo test"],
        "risks": ["Could regress parsing edge cases"],
        "rollback_plan": "Revert commit if tests fail",
        "expected_outcome": "Parser handles edge-case input",
    });
    let review =
        build_permission_review_context("edit", "fallback summary", "fallback why", Some(&context));

    assert_eq!(
        review
            .get("summary")
            .and_then(|v| v.as_str())
            .unwrap_or_default(),
        "Preparing a focused refactor"
    );
    assert_eq!(
        review
            .get("why_permission_needed")
            .and_then(|v| v.as_str())
            .unwrap_or_default(),
        "Need to modify tracked files"
    );
    assert_eq!(
        review
            .get("rollback_plan")
            .and_then(|v| v.as_str())
            .unwrap_or_default(),
        "Revert commit if tests fail"
    );
    assert_eq!(
        review
            .get("planned_steps")
            .and_then(|v| v.as_array())
            .map(|a| a.len())
            .unwrap_or_default(),
        2
    );
}

#[test]
fn test_register_unregister_ambient_session() {
    let session_id = "ambient_tool_test_session";
    unregister_ambient_session(session_id);
    assert!(!is_ambient_session_registered(session_id));

    register_ambient_session(session_id.to_string());
    assert!(is_ambient_session_registered(session_id));

    unregister_ambient_session(session_id);
    assert!(!is_ambient_session_registered(session_id));
}

#[tokio::test]
async fn test_request_permission_rejects_non_ambient_session() {
    let tool = RequestPermissionTool::new();
    let input = json!({
        "action": "edit",
        "description": "Update docs",
        "rationale": "Fix typo"
    });
    let ctx = ToolContext {
        session_id: "normal_session_test".to_string(),
        message_id: "msg_1".to_string(),
        tool_call_id: "call_1".to_string(),
        working_dir: None,
        stdin_request_tx: None,
        graceful_shutdown_signal: None,
        execution_mode: crate::tool::ToolExecutionMode::Direct,
    };

    let err = tool
        .execute(input, ctx)
        .await
        .expect_err("non-ambient session should be rejected");
    assert!(
        err.to_string()
            .contains("request_permission is only available to ambient sessions")
    );
}

#[test]
fn test_schedule_tool_input_deserialization() {
    let input = json!({
        "task": "Run the full test suite and report results",
        "wake_in_minutes": 120,
        "priority": "high",
        "relevant_files": ["src/main.rs", "tests/e2e/main.rs"],
        "background_context": "We just merged PR #42 which changed the parser",
        "success_criteria": "All tests pass, or a summary of failures is stored"
    });

    let parsed: ScheduleToolInput = serde_json::from_value(input).unwrap();
    assert_eq!(
        parsed.task.as_deref(),
        Some("Run the full test suite and report results")
    );
    assert!(parsed.action.is_none());
    assert!(parsed.schedule_id.is_none());
    assert_eq!(parsed.wake_in_minutes, Some(120));
    assert!(parsed.wake_at.is_none());
    assert_eq!(parsed.priority.as_deref(), Some("high"));
    assert_eq!(parsed.relevant_files.len(), 2);
    assert_eq!(
        parsed.background_context.as_deref(),
        Some("We just merged PR #42 which changed the parser")
    );
    assert_eq!(
        parsed.success_criteria.as_deref(),
        Some("All tests pass, or a summary of failures is stored")
    );
}

#[test]
fn test_schedule_tool_input_resume_target() {
    let input = json!({
        "task": "Follow up in this chat",
        "wake_in_minutes": 10,
        "target": "resume"
    });

    let parsed: ScheduleToolInput = serde_json::from_value(input).unwrap();
    assert_eq!(parsed.target.as_deref(), Some("resume"));
}

#[test]
fn test_schedule_tool_input_spawn_target() {
    let input = json!({
        "task": "Follow up in a new child session",
        "wake_in_minutes": 10,
        "target": "spawn"
    });

    let parsed: ScheduleToolInput = serde_json::from_value(input).unwrap();
    assert_eq!(parsed.target.as_deref(), Some("spawn"));
}

#[test]
fn test_schedule_tool_input_minimal() {
    let input = json!({
        "task": "Check CI",
        "wake_in_minutes": 30
    });

    let parsed: ScheduleToolInput = serde_json::from_value(input).unwrap();
    assert_eq!(parsed.task.as_deref(), Some("Check CI"));
    assert_eq!(parsed.wake_in_minutes, Some(30));
    assert!(parsed.relevant_files.is_empty());
    assert!(parsed.background_context.is_none());
    assert!(parsed.success_criteria.is_none());
}

#[test]
fn test_schedule_tool_input_cancel_action() {
    let input = json!({
        "action": "cancel",
        "schedule_id": "sched_abc123"
    });

    let parsed: ScheduleToolInput = serde_json::from_value(input).unwrap();
    assert_eq!(parsed.action.as_deref(), Some("cancel"));
    assert_eq!(parsed.schedule_id.as_deref(), Some("sched_abc123"));
    assert!(parsed.task.is_none());
}

#[test]
fn test_parse_schedule_target_defaults_to_resume_originating_session() {
    assert_eq!(
        parse_schedule_target(None, "session_123").unwrap(),
        ScheduleTarget::Session {
            session_id: "session_123".to_string()
        }
    );
    assert_eq!(
        parse_schedule_target(Some("resume"), "session_123").unwrap(),
        ScheduleTarget::Session {
            session_id: "session_123".to_string()
        }
    );
}

#[test]
fn test_parse_schedule_target_supports_spawn_and_ambient() {
    assert_eq!(
        parse_schedule_target(Some("spawn"), "session_123").unwrap(),
        ScheduleTarget::Spawn {
            parent_session_id: "session_123".to_string()
        }
    );
    assert_eq!(
        parse_schedule_target(Some("ambient"), "session_123").unwrap(),
        ScheduleTarget::Ambient
    );
}

#[test]
fn test_parse_schedule_target_rejects_removed_session_alias() {
    let err = parse_schedule_target(Some("session"), "session_123")
        .expect_err("removed session alias should be rejected");
    assert!(err.to_string().contains("resume, spawn, ambient"));
}

#[tokio::test]
#[allow(
    clippy::await_holding_lock,
    reason = "test intentionally serializes process-wide JCODE_HOME/env state across async tool execution"
)]
async fn test_schedule_tool_defaults_to_resuming_originating_session() {
    let _guard = crate::storage::lock_test_env();
    let temp = tempfile::tempdir().expect("tempdir");
    let prev_home = std::env::var_os("JCODE_HOME");
    crate::env::set_var("JCODE_HOME", temp.path());

    let tool = ScheduleTool::new();
    let input = json!({
        "task": "Follow up on this work",
        "wake_in_minutes": 5
    });
    let ctx = ToolContext {
        session_id: "origin_session".to_string(),
        message_id: "msg_1".to_string(),
        tool_call_id: "call_1".to_string(),
        working_dir: None,
        stdin_request_tx: None,
        graceful_shutdown_signal: None,
        execution_mode: crate::tool::ToolExecutionMode::Direct,
    };

    let output = tool
        .execute(input, ctx)
        .await
        .expect("schedule should succeed");
    assert!(
        output
            .output
            .contains("Target: resume session origin_session")
    );

    let manager = AmbientManager::new().expect("ambient manager");
    let scheduled = manager
        .queue()
        .items()
        .first()
        .expect("scheduled item should exist");
    assert_eq!(
        scheduled.target,
        ScheduleTarget::Session {
            session_id: "origin_session".to_string()
        }
    );

    if let Some(prev) = prev_home {
        crate::env::set_var("JCODE_HOME", prev);
    } else {
        crate::env::remove_var("JCODE_HOME");
    }
}

#[test]
fn test_schedule_tool_schema_avoids_top_level_combinators() {
    let tool = ScheduleTool::new();
    let schema = tool.parameters_schema();

    assert_eq!(schema.get("type"), Some(&json!("object")));
    assert!(schema.get("anyOf").is_none());
    assert!(schema.get("oneOf").is_none());
    assert!(schema.get("allOf").is_none());
}

#[tokio::test]
async fn test_schedule_tool_requires_time() {
    let tool = ScheduleTool::new();
    let input = json!({
        "task": "Do something eventually"
    });
    let ctx = ToolContext {
        session_id: "test_session".to_string(),
        message_id: "msg_1".to_string(),
        tool_call_id: "call_1".to_string(),
        working_dir: None,
        stdin_request_tx: None,
        graceful_shutdown_signal: None,
        execution_mode: crate::tool::ToolExecutionMode::Direct,
    };

    let err = tool
        .execute(input, ctx)
        .await
        .expect_err("should require wake_in_minutes or wake_at");
    assert!(err.to_string().contains("wake_in_minutes"));
}

fn schedule_ctx() -> ToolContext {
    ToolContext {
        session_id: "origin_session".to_string(),
        message_id: "msg_1".to_string(),
        tool_call_id: "call_1".to_string(),
        working_dir: None,
        stdin_request_tx: None,
        graceful_shutdown_signal: None,
        execution_mode: crate::tool::ToolExecutionMode::Direct,
    }
}

struct HomeGuard(Option<std::ffi::OsString>);

impl HomeGuard {
    fn set(path: &std::path::Path) -> Self {
        let prev = std::env::var_os("JCODE_HOME");
        crate::env::set_var("JCODE_HOME", path);
        Self(prev)
    }
}

impl Drop for HomeGuard {
    fn drop(&mut self) {
        match self.0.take() {
            Some(prev) => crate::env::set_var("JCODE_HOME", prev),
            None => crate::env::remove_var("JCODE_HOME"),
        }
    }
}

fn overdue_request(hours: i64) -> ScheduleRequest {
    ScheduleRequest {
        wake_in_minutes: None,
        wake_at: Some(Utc::now() - chrono::Duration::hours(hours)),
        context: "overdue task".into(),
        priority: Priority::Normal,
        target: ScheduleTarget::Session {
            session_id: "origin_session".into(),
        },
        created_by_session: "origin_session".into(),
        working_dir: None,
        task_description: Some("overdue task".into()),
        relevant_files: vec![],
        git_branch: None,
        additional_context: None,
    }
}

#[tokio::test]
#[allow(
    clippy::await_holding_lock,
    reason = "test intentionally serializes process-wide JCODE_HOME/env state across async tool execution"
)]
async fn schedule_list_shows_missed_history_and_retry_warns() {
    let _guard = crate::storage::lock_test_env();
    let temp = tempfile::tempdir().expect("tempdir");
    let _home = HomeGuard::set(temp.path());

    let mut manager = AmbientManager::new().expect("manager");
    let id = manager.schedule(overdue_request(2)).expect("schedule");
    manager.recover_scheduled_runs().expect("recover");

    let tool = ScheduleTool::new();
    let listed = tool
        .execute(json!({"action": "list"}), schedule_ctx())
        .await
        .expect("list");
    assert!(
        listed.output.contains("Recent history"),
        "{}",
        listed.output
    );
    assert!(listed.output.contains(&format!("{id} | missed")));
    assert!(listed.output.contains("2h late, not delivered"));
    // Listing is read-only: nothing was claimed or delivered.
    assert!(AmbientManager::new().unwrap().queue().items().is_empty());

    let retried = tool
        .execute(
            json!({"action": "retry", "schedule_id": id}),
            schedule_ctx(),
        )
        .await
        .expect("retry");
    // A missed item never ran, so there is no duplicate-effect warning.
    assert!(
        retried.output.contains("never dispatched"),
        "{}",
        retried.output
    );
    assert!(!retried.output.contains("may already have run"));
    assert_eq!(AmbientManager::new().unwrap().queue().items().len(), 1);
    assert!(
        tool.execute(
            json!({"action": "retry", "schedule_id": id}),
            schedule_ctx()
        )
        .await
        .is_err(),
        "a pending item has nothing to retry"
    );
}

#[tokio::test]
#[allow(
    clippy::await_holding_lock,
    reason = "test intentionally serializes process-wide JCODE_HOME/env state across async tool execution"
)]
async fn schedule_list_shows_delivered_and_uncertain_and_retry_warns_of_prior_effects() {
    let _guard = crate::storage::lock_test_env();
    let temp = tempfile::tempdir().expect("tempdir");
    let _home = HomeGuard::set(temp.path());
    let mut manager = AmbientManager::new().expect("manager");
    let delivered = manager.schedule(overdue_request(0)).expect("schedule");
    let uncertain = manager.schedule(overdue_request(0)).expect("schedule");
    let claimed = manager.take_ready_direct_items();
    assert_eq!(claimed.len(), 2);
    manager
        .settle(
            &delivered,
            crate::ambient::Settlement::Acked {
                target: "origin_session".into(),
                generation: None,
            },
        )
        .unwrap();
    manager
        .settle(
            &uncertain,
            crate::ambient::Settlement::Uncertain {
                note: "no ack".into(),
            },
        )
        .unwrap();

    let tool = ScheduleTool::new();
    let listed = tool
        .execute(json!({"action": "list"}), schedule_ctx())
        .await
        .expect("list");
    let delivered_line = listed
        .output
        .lines()
        .find(|line| line.contains(&format!("{delivered} | delivered")))
        .unwrap_or_else(|| panic!("no delivered line:\n{}", listed.output));
    assert!(delivered_line.contains("not task success"));
    let uncertain_line = listed
        .output
        .lines()
        .find(|line| line.contains(&format!("{uncertain} | uncertain")))
        .unwrap_or_else(|| panic!("no uncertain line:\n{}", listed.output));
    assert!(uncertain_line.contains("it may have run"));

    let retried = tool
        .execute(
            json!({"action": "retry", "schedule_id": uncertain}),
            schedule_ctx(),
        )
        .await
        .expect("retry uncertain");
    assert!(retried.output.contains("may already have run"));
    assert!(retried.output.contains("duplicate"));

    // An uncertain item can also be dismissed, keeping its attempt history.
    let mut manager = AmbientManager::new().unwrap();
    manager.take_ready_direct_items();
    manager
        .settle(
            &uncertain,
            crate::ambient::Settlement::Uncertain {
                note: "again".into(),
            },
        )
        .unwrap();
    let cancelled = tool
        .execute(
            json!({"action": "cancel", "schedule_id": uncertain}),
            schedule_ctx(),
        )
        .await
        .expect("cancel uncertain");
    assert!(cancelled.output.contains("Cancelled"));
    let manager = AmbientManager::new().unwrap();
    let kept = manager
        .queue()
        .recent_history(100)
        .into_iter()
        .find(|item| item.id == uncertain)
        .unwrap();
    assert_eq!(kept.delivery.state, crate::ambient::ItemState::Cancelled);
    assert_eq!(kept.delivery.attempts.len(), 2);
}

fn cron_ctx() -> ToolContext {
    ToolContext {
        session_id: "origin_session".to_string(),
        message_id: "msg_1".to_string(),
        tool_call_id: "call_1".to_string(),
        working_dir: None,
        stdin_request_tx: None,
        graceful_shutdown_signal: None,
        execution_mode: crate::tool::ToolExecutionMode::Direct,
    }
}

#[tokio::test]
#[expect(
    clippy::await_holding_lock,
    reason = "test intentionally serializes process-wide JCODE_HOME/env state across async tool execution"
)]
async fn cron_schedule_tool_controls_and_preview_are_persisted_or_read_only() {
    let _guard = crate::storage::lock_test_env();
    let temp = tempfile::tempdir().expect("tempdir");
    let prev_home = std::env::var_os("JCODE_HOME");
    crate::env::set_var("JCODE_HOME", temp.path());
    let tool = ScheduleTool::new();
    let run = |input: Value| {
        let tool = &tool;
        async move { tool.execute(input, cron_ctx()).await }
    };
    let store = || crate::ambient::recurring::RecurringStore::open_default().unwrap();

    // Invalid definitions persist nothing.
    for bad in [
        json!({"task": "t", "cron": "61 * * * *", "timezone": "UTC", "missed_policy": "skip"}),
        json!({"task": "t", "cron": "0 9 * * *", "timezone": "Nope/Zone", "missed_policy": "skip"}),
        json!({"task": "t", "cron": "0 9 * * *", "timezone": "UTC"}),
        json!({"task": "t", "cron": "0 9 * * *", "missed_policy": "skip"}),
        json!({"task": "t", "cron": "0 9 * * *", "timezone": "UTC", "missed_policy": "catch-up"}),
        json!({"task": "t", "cron": "0 9 * * *", "timezone": "UTC", "missed_policy": "skip", "max_active": 0}),
        json!({"task": "t", "cron": "0 9 * * *", "timezone": "UTC", "missed_policy": "skip", "wake_in_minutes": 5}),
    ] {
        assert!(run(bad).await.is_err());
    }
    assert!(store().list().unwrap().is_empty());

    let created = run(json!({
        "task": "nightly build", "cron": "30 2 * * *", "timezone": "America/New_York",
        "missed_policy": "run-once", "name": "nightly", "spend_cap_usd": 5.0
    }))
    .await
    .expect("create recurring");
    let id = store().list().unwrap()[0].id.clone();
    assert!(created.output.contains(&id));
    assert!(created.output.contains("America/New_York"));
    assert!(created.output.contains("budget_unavailable"));

    // Preview is read-only: store bytes and queue are unchanged.
    let snapshot = std::fs::read(temp.path().join("ambient/schedules.json")).unwrap();
    let preview = run(json!({"action": "preview", "schedule_id": id, "count": 3}))
        .await
        .unwrap();
    assert_eq!(preview.output.matches("next ").count(), 3);
    let ad_hoc = run(json!({"action": "preview", "cron": "*/5 * * * *", "timezone": "UTC"}))
        .await
        .unwrap();
    assert_eq!(ad_hoc.output.matches("UTC").count(), 5 + 1);
    assert_eq!(
        std::fs::read(temp.path().join("ambient/schedules.json")).unwrap(),
        snapshot
    );
    assert!(AmbientManager::new().unwrap().queue().items().is_empty());

    // Pause and resume persist and are reflected in list.
    run(json!({"action": "pause", "schedule_id": id}))
        .await
        .unwrap();
    assert!(store().get(&id).unwrap().unwrap().paused);
    assert!(
        run(json!({"action": "list"}))
            .await
            .unwrap()
            .output
            .contains("paused")
    );
    run(json!({"action": "resume", "schedule_id": id}))
        .await
        .unwrap();
    assert!(!store().get(&id).unwrap().unwrap().paused);
    assert!(
        run(json!({"action": "pause", "schedule_id": "cron_missing"}))
            .await
            .is_err()
    );

    // Recurring-only fields on a one-shot create are rejected, not ignored.
    assert!(
        run(json!({"task": "x", "wake_in_minutes": 5, "max_active": 3}))
            .await
            .is_err()
    );
    // Legacy one-shot behavior is untouched.
    run(json!({"task": "once", "wake_in_minutes": 5}))
        .await
        .unwrap();
    assert_eq!(AmbientManager::new().unwrap().queue().items().len(), 1);
    assert_eq!(store().list().unwrap().len(), 1);

    // Cancel by recurring id removes the definition.
    run(json!({"action": "cancel", "schedule_id": id}))
        .await
        .unwrap();
    assert!(store().list().unwrap().is_empty());

    if let Some(prev) = prev_home {
        crate::env::set_var("JCODE_HOME", prev);
    } else {
        crate::env::remove_var("JCODE_HOME");
    }
}
