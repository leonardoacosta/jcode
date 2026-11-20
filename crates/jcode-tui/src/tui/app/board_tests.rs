//! `/board` glue tests: fake provider and a temp audit file only. No tracker,
//! MCP server or network is contacted.

use super::*;
use crate::board::{BoardBackend, BoardIssue as CoreIssue, BoardUnavailable, WorkflowState};
use async_trait::async_trait;
use std::sync::Mutex;

fn scope(backend: BoardBackend) -> BoardScope {
    BoardScope {
        world: "w".into(),
        backend,
        team: "team-1".into(),
        project: None,
        server: "Linear".into(),
        canonical_root: PathBuf::from("/r"),
    }
}

fn state(id: &str, name: &str, position: f64) -> WorkflowState {
    WorkflowState {
        id: id.into(),
        name: name.into(),
        position,
    }
}

fn issue(id: &str, state_id: &str) -> CoreIssue {
    CoreIssue {
        id: id.into(),
        identifier: format!("T-{id}"),
        title: format!("title {id}"),
        state_id: state_id.into(),
    }
}

#[derive(Default)]
struct Fake {
    live_state: Mutex<String>,
    set_calls: Mutex<Vec<(String, String)>>,
    set_result: Mutex<Option<BoardError>>,
}

#[async_trait]
impl BoardProvider for Fake {
    async fn workflow_states(&self, _: &BoardScope) -> Result<Vec<WorkflowState>, BoardError> {
        Ok(vec![])
    }
    async fn issues(&self, _: &BoardScope) -> Result<Vec<CoreIssue>, BoardError> {
        Ok(vec![])
    }
    async fn issue_state(&self, _: &BoardScope, _: &str) -> Result<String, BoardError> {
        Ok(self.live_state.lock().unwrap().clone())
    }
    async fn set_issue_state(
        &self,
        _: &BoardScope,
        issue: &str,
        state: &str,
    ) -> Result<(), BoardError> {
        self.set_calls
            .lock()
            .unwrap()
            .push((issue.into(), state.into()));
        match self.set_result.lock().unwrap().take() {
            Some(error) => Err(error),
            None => {
                *self.live_state.lock().unwrap() = state.into();
                Ok(())
            }
        }
    }
}

fn change(scope: BoardScope) -> StatusChangeRequest {
    StatusChangeRequest {
        scope,
        issue_id: "1".into(),
        expected_prior_state: "todo".into(),
        requested_state: "done".into(),
    }
}

fn audit_lines(path: &std::path::Path) -> usize {
    std::fs::read_to_string(path).map_or(0, |text| text.lines().count())
}

#[test]
fn board_glue_columns_follow_provider_order_and_unknown_states_go_to_other() {
    let snapshot = BoardSnapshot {
        states: vec![state("a", "Todo", 1.0), state("b", "Done", 2.0)],
        issues: vec![issue("1", "b"), issue("2", "a"), issue("3", "zzz")],
    };
    let BoardLoad::Ready(data) = to_board_load(&scope(BoardBackend::Linear), &snapshot) else {
        panic!("ready")
    };
    let names: Vec<_> = data.columns.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(names, ["Todo", "Done", "Other"]);
    assert_eq!(data.columns[0].issues[0].id, "2");
    assert_eq!(data.columns[1].issues[0].id, "1");
    assert_eq!(data.columns[2].issues[0].id, "3", "no issue is dropped");
    assert!(!data.read_only);
    assert!(data.scope_label.contains("team-wide"));
}

#[test]
fn board_glue_empty_and_read_only_and_errors_map_to_distinct_states() {
    let empty = BoardSnapshot {
        states: vec![state("a", "Todo", 1.0)],
        issues: vec![],
    };
    assert_eq!(
        to_board_load(&scope(BoardBackend::Linear), &empty),
        BoardLoad::Empty
    );
    let full = BoardSnapshot {
        states: vec![state("a", "Todo", 1.0)],
        issues: vec![issue("1", "a")],
    };
    let BoardLoad::Ready(data) = to_board_load(&scope(BoardBackend::Ado), &full) else {
        panic!("ready")
    };
    assert!(data.read_only, "ADO boards are read-only");
    assert!(matches!(
        error_to_load(&BoardError::Unauthorized("x".into())),
        BoardLoad::Unauthorized(_)
    ));
    assert!(matches!(
        error_to_load(&BoardError::Unavailable("x".into())),
        BoardLoad::Unavailable(_)
    ));
    assert!(matches!(
        error_to_load(&BoardError::Provider("x".into())),
        BoardLoad::Error(_)
    ));
}

#[tokio::test]
async fn board_glue_ado_move_touches_neither_provider_nor_audit() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("audit.jsonl");
    let audit = AuditLog::open(&path).unwrap();
    let fake = Fake::default();
    let ado = scope(BoardBackend::Ado);
    let current = || Ok::<_, BoardUnavailable>(scope(BoardBackend::Ado));
    let text = run_move(&change(ado), &fake, &audit, &current).await;
    assert!(text.starts_with("not moved"), "{text}");
    assert!(fake.set_calls.lock().unwrap().is_empty());
    assert_eq!(
        audit_lines(&path),
        0,
        "no audit intent for a read-only board"
    );
}

#[tokio::test]
async fn board_glue_confirmed_move_writes_intent_then_outcome_and_dispatches_once() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("audit.jsonl");
    let audit = AuditLog::open(&path).unwrap();
    let fake = Fake::default();
    *fake.live_state.lock().unwrap() = "todo".into();
    let current = || Ok::<_, BoardUnavailable>(scope(BoardBackend::Linear));
    let text = run_move(
        &change(scope(BoardBackend::Linear)),
        &fake,
        &audit,
        &current,
    )
    .await;
    assert_eq!(text, "moved");
    assert_eq!(
        *fake.set_calls.lock().unwrap(),
        vec![("1".to_string(), "done".to_string())]
    );
    assert_eq!(audit_lines(&path), 2, "intent and outcome");
}

#[tokio::test]
async fn board_glue_remote_change_is_a_conflict_with_no_write() {
    let dir = tempfile::tempdir().unwrap();
    let audit = AuditLog::open(&dir.path().join("a.jsonl")).unwrap();
    let fake = Fake::default();
    *fake.live_state.lock().unwrap() = "someone-else-moved-it".into();
    let current = || Ok::<_, BoardUnavailable>(scope(BoardBackend::Linear));
    let text = run_move(
        &change(scope(BoardBackend::Linear)),
        &fake,
        &audit,
        &current,
    )
    .await;
    assert!(text.contains("changed remotely"), "{text}");
    assert!(fake.set_calls.lock().unwrap().is_empty());
}

#[tokio::test]
async fn board_glue_world_change_before_write_refuses_without_contacting_the_tracker() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("a.jsonl");
    let audit = AuditLog::open(&path).unwrap();
    let fake = Fake::default();
    *fake.live_state.lock().unwrap() = "todo".into();
    let current = || Err::<BoardScope, _>(BoardUnavailable::UnknownWorld);
    let text = run_move(
        &change(scope(BoardBackend::Linear)),
        &fake,
        &audit,
        &current,
    )
    .await;
    assert!(text.starts_with("not moved"), "{text}");
    assert!(fake.set_calls.lock().unwrap().is_empty());
    assert_eq!(audit_lines(&path), 0);
}

#[tokio::test]
async fn board_glue_uncertain_write_is_reconciled_once_and_never_retried() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("a.jsonl");
    let audit = AuditLog::open(&path).unwrap();
    let fake = Fake::default();
    *fake.live_state.lock().unwrap() = "todo".into();
    *fake.set_result.lock().unwrap() = Some(BoardError::Uncertain("timeout".into()));
    let current = || Ok::<_, BoardUnavailable>(scope(BoardBackend::Linear));
    let text = run_move(
        &change(scope(BoardBackend::Linear)),
        &fake,
        &audit,
        &current,
    )
    .await;
    assert!(text.contains("not applied yet"), "{text}");
    assert_eq!(fake.set_calls.lock().unwrap().len(), 1, "no retry");
    assert_eq!(audit_lines(&path), 3, "intent, uncertain, reconciled");
}

#[cfg(target_os = "linux")]
#[tokio::test]
async fn board_glue_audit_failure_blocks_dispatch() {
    // `/dev/full` accepts opens and fails every write with ENOSPC.
    let audit = AuditLog::open(std::path::Path::new("/dev/full")).expect("/dev/full opens");
    let fake = Fake::default();
    *fake.live_state.lock().unwrap() = "todo".into();
    let current = || Ok::<_, BoardUnavailable>(scope(BoardBackend::Linear));
    let text = run_move(
        &change(scope(BoardBackend::Linear)),
        &fake,
        &audit,
        &current,
    )
    .await;
    assert!(text.contains("audit"), "{text}");
    assert!(
        fake.set_calls.lock().unwrap().is_empty(),
        "an unwritable audit log must prevent dispatch"
    );
}

#[tokio::test]
async fn board_glue_moves_from_or_to_the_other_column_are_never_dispatched() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("a.jsonl");
    let audit = AuditLog::open(&path).unwrap();
    let fake = Fake::default();
    *fake.live_state.lock().unwrap() = "todo".into();
    let current = || Ok::<_, BoardUnavailable>(scope(BoardBackend::Linear));
    for (from, to) in [("", "done"), ("todo", "")] {
        let mut request = change(scope(BoardBackend::Linear));
        request.expected_prior_state = from.into();
        request.requested_state = to.into();
        let text = run_move(&request, &fake, &audit, &current).await;
        assert!(text.contains("Other column"), "{text}");
    }
    assert!(fake.set_calls.lock().unwrap().is_empty());
    assert_eq!(audit_lines(&path), 0);
}

fn config_with(server_scope: &str) -> crate::mcp::McpConfig {
    let mut config = crate::mcp::McpConfig::default();
    config.servers.insert(
        "Linear".into(),
        serde_json::from_value(serde_json::json!({"command": "true"})).unwrap(),
    );
    config
        .server_scopes
        .insert("Linear".into(), server_scope.into());
    config
}

#[test]
fn board_glue_a_project_local_linear_server_is_refused() {
    let linear = scope(BoardBackend::Linear);
    assert!(check_board_server(&linear, &config_with("global")).is_ok());
    for local in ["project", "user"] {
        let Err(BoardLoad::Unavailable(why)) = check_board_server(&linear, &config_with(local))
        else {
            panic!("{local} scope must be refused")
        };
        assert!(why.contains("global config"), "{why}");
    }
    let Err(BoardLoad::Unavailable(why)) =
        check_board_server(&linear, &crate::mcp::McpConfig::default())
    else {
        panic!("missing server must be refused")
    };
    assert!(why.contains("not configured"), "{why}");
}

#[test]
fn board_glue_events_from_an_older_open_are_dropped() {
    let mut app = crate::tui::app::tests::create_test_app();
    app.open_board_overlay(BoardLoad::Loading);
    app.board.generation = 2;
    let tx = app.board_sender();
    tx.send(BoardEvent {
        generation: 1,
        kind: BoardEventKind::Loaded {
            load: BoardLoad::Empty,
            notice: Some("stale".into()),
        },
    })
    .unwrap();
    tx.send(BoardEvent {
        generation: 2,
        kind: BoardEventKind::Loaded {
            load: BoardLoad::Unavailable("current".into()),
            notice: None,
        },
    })
    .unwrap();
    assert!(app.poll_board());
    let shown = app
        .board_overlay
        .as_ref()
        .map(|cell| format!("{:?}", cell.borrow()))
        .unwrap();
    assert!(shown.contains("current"), "{shown}");
    assert!(
        !shown.contains("stale") && !shown.contains("Empty"),
        "{shown}"
    );
}

#[test]
fn board_glue_results_never_reopen_a_closed_board() {
    let mut app = crate::tui::app::tests::create_test_app();
    app.open_board_overlay(BoardLoad::Loading);
    app.board.generation = 1;
    let tx = app.board_sender();
    app.board_overlay = None;
    tx.send(BoardEvent {
        generation: 1,
        kind: BoardEventKind::Loaded {
            load: BoardLoad::Empty,
            notice: None,
        },
    })
    .unwrap();
    assert!(!app.poll_board());
    assert!(app.board_overlay.is_none());
    assert!(app.board.events.is_none(), "queued results are dropped");
}

#[test]
fn board_glue_paste_does_not_reach_the_composer_while_the_board_is_open() {
    let mut app = crate::tui::app::tests::create_test_app();
    app.open_board_overlay(BoardLoad::Loading);
    let before = app.input.clone();
    super::super::input::handle_paste(&mut app, "pasted text".into());
    assert_eq!(
        app.input, before,
        "paste is ignored while the board is open"
    );
}

#[test]
fn board_glue_command_without_a_working_directory_shows_no_board_and_queries_nothing() {
    let mut app = crate::tui::app::tests::create_test_app();
    app.session.working_dir = None;
    let generation = app.board.generation;
    assert!(handle_board_command(&mut app, "/board"));
    assert!(app.board_overlay.is_some());
    assert!(app.board.runtime.is_none());
    assert_eq!(app.board.generation, generation, "no fetch was started");
    let shown = format!("{:?}", app.board_overlay.as_ref().unwrap().borrow());
    assert!(shown.contains("no working directory"), "{shown}");
    assert!(
        !handle_board_command(&mut app, "/boardx"),
        "not our command"
    );
}

#[test]
fn board_glue_confirmed_move_without_a_runtime_never_sticks_on_loading() {
    let mut app = crate::tui::app::tests::create_test_app();
    app.open_board_overlay(BoardLoad::Empty);
    app.pending_board_action = Some(BoardAction::Confirmed(MoveRequest {
        issue_id: "1".into(),
        from_column_id: "a".into(),
        to_column_id: "b".into(),
    }));
    assert!(app.poll_board());
    let shown = format!("{:?}", app.board_overlay.as_ref().unwrap().borrow());
    assert!(shown.contains("not ready"), "{shown}");
    assert!(!shown.contains("Loading"), "{shown}");
}

#[test]
fn board_glue_a_workspace_scoped_linear_server_is_refused() {
    let linear = scope(BoardBackend::Linear);
    let mut config = config_with("global");
    config.servers.get_mut("Linear").unwrap().workspaces = vec!["/work/**".into()];
    let Err(BoardLoad::Unavailable(why)) = check_board_server(&linear, &config) else {
        panic!("workspace-scoped server must be refused")
    };
    assert!(why.contains("workspace-scoped"), "{why}");
}
