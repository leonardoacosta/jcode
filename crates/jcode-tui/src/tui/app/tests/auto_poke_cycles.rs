#[test]
fn auto_poke_stays_armed_when_a_turn_has_no_todos() {
    with_temp_jcode_home(|| {
        let mut app = create_test_app();
        app.auto_poke_incomplete_todos = true;
        app.auto_poke_default_on = true;

        assert!(
            !app.schedule_auto_poke_followup_if_needed(),
            "no todos means nothing to poke about this turn"
        );
        assert!(
            app.auto_poke_incomplete_todos,
            "a todo-free turn must not disable the default-on feature"
        );

        // Later work with incomplete todos must still be poked.
        crate::todo::save_todos(
            &app.session.id,
            &[crate::todo::TodoItem {
                id: "todo-1".to_string(),
                content: "Finish the thing".to_string(),
                status: "pending".to_string(),
                priority: "high".to_string(),
                confidence: Some(crate::todo::ConfidenceState::from_legacy_score(80)),
                ..Default::default()
            }],
        )
        .expect("save incomplete todo");

        assert!(
            app.schedule_auto_poke_followup_if_needed(),
            "incomplete todos on a later turn must still schedule a poke"
        );
    });
}

#[test]
fn auto_poke_does_not_repeat_until_incomplete_todos_change() {
    with_temp_jcode_home(|| {
        let mut app = create_test_app();
        app.auto_poke_incomplete_todos = true;
        let pending = |content: &str| crate::todo::TodoItem {
            id: "todo-1".to_string(),
            content: content.to_string(),
            status: "pending".to_string(),
            priority: "high".to_string(),
            confidence: Some(crate::todo::ConfidenceState::from_legacy_score(80)),
            ..Default::default()
        };

        crate::todo::save_todos(&app.session.id, &[pending("Wait for worker")]).expect("save");
        assert!(app.schedule_auto_poke_followup_if_needed());

        // Simulate dispatch and completion of the automatically poked turn.
        app.queued_messages.clear();
        app.pending_queued_dispatch = false;
        assert!(
            !app.schedule_auto_poke_followup_if_needed(),
            "an unchanged list must not consume another model turn"
        );

        crate::todo::save_todos(&app.session.id, &[pending("Review worker result")])
            .expect("update");
        assert!(
            app.schedule_auto_poke_followup_if_needed(),
            "changing the todo list must re-arm the automatic nudge"
        );

        app.queued_messages.clear();
        app.pending_queued_dispatch = false;
        crate::todo::save_todos(&app.session.id, &[]).expect("finish cycle");
        assert!(!app.schedule_auto_poke_followup_if_needed());
        crate::todo::save_todos(&app.session.id, &[pending("Review worker result")])
            .expect("start equivalent new cycle");
        assert!(
            app.schedule_auto_poke_followup_if_needed(),
            "an equivalent todo in a new cycle must receive one fresh nudge"
        );
    });
}

#[test]
fn completed_cycle_rearms_auto_poke_only_when_default_on() {
    with_temp_jcode_home(|| {
        let completed = |id: &str| crate::todo::TodoItem {
            id: id.to_string(),
            content: "Done".to_string(),
            status: "completed".to_string(),
            priority: "high".to_string(),
            confidence: Some(crate::todo::ConfidenceState::from_legacy_score(100)),
            completion_confidence: Some(crate::todo::ConfidenceState::from_legacy_score(100)),
            confidence_history: vec![
                crate::todo::ConfidenceState::Validated,
                crate::todo::ConfidenceState::Verified,
            ],
            ..Default::default()
        };

        let mut app = create_test_app();
        app.auto_poke_incomplete_todos = true;
        app.auto_poke_default_on = true;
        crate::todo::save_todos(&app.session.id, &[completed("todo-1")]).expect("save");
        crate::todo::save_goals(
            &app.session.id,
            &[crate::todo::TodoGoal {
                delivery_state: Some(crate::todo::DeliveryState::WorkflowValidated),
                autonomy: Some(crate::todo::Autonomy::NecessaryFollowthrough),
                iteration_maturity: Some(crate::todo::IterationMaturity::OutcomeReached),
                feedback_loop_relevance: Some(crate::todo::FeedbackLoopRelevance::Representative),
                feedback_loop_coverage: Some(crate::todo::FeedbackLoopCoverage::MainPaths),
                feedback_loop_traceability: Some(crate::todo::FeedbackLoopTraceability::Complete),
                ..Default::default()
            }],
        )
        .expect("save passing goal");
        assert!(app.schedule_auto_poke_followup_if_needed());
        assert_eq!(
            app.queued_messages,
            vec![crate::todo::TODO_FINAL_RESPONSE_CONTINUATION_MESSAGE.to_string()]
        );
        app.queued_messages.clear();
        app.pending_queued_dispatch = false;
        assert!(!app.schedule_auto_poke_followup_if_needed());
        assert!(
            app.auto_poke_incomplete_todos,
            "default-on auto-poke should cover the next batch of work too"
        );

        // The final handoff must stay latched across a client reload too.
        app.save_input_for_reload(&app.session.id);
        let restored = App::restore_input_for_reload(&app.session.id)
            .expect("final handoff should survive reload");
        let mut reloaded = create_test_app();
        reloaded.session.id = app.session.id.clone();
        reloaded.auto_poke_incomplete_todos = true;
        reloaded.auto_poke_default_on = true;
        reloaded.apply_restored_reload_input(restored);
        assert_eq!(
            reloaded.final_response_todo_fingerprint,
            app.final_response_todo_fingerprint
        );
        assert!(reloaded.todo_final_response_requested);
        app = reloaded;

        // An overdue timed review alone must not restart finished work.
        let review_path = crate::storage::jcode_dir()
            .expect("jcode home")
            .join("todos")
            .join(format!("{}-review-state.json", app.session.id));
        crate::storage::write_json_fast(
            &review_path,
            &serde_json::json!({
                "cycle_started_at": "2020-01-01T00:00:00Z",
                "review_delivered": false,
            }),
        )
        .expect("age review");

        // A late observation must not resurrect gates after the final answer,
        // even when the scheduler is called repeatedly with the same todo state.
        crate::todo::append_gate_observations(
            &app.session.id,
            &[crate::todo::GateObservation {
                kind: crate::todo::GateObservationKind::IntentUnderstanding,
                group: None,
                state: Some("uncertain".to_string()),
            }],
        )
        .expect("record late observation");
        for _ in 0..3 {
            assert!(!app.schedule_auto_poke_followup_if_needed());
            assert!(app.queued_messages.is_empty());
            assert!(!app.pending_queued_dispatch);
        }

        // Rewriting identical data is not a new cycle either.
        crate::todo::save_todos(&app.session.id, &[completed("todo-1")]).expect("save");
        assert!(!app.schedule_auto_poke_followup_if_needed());

        assert!(
            crate::todo::take_long_session_review_if_due(&app.session.id)
                .expect("the final-response guard must not consume the timed review")
        );

        // A real todo change re-arms the quality checks and final handoff.
        let mut changed = completed("todo-1");
        changed.completion_confidence = Some(crate::todo::ConfidenceState::Plausible);
        crate::todo::save_todos(&app.session.id, &[changed]).expect("save changed todo");
        assert!(app.schedule_auto_poke_followup_if_needed());
        assert!(app.queued_messages[0].starts_with(crate::todo::TODO_GATE_DIGEST_PREFIX));
        assert!(!app.todo_final_response_requested);
        app.queued_messages.clear();
        app.pending_queued_dispatch = false;
        assert!(app.schedule_auto_poke_followup_if_needed());
        assert!(
            app.queued_messages[0].starts_with(crate::todo::TODO_COMPLETION_CONTINUATION_MESSAGE)
        );
        app.queued_messages.clear();
        app.pending_queued_dispatch = false;
        crate::todo::save_todos(&app.session.id, &[completed("todo-1")]).expect("save validated");
        assert!(app.schedule_auto_poke_followup_if_needed());
        assert_eq!(
            app.queued_messages,
            vec![crate::todo::TODO_FINAL_RESPONSE_CONTINUATION_MESSAGE.to_string()]
        );

        // An explicit /poke off must stick.
        let mut app = create_test_app();
        app.auto_poke_incomplete_todos = true;
        app.auto_poke_default_on = true;
        crate::tui::app::commands::disable_auto_poke(&mut app);
        crate::todo::save_todos(&app.session.id, &[completed("todo-2")]).expect("save");
        crate::todo::save_goals(
            &app.session.id,
            &[crate::todo::TodoGoal {
                delivery_state: Some(crate::todo::DeliveryState::WorkflowValidated),
                autonomy: Some(crate::todo::Autonomy::NecessaryFollowthrough),
                iteration_maturity: Some(crate::todo::IterationMaturity::OutcomeReached),
                feedback_loop_relevance: Some(crate::todo::FeedbackLoopRelevance::Representative),
                feedback_loop_coverage: Some(crate::todo::FeedbackLoopCoverage::MainPaths),
                feedback_loop_traceability: Some(crate::todo::FeedbackLoopTraceability::Complete),
                ..Default::default()
            }],
        )
        .expect("save passing goal");
        app.auto_poke_incomplete_todos = true; // pretend a stale arm survived
        app.todo_final_response_requested = true; // final handoff already delivered
        assert!(!app.schedule_auto_poke_followup_if_needed());
        assert!(
            !app.auto_poke_incomplete_todos,
            "/poke off must not be undone by the default-on re-arm"
        );
    });
}

#[test]
fn peer_auto_poke_is_shown_and_suppresses_duplicate_local_poke() {
    with_temp_jcode_home(|| {
        let mut app = create_test_app();
        app.auto_poke_incomplete_todos = true;
        app.auto_poke_default_on = true;
        crate::todo::save_todos(
            &app.session.id,
            &[crate::todo::TodoItem {
                id: "todo-1".to_string(),
                content: "Send the email".to_string(),
                status: "pending".to_string(),
                priority: "high".to_string(),
                confidence: Some(crate::todo::ConfidenceState::from_legacy_score(80)),
                ..Default::default()
            }],
        )
        .expect("save incomplete todo");

        let poke = crate::todo::build_auto_poke_message(1);
        app.handle_peer_user_message(&poke);

        assert!(
            app.display_messages()
                .iter()
                .any(|msg| msg.role == "system" && msg.content.contains("1 incomplete todo")),
            "a poke sent by another window must be shown here too"
        );
        assert!(
            !app.display_messages().iter().any(|msg| msg.role == "user"),
            "a peer poke must not render as a user prompt"
        );
        assert!(
            !app.schedule_auto_poke_followup_if_needed(),
            "this window must not re-poke the same open work a peer already poked"
        );
    });
}

// Synthetic native TUI scheduler-only baseline. This observes local TUI decisions and
// persisted-state effects, not inference acceptance: absent/unmatched gateway
// traffic has no attributable scheduler meaning, and claimed confidence,
// ToolDone, or PASS text alone cannot establish accepted completion. The
// scheduler cannot attribute missing, stale, or conflicting external identity.
// Tool-level errors are not scheduler inputs; the error case below uses a real
// persisted review-state read failure, not a fabricated ToolDone input.
// Expected behavior below is the current baseline, not a loop-fix or a parity
// promise for another engine.
#[test]
fn continuation_baseline_auto_poke_repeated_and_required() {
    with_temp_jcode_home(|| {
        let mut app = create_test_app();
        app.auto_poke_incomplete_todos = true;
        app.auto_poke_default_on = true;
        app.todo_completion_gate_attempts = 2;
        let pending = |id: &str, content: &str| crate::todo::TodoItem {
            id: id.to_string(),
            content: content.to_string(),
            status: "pending".to_string(),
            priority: "high".to_string(),
            confidence: Some(crate::todo::ConfidenceState::from_legacy_score(80)),
            ..Default::default()
        };

        crate::todo::save_todos(&app.session.id, &[pending("todo-1", "Review the result")])
            .expect("save synthetic open todo");
        assert!(app.schedule_auto_poke_followup_if_needed());
        assert_eq!(
            app.queued_messages,
            vec![crate::todo::build_auto_poke_message(1)]
        );
        assert!(app.pending_queued_dispatch);
        assert_eq!(app.todo_completion_gate_attempts, 0);
        let first_fingerprint = app.last_auto_poke_fingerprint.clone();
        let notice = app
            .display_messages()
            .iter()
            .find(|message| message.content.contains("1 incomplete todo"))
            .expect("scheduler displays its queue decision");
        assert_eq!(notice.role, "system");

        // Model a consumed queue boundary without claiming that the model
        // accepted or completed the prompt. Goal/plan metadata alone does not
        // change the open-todo fingerprint used by this scheduler path.
        app.queued_messages.clear();
        app.pending_queued_dispatch = false;
        crate::todo::save_goals(
            &app.session.id,
            &[crate::todo::TodoGoal {
                feedback_loop: Some("metadata-only assessment change".to_string()),
                ..Default::default()
            }],
        )
        .expect("save synthetic metadata-only goal");
        crate::todo::save_plan(
            &app.session.id,
            &crate::todo::TodoPlan {
                user_intention: Some("metadata-only plan change".to_string()),
                ..Default::default()
            },
        )
        .expect("save synthetic metadata-only plan");
        assert!(!app.schedule_auto_poke_followup_if_needed());
        assert!(app.queued_messages.is_empty());
        assert!(!app.pending_queued_dispatch);
        assert_eq!(app.todo_completion_gate_attempts, 0);
        assert!(app.last_auto_poke_fingerprint.is_some());

        // Existing queued work wins even when new required work appears.
        // Consume that queue boundary before asking for a new continuation.
        let prior = "synthetic user follow-up".to_string();
        app.queued_messages.push(prior.clone());
        crate::todo::save_todos(
            &app.session.id,
            &[
                pending("todo-1", "Review the result"),
                pending("todo-2", "Verify the required outcome"),
            ],
        )
        .expect("add synthetic required work");
        assert!(!app.schedule_auto_poke_followup_if_needed());
        assert_eq!(app.queued_messages, vec![prior]);
        assert!(!app.pending_queued_dispatch);
        assert_eq!(app.last_auto_poke_fingerprint, first_fingerprint);
        app.queued_messages.clear();
        assert!(app.schedule_auto_poke_followup_if_needed());
        assert_eq!(
            app.queued_messages,
            vec![crate::todo::build_auto_poke_message(2)]
        );
        assert!(app.pending_queued_dispatch);
        assert_eq!(app.todo_completion_gate_attempts, 0);
        assert_ne!(app.last_auto_poke_fingerprint, first_fingerprint);
        let messages = app.display_messages();
        assert!(
            messages.iter().any(|message| message.role == "system"
                && message.content.contains("2 incomplete todos"))
        );
    });
}

#[test]
fn continuation_baseline_auto_poke_wait_stop_and_errors() {
    with_temp_jcode_home(|| {
        let pending = || crate::todo::TodoItem {
            id: "todo-wait".to_string(),
            content: "Inspect the synthetic result".to_string(),
            status: "pending".to_string(),
            priority: "high".to_string(),
            confidence: Some(crate::todo::ConfidenceState::from_legacy_score(80)),
            ..Default::default()
        };

        // Missing todo state is read as empty work: no queue or fingerprint
        // effect, and default-on remains armed for later work.
        let mut missing = create_test_app();
        missing.auto_poke_default_on = true;
        assert!(!missing.schedule_auto_poke_followup_if_needed());
        assert!(missing.auto_poke_incomplete_todos);
        assert!(missing.queued_messages.is_empty());
        assert!(!missing.pending_queued_dispatch);
        assert!(missing.last_auto_poke_fingerprint.is_none());

        // Queued follow-up and pending-turn state are scheduler wait gates.
        // These inputs remain untouched for the other dispatch owner to read.
        for wait in ["queued", "pending turn"] {
            let mut app = create_test_app();
            app.auto_poke_incomplete_todos = true;
            app.auto_poke_default_on = true;
            crate::todo::save_todos(&app.session.id, &[pending()]).expect("save open todo");
            let queued = "already queued work".to_string();
            app.queued_messages.clear();
            if wait == "pending turn" {
                app.pending_turn = true;
            } else {
                app.queued_messages.push(queued.clone());
            }
            let before = app.queued_messages.clone();
            assert!(!app.schedule_auto_poke_followup_if_needed(), "{wait}");
            assert_eq!(app.queued_messages, before);
            assert!(app.auto_poke_incomplete_todos);
            assert!(!app.pending_queued_dispatch);
            assert!(app.last_auto_poke_fingerprint.is_none());
            assert_eq!(app.todo_completion_gate_attempts, 0);
        }

        // Explicit /poke off wins over the configured default, including the
        // pre-guard auto-rearm path. It leaves no generated message or ack.
        let mut stopped = create_test_app();
        stopped.auto_poke_incomplete_todos = true;
        stopped.auto_poke_default_on = true;
        crate::tui::app::commands::disable_auto_poke(&mut stopped);
        crate::todo::save_todos(&stopped.session.id, &[pending()]).expect("save open todo");
        assert!(!stopped.schedule_auto_poke_followup_if_needed());
        assert!(!stopped.auto_poke_incomplete_todos);
        assert!(!stopped.auto_poke_default_on);
        assert!(stopped.queued_messages.is_empty());
        assert!(!stopped.pending_queued_dispatch);
        assert!(stopped.last_auto_poke_fingerprint.is_none());

        // A changed todo invalidates stale final-response latches before a new
        // open-work continuation is queued; stale state is not treated as an
        // accepted completion receipt.
        let mut stale = create_test_app();
        stale.auto_poke_incomplete_todos = true;
        stale.auto_poke_default_on = true;
        stale.todo_final_response_requested = true;
        stale.final_response_todo_fingerprint = Some("stale synthetic fingerprint".to_string());
        stale.todo_completion_gate_attempts = 2;
        crate::todo::save_todos(&stale.session.id, &[pending()]).expect("save open todo");
        assert!(stale.schedule_auto_poke_followup_if_needed());
        assert!(!stale.todo_final_response_requested);
        assert!(stale.final_response_todo_fingerprint.is_none());
        assert_eq!(stale.todo_completion_gate_attempts, 0);
        assert_eq!(
            stale.queued_messages,
            vec![crate::todo::build_auto_poke_message(1)]
        );
        assert!(stale.pending_queued_dispatch);

        // A malformed review-state file is a real read error consumed by the
        // boundary: review errors become not-due; the ordinary open-work poke
        // proceeds, and the failed read does not acknowledge/overwrite it.
        let mut errored = create_test_app();
        errored.auto_poke_incomplete_todos = true;
        crate::todo::save_todos(&errored.session.id, &[pending()]).expect("save open todo");
        let review_path = crate::storage::jcode_dir()
            .expect("isolated jcode home")
            .join("todos")
            .join(format!("{}-review-state.json", errored.session.id));
        std::fs::create_dir_all(review_path.parent().expect("todo directory"))
            .expect("create synthetic todo directory");
        let malformed_review = b"{ reviewed synthetic malformed state";
        std::fs::write(&review_path, malformed_review).expect("write synthetic malformed state");
        assert!(errored.schedule_auto_poke_followup_if_needed());
        assert_eq!(
            errored.queued_messages,
            vec![crate::todo::build_auto_poke_message(1)]
        );
        assert!(errored.pending_queued_dispatch);
        assert_eq!(
            std::fs::read(&review_path).expect("review state remains"),
            malformed_review
        );

        // Due review state is read and acknowledged before the review message
        // is queued. This is a local persisted write, not an inference ack.
        let mut due = create_test_app();
        due.auto_poke_incomplete_todos = true;
        crate::todo::save_todos(&due.session.id, &[pending()]).expect("save open todo");
        let due_path = crate::storage::jcode_dir()
            .expect("isolated jcode home")
            .join("todos")
            .join(format!("{}-review-state.json", due.session.id));
        std::fs::create_dir_all(due_path.parent().expect("todo directory"))
            .expect("create synthetic todo directory");
        std::fs::write(
            &due_path,
            serde_json::to_vec(&serde_json::json!({
                "cycle_started_at": "2020-01-01T00:00:00Z",
                "review_delivered": false,
            }))
            .expect("encode due synthetic review"),
        )
        .expect("write due synthetic review");
        assert!(due.schedule_auto_poke_followup_if_needed());
        assert_eq!(
            due.queued_messages,
            vec![crate::todo::TODO_LONG_SESSION_REVIEW_MESSAGE.to_string()]
        );
        assert!(due.pending_queued_dispatch);
        let acknowledged: serde_json::Value =
            crate::storage::read_json(&due_path).expect("read acknowledged review state");
        assert_eq!(acknowledged["review_delivered"], true);
        assert!(
            due.display_messages()
                .iter()
                .any(|message| message.role == "system"
                    && message.content.contains("Rechecking the plan"))
        );
    });
}
