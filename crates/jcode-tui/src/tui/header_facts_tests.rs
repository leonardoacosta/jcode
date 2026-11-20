use super::*;

const LIST: &str = r#"{"changes":[
 {"name":"a","completedTasks":3,"totalTasks":5,"status":"in-progress"},
 {"name":"b","completedTasks":0,"totalTasks":4,"status":"in-progress"},
 {"name":"c","completedTasks":0,"totalTasks":2,"status":"in-progress"},
 {"name":"d","completedTasks":0,"totalTasks":0,"status":"in-progress"},
 {"name":"e","completedTasks":5,"totalTasks":5,"status":"complete"}]}"#;
const VALIDATE: &str = r#"{"items":[
 {"id":"a","type":"change","valid":true},
 {"id":"b","type":"change","valid":true},
 {"id":"c","type":"change","valid":false},
 {"id":"d","type":"change","valid":true},
 {"id":"e","type":"change","valid":true}]}"#;

#[test]
fn openspec_buckets_active_ready_and_not_ready() {
    let changes = parse_openspec(LIST, VALIDATE).unwrap();
    // a active; b ready; c invalid and d without tasks are not ready; e complete.
    assert_eq!(
        openspec_text(&changes),
        "openspec: 2 not ready | 1 ready | 1 active"
    );
}

#[test]
fn openspec_change_missing_from_validate_is_not_ready() {
    let changes = parse_openspec(LIST, r#"{"items":[]}"#).unwrap();
    assert_eq!(
        openspec_text(&changes),
        "openspec: 3 not ready | 0 ready | 1 active"
    );
}

#[test]
fn openspec_unreadable_output_hides_the_row() {
    assert!(parse_openspec("not json", VALIDATE).is_none());
    assert!(parse_openspec(r#"{"changes":[{"name":"x"}]}"#, VALIDATE).is_none());
}

#[test]
fn openspec_row_absent_without_openspec_dir_and_spawns_nothing() {
    let dir = tempfile::TempDir::new().unwrap();
    assert_eq!(openspec_row(dir.path()), None);
    assert!(
        lock(&OPENSPEC).in_flight.is_empty()
            || !lock(&OPENSPEC)
                .in_flight
                .iter()
                .any(|(p, _)| p == dir.path())
    );
}

#[test]
fn linear_text_lists_open_states_in_board_order() {
    use crate::board::{BoardBackend, BoardIssue, BoardScope, BoardSnapshot, WorkflowState};
    let state = |id: &str, name: &str, kind: &str| WorkflowState {
        id: id.into(),
        name: name.into(),
        position: 0.0,
        kind: kind.into(),
    };
    let issue = |state: &str| BoardIssue {
        id: state.into(),
        identifier: "JC-1".into(),
        title: String::new(),
        state_id: state.into(),
        team_name: Some("Jcode".into()),
    };
    let scope = BoardScope {
        world: "personal".into(),
        backend: BoardBackend::Linear,
        team: "team-id".into(),
        project: None,
        server: "linear".into(),
        canonical_root: PathBuf::new(),
    };
    let snapshot = BoardSnapshot {
        states: vec![
            state("b", "Backlog", "backlog"),
            state("t", "Todo", "unstarted"),
            state("p", "In Progress", "started"),
            state("d", "Done", "completed"),
            state("c", "Canceled", "canceled"),
        ],
        issues: vec![issue("b"), issue("b"), issue("p"), issue("d"), issue("c")],
    };
    assert_eq!(
        linear_text(&scope, &snapshot),
        "linear: Jcode · Backlog 2 · In Progress 1"
    );
}
