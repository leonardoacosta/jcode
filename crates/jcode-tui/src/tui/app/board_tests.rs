//! `/board` glue tests: fake provider and a temp audit file only. No tracker,
//! MCP server or network is contacted.

use super::*;
use crate::board::{
    BoardBackend, BoardIssue as CoreIssue, BoardUnavailable, CacheKey, ScopeOrigin, WorkflowState,
};
use async_trait::async_trait;
use std::sync::Mutex;

fn scope(backend: BoardBackend) -> BoardScope {
    BoardScope {
        origin: ScopeOrigin::Configured { world: "w".into() },
        world: "w".into(),
        backend,
        team: "team-1".into(),
        project: None,
        server: "Linear".into(),
        canonical_root: Some(PathBuf::from("/r")),
    }
}

fn state(id: &str, name: &str, position: f64) -> WorkflowState {
    WorkflowState {
        id: id.into(),
        name: name.into(),
        position,
        kind: String::new(),
    }
}

fn issue(id: &str, state_id: &str) -> CoreIssue {
    CoreIssue {
        id: id.into(),
        identifier: format!("T-{id}"),
        title: format!("title {id}"),
        state_id: state_id.into(),
        team_name: None,
    }
}

fn named(identifier: &str, team_name: Option<&str>) -> CoreIssue {
    CoreIssue {
        identifier: identifier.into(),
        team_name: team_name.map(Into::into),
        ..issue(identifier, "a")
    }
}

fn label_of(scope: &BoardScope, issues: Vec<CoreIssue>) -> String {
    let snapshot = BoardSnapshot {
        states: vec![state("a", "Todo", 1.0)],
        issues,
    };
    scope_label(scope, &snapshot)
}

#[test]
fn board_glue_label_shows_team_name_and_shared_key() {
    let linear = scope(BoardBackend::Linear);
    let label = label_of(
        &linear,
        vec![
            named("PRICE-16", Some("Priceless")),
            named("PRICE-10", Some("Priceless")),
        ],
    );
    assert_eq!(label, "Linear · Priceless (PRICE) · team-wide");
    assert!(!label.contains("team-1"), "no team id once a name is known");

    let mut project = linear.clone();
    project.project = Some("proj-1".into());
    assert_eq!(
        label_of(&project, vec![named("PRICE-1", Some("Priceless"))]),
        "Linear · Priceless (PRICE) · project proj-1"
    );
}

#[test]
fn board_glue_label_drops_a_key_the_issues_do_not_share() {
    let label = label_of(
        &scope(BoardBackend::Linear),
        vec![
            named("PRICE-1", Some("Priceless")),
            named("OLD-2", Some("Priceless")),
        ],
    );
    assert_eq!(label, "Linear · Priceless · team-wide");
}

#[test]
fn board_glue_label_falls_back_to_the_team_id_without_a_name() {
    let linear = scope(BoardBackend::Linear);
    assert_eq!(
        label_of(&linear, vec![named("PRICE-1", None)]),
        linear.scope_label()
    );
    assert_eq!(label_of(&linear, vec![]), linear.scope_label());
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

/// Disposable JCODE_HOME holding a global config and global MCP servers,
/// serialized on the shared test-env lock.
struct Env {
    _guard: std::sync::MutexGuard<'static, ()>,
    _home: tempfile::TempDir,
    prev: Vec<(&'static str, Option<std::ffi::OsString>)>,
}

impl Env {
    fn new(config_toml: &str, mcp_json: &str) -> Self {
        let guard = crate::storage::lock_test_env();
        let home = tempfile::TempDir::new().unwrap();
        std::fs::write(home.path().join("config.toml"), config_toml).unwrap();
        std::fs::write(home.path().join("mcp.json"), mcp_json).unwrap();
        let mut prev = Vec::new();
        for (key, value) in [
            ("JCODE_HOME", Some(home.path().as_os_str())),
            ("JCODE_DISABLE_CLAUDE_MCP", Some(std::ffi::OsStr::new("1"))),
        ] {
            prev.push((key, std::env::var_os(key)));
            match value {
                Some(value) => crate::env::set_var(key, value),
                None => crate::env::remove_var(key),
            }
        }
        crate::config::Config::invalidate_cache();
        Self {
            _guard: guard,
            _home: home,
            prev,
        }
    }
}

impl Drop for Env {
    fn drop(&mut self) {
        for (key, value) in self.prev.drain(..).rev() {
            match value {
                Some(value) => crate::env::set_var(key, value),
                None => crate::env::remove_var(key),
            }
        }
        crate::config::Config::invalidate_cache();
    }
}

const RULES: &str = r#"
[[workspaces.rules]]
name = "personal"
origin = "github.com/me/**"
tracker = "linear"
[workspaces.rules.board]
backend = "linear"
team = "team-personal"

[[workspaces.rules]]
name = "priceless"
origin = "github.com/priceless/**"
tracker = "linear"
[workspaces.rules.board]
backend = "linear"
team = "team-priceless"
server = "LinearAlt"

[[workspaces.rules]]
name = "brown"
origin = "dev.azure.com/brown/**"
tracker = "ado"
[workspaces.rules.board]
backend = "ado"
team = "brown-team"
"#;

const SERVERS: &str = r#"{"mcpServers": {
  "Linear": {"url": "https://mcp.example.test/personal"},
  "LinearAlt": {"url": "https://mcp.example.test/priceless"}
}}"#;

fn target_scope(target: &str) -> BoardScope {
    let Ok(BoardRuntime::One(board)) = prepare_runtime(None, Some(target.into())) else {
        panic!("{target} should open one board")
    };
    board.scope
}

#[test]
fn board_command_parses_targets_login_and_logout() {
    use BoardCommand::*;
    assert_eq!(parse_board_command(""), Open(None));
    assert_eq!(
        parse_board_command(" personal/t1 "),
        Open(Some("personal/t1"))
    );
    assert_eq!(
        parse_board_command(" login"),
        Account(AccountAction::Login, None)
    );
    assert_eq!(
        parse_board_command(" login Linear"),
        Account(AccountAction::Login, Some("Linear"))
    );
    assert_eq!(
        parse_board_command(" logout priceless"),
        Account(AccountAction::Logout, Some("priceless"))
    );
    assert_eq!(parse_board_command(" a b"), Usage);
    assert_eq!(parse_board_command(" login a b"), Usage);
}

#[test]
fn board_glue_a_named_world_opens_without_a_cwd_and_writable_only_when_configured() {
    let _env = Env::new(RULES, SERVERS);
    let bound = target_scope("personal");
    assert!(!bound.read_only(), "the configured board is writable");
    assert_eq!(bound.canonical_root, None);
    assert_eq!(bound.server, "Linear");
    assert!(!target_scope("personal/team-personal").read_only());
    let other = target_scope("personal/some-other-team");
    assert!(other.read_only() && other.is_discovered(), "{other:?}");
    assert_eq!(other.server, "Linear");
    // Another world's account is used for that world.
    assert_eq!(target_scope("priceless").server, "LinearAlt");
    let Err(BoardLoad::Unavailable(why)) = prepare_runtime(None, Some("nope".into())) else {
        panic!("unknown world must be refused")
    };
    assert!(why.contains("nope"), "{why}");
}

#[test]
fn board_glue_a_named_board_is_reverified_by_name_before_a_write() {
    let _env = Env::new(RULES, SERVERS);
    let bound = target_scope("personal");
    // Same scope: allowed. Cwd is ignored for a scope without a root.
    assert_eq!(
        current_scope(Some(Path::new("/nowhere")), &bound),
        Ok(bound.clone())
    );
    assert_eq!(current_scope(None, &bound), Ok(bound.clone()));
    // A discovered scope never equals a configured one, so no write passes.
    let mut moved = bound.clone();
    moved.team = "team-other".into();
    assert_ne!(current_scope(None, &moved), Ok(bound));
    // A rooted scope cannot be verified without a directory.
    let mut rooted = target_scope("personal");
    rooted.canonical_root = Some(PathBuf::from("/r"));
    assert_eq!(
        current_scope(None, &rooted),
        Err(BoardUnavailable::UnknownWorld)
    );
}

#[test]
fn board_glue_no_world_builds_the_all_accounts_runtime_and_other_refusals_stay() {
    let _env = Env::new(RULES, SERVERS);
    // No cwd.
    let Ok(BoardRuntime::All(all)) = prepare_runtime(None, None) else {
        panic!("no cwd opens the all-accounts view")
    };
    let servers: Vec<&str> = all.accounts.iter().map(|a| a.server.as_str()).collect();
    assert_eq!(
        servers,
        ["Linear", "LinearAlt"],
        "ADO world is not an account"
    );
    assert!(all.accounts.iter().all(|a| a.gate.is_ok()));
    // A directory in no world, same.
    let dir = tempfile::TempDir::new().unwrap();
    assert!(matches!(
        prepare_runtime(Some(dir.path().into()), None),
        Ok(BoardRuntime::All(_))
    ));
}

#[test]
fn board_glue_the_all_accounts_view_is_not_opened_when_no_account_exists() {
    let _env = Env::new("", "{}");
    let Err(BoardLoad::Unavailable(why)) = prepare_runtime(None, None) else {
        panic!("no accounts, no board")
    };
    assert!(why.contains("no board account"), "{why}");
}

/// Answers `list_teams` per server and `list_issues`/`list_issue_statuses`
/// per team, or fails a whole server.
struct FakeLinear {
    teams: Vec<(&'static str, Result<&'static str, &'static str>)>,
}

const STATUSES: &str = r#"[{"id":"s-back","type":"backlog","name":"Backlog"},{"id":"s-todo","type":"unstarted","name":"Todo"},{"id":"s-done","type":"completed","name":"Done"},{"id":"s-dup","type":"duplicate","name":"Duplicate"}]"#;

fn issues_json(team_id: &str, key: &str, rows: &[(&str, &str)]) -> String {
    let items: Vec<String> = rows
        .iter()
        .map(|(n, status)| {
            format!(
                r#"{{"id":"{key}-{n}","title":"t{n}","status":"{status}","teamId":"{team_id}","team":"{key} team"}}"#
            )
        })
        .collect();
    format!(r#"{{"issues":[{}],"hasNextPage":false}}"#, items.join(","))
}

#[async_trait]
impl crate::board::McpCaller for FakeLinear {
    async fn call(
        &self,
        server: &str,
        tool: &str,
        args: serde_json::Value,
    ) -> anyhow::Result<String> {
        let Some((_, teams)) = self.teams.iter().find(|(name, _)| *name == server) else {
            anyhow::bail!("unknown server {server}")
        };
        let teams = teams.map_err(|why| anyhow::anyhow!("{why}"))?;
        match tool {
            "list_teams" => Ok(teams.to_string()),
            "list_issue_statuses" => Ok(STATUSES.to_string()),
            "list_issues" => {
                let team = args["team"].as_str().unwrap();
                Ok(match team {
                    "team-personal" => issues_json(team, "PER", &[("1", "Todo"), ("2", "Done")]),
                    "team-extra" => {
                        issues_json(team, "EXT", &[("1", "Backlog"), ("2", "Duplicate")])
                    }
                    _ => issues_json(team, "ALT", &[("7", "Todo")]),
                })
            }
            other => anyhow::bail!("unexpected tool {other}"),
        }
    }
}

fn account(server: &str, bindings: Vec<BoardScope>) -> AllAccount {
    AllAccount {
        server: server.into(),
        auth: format!("auth:{server}"),
        gate: Ok(()),
        bindings,
    }
}

fn all_boards_with(
    accounts: Vec<AllAccount>,
    teams: Vec<(&'static str, Result<&'static str, &'static str>)>,
) -> AllBoards {
    AllBoards {
        accounts,
        provider: Arc::new(LinearMcpProvider::new(Arc::new(FakeLinear { teams }))),
        cache: BoardCache::new(CACHE_TTL),
    }
}

fn cards(data: &BoardData, kind: &str) -> Vec<String> {
    data.columns
        .iter()
        .filter(|c| c.kind == kind)
        .flat_map(|c| {
            c.issues
                .iter()
                .map(|i| format!("{} {}", i.identifier, i.title))
        })
        .collect()
}

#[tokio::test]
async fn board_glue_all_accounts_groups_by_state_kind_with_account_and_team_labels() {
    let mut bound = scope(BoardBackend::Linear);
    bound.team = "team-personal".into();
    bound.server = "Linear".into();
    let all = all_boards_with(
        vec![
            account("Linear", vec![bound]),
            account("LinearAlt", Vec::new()),
        ],
        vec![
            (
                "Linear",
                Ok(
                    r#"[{"id":"team-personal","key":"PER","name":"Personal"},{"id":"team-extra","key":"EXT","name":"Extra"}]"#,
                ),
            ),
            (
                "LinearAlt",
                Ok(r#"[{"id":"team-alt","key":"ALT","name":"Alt"}]"#),
            ),
        ],
    );
    let (load, notice) = load_all(&all, false).await;
    assert_eq!(notice, None);
    let BoardLoad::Ready(data) = load else {
        panic!("expected a board, got {load:?}")
    };
    assert_eq!(data.scope_label, "All accounts · read-only");
    assert!(data.read_only);
    let kinds: Vec<&str> = data.columns.iter().map(|c| c.kind.as_str()).collect();
    assert_eq!(
        kinds,
        [
            "backlog",
            "unstarted",
            "started",
            "completed",
            "canceled",
            ""
        ],
        "fixed order, unknown kinds in a trailing Other"
    );
    assert_eq!(cards(&data, "backlog"), ["EXT-1 [Linear] t1"]);
    assert_eq!(
        cards(&data, "unstarted"),
        ["PER-1 [Linear] t1", "ALT-7 [LinearAlt] t7"]
    );
    assert_eq!(cards(&data, "completed"), ["PER-2 [Linear] t2"]);
    assert_eq!(data.columns.last().unwrap().name, "Other");
    assert_eq!(cards(&data, ""), ["EXT-2 [Linear] t2"]);
}

#[tokio::test]
async fn board_glue_all_accounts_omits_the_account_label_for_a_single_account() {
    let all = all_boards_with(
        vec![account("Linear", Vec::new())],
        vec![(
            "Linear",
            Ok(r#"[{"id":"team-extra","key":"EXT","name":"Extra"}]"#),
        )],
    );
    let (BoardLoad::Ready(data), None) = load_all(&all, false).await else {
        panic!("expected a board")
    };
    assert_eq!(cards(&data, "backlog"), ["EXT-1 t1"]);
}

#[tokio::test]
async fn board_glue_all_accounts_uses_the_configured_scope_once_and_discovered_for_the_rest() {
    // The seen scopes tell which origin each team was read with.
    let mut bound = scope(BoardBackend::Linear);
    bound.team = "team-personal".into();
    let all = all_boards_with(
        vec![account("Linear", vec![bound])],
        vec![(
            "Linear",
            Ok(
                r#"[{"id":"team-personal","key":"PER","name":"P"},{"id":"team-extra","key":"EXT","name":"E"}]"#,
            ),
        )],
    );
    let _ = load_all(&all, false).await;
    let configured = CacheKey::new(
        &BoardScope {
            team: "team-personal".into(),
            ..scope(BoardBackend::Linear)
        },
        "auth:Linear",
    );
    assert!(
        all.cache.get(&configured).is_some(),
        "bound team: configured"
    );
    let discovered = CacheKey::new(
        &BoardScope {
            origin: ScopeOrigin::Discovered,
            world: String::new(),
            team: "team-extra".into(),
            canonical_root: None,
            ..scope(BoardBackend::Linear)
        },
        "auth:Linear",
    );
    assert!(
        all.cache.get(&discovered).is_some(),
        "other team: discovered"
    );
}

#[tokio::test]
async fn board_glue_all_accounts_a_failing_account_gets_a_login_notice_and_hides_no_one() {
    let all = all_boards_with(
        vec![
            account("Linear", Vec::new()),
            account("LinearAlt", Vec::new()),
        ],
        vec![
            ("Linear", Err("HTTP 401 unauthorized")),
            (
                "LinearAlt",
                Ok(r#"[{"id":"team-alt","key":"ALT","name":"Alt"}]"#),
            ),
        ],
    );
    let (load, notice) = load_all(&all, false).await;
    let BoardLoad::Ready(data) = load else {
        panic!("the other account still shows: {load:?}")
    };
    assert_eq!(cards(&data, "unstarted"), ["ALT-7 [LinearAlt] t7"]);
    assert_eq!(notice.as_deref(), Some("Linear: run /board login Linear"));
}

#[tokio::test]
async fn board_glue_all_accounts_unparseable_teams_are_a_notice_not_a_panic() {
    let all = all_boards_with(
        vec![account("Linear", Vec::new())],
        vec![("Linear", Ok("this is not json"))],
    );
    let (load, notice) = load_all(&all, false).await;
    assert!(
        matches!(&load, BoardLoad::Error(text) if text.contains("Linear:")),
        "{load:?}"
    );
    assert_eq!(notice, None);
    // Every account unauthorized is the login state.
    let all = all_boards_with(
        vec![account("Linear", Vec::new())],
        vec![("Linear", Err("401"))],
    );
    let (load, _) = load_all(&all, false).await;
    assert!(
        matches!(&load, BoardLoad::Unauthorized(text) if text.contains("run /board login Linear")),
        "{load:?}"
    );
}

#[tokio::test]
async fn board_glue_all_accounts_a_gated_account_is_a_notice() {
    let mut gated = account("Linear", Vec::new());
    gated.gate = Err("MCP server 'Linear' is not configured".into());
    let all = all_boards_with(
        vec![gated, account("LinearAlt", Vec::new())],
        vec![(
            "LinearAlt",
            Ok(r#"[{"id":"team-alt","key":"ALT","name":"Alt"}]"#),
        )],
    );
    let (load, notice) = load_all(&all, false).await;
    assert!(matches!(load, BoardLoad::Ready(_)));
    assert!(
        notice
            .unwrap()
            .starts_with("Linear: MCP server 'Linear' is not configured")
    );
}

fn messages_since(app: &crate::tui::app::App, before: usize) -> String {
    app.display_messages()[before..]
        .iter()
        .map(|m| m.content.clone())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn board_glue_login_and_logout_refuse_what_they_cannot_resolve() {
    let _env = Env::new(RULES, SERVERS);
    // ADO world has no /board login.
    let mut app = crate::tui::app::tests::create_test_app();
    let before = app.display_messages().len();
    assert!(handle_board_command(&mut app, "/board login brown"));
    let text = messages_since(&app, before);
    assert!(text.contains("no /board login"), "{text}");
    // Unknown name.
    let before = app.display_messages().len();
    assert!(handle_board_command(&mut app, "/board login nope"));
    let text = messages_since(&app, before);
    assert!(
        text.contains("neither a board account nor a configured world"),
        "{text}"
    );
    assert!(app.board_overlay.is_none());
    // Too many words.
    let before = app.display_messages().len();
    assert!(handle_board_command(&mut app, "/board nope nope"));
    assert!(messages_since(&app, before).contains("/board login"));
}

#[test]
fn board_glue_logout_outside_a_world_lists_accounts_and_needs_a_name() {
    let _env = Env::new(RULES, SERVERS);
    let mut app = crate::tui::app::tests::create_test_app();
    app.session.working_dir = None;
    let before = app.display_messages().len();
    assert!(handle_board_command(&mut app, "/board logout"));
    let text = messages_since(&app, before);
    assert!(text.contains("Name the account"), "{text}");
    assert!(text.contains("Linear, LinearAlt"), "{text}");
    assert!(
        !text.contains("Removed local OAuth") && !text.contains("No local OAuth"),
        "{text}"
    );
}

#[test]
fn board_glue_login_with_no_cwd_starts_a_login_for_each_tokenless_account() {
    let _env = Env::new(RULES, SERVERS);
    let mut app = crate::tui::app::tests::create_test_app();
    app.session.working_dir = None;
    let before = app.display_messages().len();
    assert!(handle_board_command(&mut app, "/board login"));
    // Sync test: no async runtime, so each start reports it instead of
    // opening a browser. One report per account proves one start each.
    let text = messages_since(&app, before);
    // Identical consecutive messages merge into one with a count.
    assert_eq!(text, "No async runtime for MCP login. [×2]", "{text}");
    // A world name resolves to its account and says which.
    let before = app.display_messages().len();
    assert!(handle_board_command(&mut app, "/board login priceless"));
    let text = messages_since(&app, before);
    assert!(
        text.contains("world 'priceless': MCP server 'LinearAlt'"),
        "{text}"
    );
}

#[test]
fn board_glue_command_without_a_working_directory_opens_the_all_accounts_view() {
    let _env = Env::new(RULES, SERVERS);
    let mut app = crate::tui::app::tests::create_test_app();
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let _enter = runtime.enter();
    app.session.working_dir = None;
    let generation = app.board.generation;
    assert!(handle_board_command(&mut app, "/board"));
    assert!(app.board_overlay.is_some());
    assert!(
        app.board.generation > generation,
        "a fetch started: no cwd is the all-accounts view"
    );
    assert!(
        !handle_board_command(&mut app, "/boardx"),
        "not our command"
    );
}

#[test]
fn board_glue_unauthorized_leads_with_the_login_hint() {
    let BoardLoad::Unauthorized(text) = error_to_load(&BoardError::Unauthorized("HTTP 401".into()))
    else {
        panic!("unauthorized maps to Unauthorized")
    };
    assert!(text.starts_with("run /board login"), "{text}");
    assert!(text.contains("HTTP 401"), "{text}");
}

#[test]
fn board_glue_label_names_a_non_default_account() {
    let mut s = scope(BoardBackend::Linear);
    assert!(!label_of(&s, vec![named("PRICE-1", Some("Priceless"))]).contains("via"));
    s.server = "linear-priceless".into();
    assert_eq!(
        label_of(&s, vec![named("PRICE-1", Some("Priceless"))]),
        "Linear · Priceless (PRICE) · team-wide · via linear-priceless"
    );
    assert!(label_of(&s, vec![named("PRICE-1", None)]).ends_with(" · via linear-priceless"));
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

async fn verdict_for(live: &str, set_result: Option<BoardError>) -> (MoveVerdict, String) {
    let dir = tempfile::tempdir().unwrap();
    let audit = AuditLog::open(&dir.path().join("a.jsonl")).unwrap();
    let fake = Fake::default();
    *fake.live_state.lock().unwrap() = live.into();
    *fake.set_result.lock().unwrap() = set_result;
    let current = || Ok::<_, BoardUnavailable>(scope(BoardBackend::Linear));
    run_move_verdict(
        &change(scope(BoardBackend::Linear)),
        &fake,
        &audit,
        &current,
    )
    .await
}

#[tokio::test]
async fn board_glue_verdict_moved_only_when_applied() {
    assert_eq!(verdict_for("todo", None).await.0, MoveVerdict::Moved);
}

#[tokio::test]
async fn board_glue_verdict_not_moved_when_nothing_was_written() {
    // Remote conflict, definitive provider failure.
    assert_eq!(
        verdict_for("someone-else", None).await.0,
        MoveVerdict::NotMoved
    );
    let failed = verdict_for("todo", Some(BoardError::Provider("denied".into()))).await;
    assert_eq!(failed.0, MoveVerdict::NotMoved, "{failed:?}");
}

#[tokio::test]
async fn board_glue_verdict_unknown_when_the_write_may_have_landed() {
    // The fake leaves the live state at "todo" after an uncertain error, so the
    // reconcile read says "not applied yet": still unknown, never "not moved".
    let (verdict, text) = verdict_for("todo", Some(BoardError::Uncertain("timeout".into()))).await;
    assert_eq!(verdict, MoveVerdict::Unknown, "{text}");
}

#[test]
fn board_glue_move_done_event_reaches_the_overlay_and_rolls_back_a_failure() {
    let mut app = crate::tui::app::tests::create_test_app();
    let data = BoardData {
        scope_label: "s".into(),
        read_only: false,
        columns: vec![
            BoardColumn {
                id: "a".into(),
                name: "A".into(),
                kind: "unstarted".into(),
                issues: vec![BoardIssue {
                    id: "1".into(),
                    identifier: "T-1".into(),
                    title: "t".into(),
                }],
            },
            BoardColumn {
                id: "b".into(),
                name: "B".into(),
                kind: "started".into(),
                issues: vec![],
            },
        ],
    };
    app.open_board_overlay(BoardLoad::Ready(data));
    app.board.generation = 1;
    let cell = app.board_overlay.as_ref().unwrap();
    for code in [
        crossterm::event::KeyCode::Char('m'),
        crossterm::event::KeyCode::Enter,
        crossterm::event::KeyCode::Char('y'),
    ] {
        cell.borrow_mut()
            .handle_key(code, crossterm::event::KeyModifiers::NONE);
    }
    let tx = app.board_sender();
    tx.send(BoardEvent {
        generation: 1,
        kind: BoardEventKind::MoveDone {
            verdict: MoveVerdict::NotMoved,
            text: "not moved: audit log unavailable".into(),
            fresh: None,
        },
    })
    .unwrap();
    assert!(app.poll_board());
    let shown = format!("{:?}", app.board_overlay.as_ref().unwrap().borrow());
    assert!(
        shown.contains("not moved: audit log unavailable"),
        "{shown}"
    );
}
