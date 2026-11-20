use super::*;
use crate::board::test_scope as base_scope;
use jcode_config_types::BoardBackend;
use std::collections::VecDeque;
use std::sync::Mutex;

type Reply = anyhow::Result<String>;

/// Records every call and replays queued replies per tool.
#[derive(Default)]
struct Fake {
    calls: Mutex<Vec<(String, String, Value)>>,
    statuses: Mutex<VecDeque<Reply>>,
    issues: Mutex<VecDeque<Reply>>,
    save: Mutex<VecDeque<Reply>>,
}

impl Fake {
    fn calls(&self) -> Vec<(String, String, Value)> {
        self.calls.lock().unwrap().clone()
    }
    fn tools(&self) -> Vec<String> {
        self.calls().into_iter().map(|c| c.1).collect()
    }
}

#[async_trait]
impl McpCaller for Fake {
    async fn call(&self, server: &str, tool: &str, args: Value) -> anyhow::Result<String> {
        self.calls
            .lock()
            .unwrap()
            .push((server.into(), tool.into(), args));
        let queue = match tool {
            "list_issue_statuses" => &self.statuses,
            "list_issues" => &self.issues,
            "save_issue" => &self.save,
            other => panic!("tool outside the allowlist: {other}"),
        };
        queue
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or_else(|| Err(anyhow::anyhow!("no reply queued for {tool}")))
    }
}

fn provider(fake: &Arc<Fake>) -> LinearMcpProvider {
    LinearMcpProvider::new(fake.clone())
}

fn scope() -> BoardScope {
    base_scope(BoardBackend::Linear)
}

fn project_scope() -> BoardScope {
    BoardScope {
        project: Some("proj-1".into()),
        ..scope()
    }
}

// assumed shape: bare array of statuses with `position`.
const STATUSES_ARRAY: &str = r#"[
  {"id":"s-done","name":"Done","type":"completed","position":3},
  {"id":"s-todo","name":"Todo","type":"unstarted","position":1},
  {"id":"s-prog","name":"In Progress","type":"started","position":2}
]"#;

// assumed shape: object with `statuses`, no `position`, only `type`.
const STATUSES_TYPED: &str = r#"{"statuses":[
  {"id":"b","name":"Done","type":"completed"},
  {"id":"a","name":"Backlog","type":"backlog"},
  {"id":"c","name":"Started","type":"started"}
]}"#;

// assumed shape: object with `nodes`, no position and no type (provider order).
const STATUSES_NODES: &str = r#"{"nodes":[{"id":"x","name":"X"},{"id":"y","name":"Y"}]}"#;

// assumed shape: `issues` with nested state/team/project and pageInfo.
fn issues_page(items: &str, next: Option<&str>) -> String {
    let page = match next {
        Some(c) => format!(r#","pageInfo":{{"hasNextPage":true,"endCursor":"{c}"}}"#),
        None => r#","pageInfo":{"hasNextPage":false}"#.to_string(),
    };
    format!(r#"{{"issues":[{items}]{page}}}"#)
}

const ISSUE_A: &str = r#"{"id":"i-a","identifier":"PRICE-1","title":"A",
  "state":{"id":"s-todo","name":"Todo"},"team":{"id":"team-1"},"project":{"id":"proj-1"}}"#;
// assumed shape: flat teamId/projectId and a `status` string name.
const ISSUE_B_FLAT: &str = r#"{"id":"i-b","identifier":"PRICE-2","title":"B",
  "status":"In Progress","teamId":"team-1","projectId":"proj-1"}"#;
const ISSUE_OTHER_TEAM: &str = r#"{"id":"i-brown","identifier":"BROWN-9","title":"leak",
  "state":{"id":"s-todo","name":"Todo"},"team":{"id":"team-brown"}}"#;
const ISSUE_NO_TEAM: &str = r#"{"id":"i-noteam","identifier":"X-1","title":"t",
  "state":{"id":"s-todo","name":"Todo"}}"#;
const ISSUE_OTHER_PROJECT: &str = r#"{"id":"i-p2","identifier":"PRICE-3","title":"t",
  "state":{"id":"s-todo","name":"Todo"},"team":{"id":"team-1"},"project":{"id":"proj-2"}}"#;
const ISSUE_NO_PROJECT: &str = r#"{"id":"i-np","identifier":"PRICE-4","title":"t",
  "state":{"id":"s-todo","name":"Todo"},"team":{"id":"team-1"}}"#;

#[tokio::test]
async fn states_sorted_by_position() {
    let fake = Arc::new(Fake::default());
    fake.statuses
        .lock()
        .unwrap()
        .push_back(Ok(STATUSES_ARRAY.into()));
    let states = provider(&fake).workflow_states(&scope()).await.unwrap();
    let ids: Vec<_> = states.iter().map(|s| s.id.as_str()).collect();
    assert_eq!(ids, ["s-todo", "s-prog", "s-done"]);
    assert_eq!(
        fake.calls(),
        vec![(
            "Linear".to_string(),
            "list_issue_statuses".to_string(),
            json!({"team":"team-1"})
        )]
    );
}

#[tokio::test]
async fn states_fall_back_to_type_then_provider_order() {
    let fake = Arc::new(Fake::default());
    fake.statuses
        .lock()
        .unwrap()
        .push_back(Ok(STATUSES_TYPED.into()));
    fake.statuses
        .lock()
        .unwrap()
        .push_back(Ok(STATUSES_NODES.into()));
    let p = provider(&fake);
    let names: Vec<_> = p
        .workflow_states(&scope())
        .await
        .unwrap()
        .into_iter()
        .map(|s| s.name)
        .collect();
    assert_eq!(names, ["Backlog", "Started", "Done"]);
    let names: Vec<_> = p
        .workflow_states(&scope())
        .await
        .unwrap()
        .into_iter()
        .map(|s| s.name)
        .collect();
    assert_eq!(names, ["X", "Y"]);
}

#[tokio::test]
async fn unparseable_output_is_a_provider_error_never_partial() {
    for bad in [
        "not json at all",
        r#"{"unexpected":1}"#,
        r#"[{"id":"a","name":"A"},{"name":"no id"}]"#,
        r#"[{"id":"a","name":"A"}, 5]"#,
    ] {
        let fake = Arc::new(Fake::default());
        fake.statuses.lock().unwrap().push_back(Ok(bad.into()));
        let err = provider(&fake).workflow_states(&scope()).await.unwrap_err();
        assert!(matches!(err, BoardError::Provider(_)), "{bad}: {err:?}");
    }
    // One bad issue poisons the whole page.
    let fake = Arc::new(Fake::default());
    fake.issues.lock().unwrap().push_back(Ok(issues_page(
        &format!("{ISSUE_A},{{\"title\":\"no id\"}}"),
        None,
    )));
    assert!(matches!(
        provider(&fake).issues(&scope()).await,
        Err(BoardError::Provider(_))
    ));
}

#[tokio::test]
async fn issues_are_scoped_client_side_and_args_carry_binding() {
    let fake = Arc::new(Fake::default());
    let items = [
        ISSUE_A,
        ISSUE_OTHER_TEAM,
        ISSUE_OTHER_PROJECT,
        ISSUE_NO_PROJECT,
    ]
    .join(",");
    fake.issues
        .lock()
        .unwrap()
        .push_back(Ok(issues_page(&items, None)));
    let got = provider(&fake).issues(&scope()).await.unwrap();
    // Team-wide binding: only issues of a different (present) team are dropped.
    let ids: Vec<_> = got.iter().map(|i| i.id.as_str()).collect();
    assert_eq!(ids, ["i-a", "i-p2", "i-np"]);
    let (server, tool, args) = &fake.calls()[0];
    assert_eq!((server.as_str(), tool.as_str()), ("Linear", "list_issues"));
    assert_eq!(args, &json!({"team":"team-1","limit":100}));
}

#[tokio::test]
async fn project_binding_drops_other_projects() {
    let fake = Arc::new(Fake::default());
    let items = [ISSUE_A, ISSUE_OTHER_PROJECT, ISSUE_OTHER_TEAM].join(",");
    fake.issues
        .lock()
        .unwrap()
        .push_back(Ok(issues_page(&items, None)));
    let got = provider(&fake).issues(&project_scope()).await.unwrap();
    assert_eq!(
        got.iter().map(|i| i.id.as_str()).collect::<Vec<_>>(),
        ["i-a"]
    );
    assert_eq!(
        fake.calls()[0].2,
        json!({"team":"team-1","project":"proj-1","limit":100})
    );
}

#[tokio::test]
async fn pagination_follows_cursor_and_caps_pages() {
    let fake = Arc::new(Fake::default());
    fake.issues
        .lock()
        .unwrap()
        .push_back(Ok(issues_page(ISSUE_A, Some("c1"))));
    fake.issues
        .lock()
        .unwrap()
        .push_back(Ok(issues_page(ISSUE_NO_PROJECT, None)));
    let got = provider(&fake).issues(&scope()).await.unwrap();
    assert_eq!(got.len(), 2);
    assert_eq!(fake.calls()[1].2["cursor"], json!("c1"));

    // Endless pages hit the hard cap and error instead of truncating.
    let fake = Arc::new(Fake::default());
    for n in 0..MAX_PAGES + 2 {
        fake.issues
            .lock()
            .unwrap()
            .push_back(Ok(issues_page(ISSUE_A, Some(&format!("c{n}")))));
    }
    let err = provider(&fake).issues(&scope()).await.unwrap_err();
    assert!(matches!(err, BoardError::Provider(m) if m.contains("pages")));
    assert_eq!(fake.calls().len(), MAX_PAGES);

    // A cursor that does not advance is an error, not a loop.
    let fake = Arc::new(Fake::default());
    for _ in 0..3 {
        fake.issues
            .lock()
            .unwrap()
            .push_back(Ok(issues_page(ISSUE_A, Some("same"))));
    }
    assert!(provider(&fake).issues(&scope()).await.is_err());
    assert!(fake.calls().len() <= 2);
}

#[tokio::test]
async fn state_name_is_resolved_through_workflow_states() {
    let fake = Arc::new(Fake::default());
    fake.issues
        .lock()
        .unwrap()
        .push_back(Ok(issues_page(ISSUE_B_FLAT, None)));
    fake.statuses
        .lock()
        .unwrap()
        .push_back(Ok(STATUSES_ARRAY.into()));
    let got = provider(&fake).issues(&scope()).await.unwrap();
    assert_eq!(got[0].state_id, "s-prog");
    assert_eq!(fake.tools(), ["list_issues", "list_issue_statuses"]);

    // A state name the team does not have is an error.
    let fake = Arc::new(Fake::default());
    fake.issues
        .lock()
        .unwrap()
        .push_back(Ok(issues_page(ISSUE_B_FLAT, None)));
    fake.statuses
        .lock()
        .unwrap()
        .push_back(Ok(r#"[{"id":"q","name":"Other"}]"#.into()));
    assert!(matches!(
        provider(&fake).issues(&scope()).await,
        Err(BoardError::Provider(_))
    ));
}

#[tokio::test]
async fn auth_failures_map_to_unauthorized() {
    for text in ["HTTP 401", "Unauthorized", "403 Forbidden", "Not signed in"] {
        let fake = Arc::new(Fake::default());
        fake.statuses
            .lock()
            .unwrap()
            .push_back(Err(anyhow::anyhow!("{text}")));
        let err = provider(&fake).workflow_states(&scope()).await.unwrap_err();
        assert!(
            matches!(err, BoardError::Unauthorized(_)),
            "{text}: {err:?}"
        );
    }
    // Auth text in an "ok" body that is not JSON.
    let fake = Arc::new(Fake::default());
    fake.issues
        .lock()
        .unwrap()
        .push_back(Ok("Unauthorized: please sign in".into()));
    assert!(matches!(
        provider(&fake).issues(&scope()).await,
        Err(BoardError::Unauthorized(_))
    ));
    // Plain transport failure on a read is Unavailable.
    let fake = Arc::new(Fake::default());
    fake.statuses
        .lock()
        .unwrap()
        .push_back(Err(anyhow::anyhow!("connection reset")));
    assert!(matches!(
        provider(&fake).workflow_states(&scope()).await,
        Err(BoardError::Unavailable(_))
    ));
}

#[tokio::test]
async fn save_issue_sends_exactly_id_and_state_after_scope_check() {
    let fake = Arc::new(Fake::default());
    fake.issues
        .lock()
        .unwrap()
        .push_back(Ok(issues_page(ISSUE_A, None)));
    fake.statuses
        .lock()
        .unwrap()
        .push_back(Ok(STATUSES_ARRAY.into()));
    fake.save.lock().unwrap().push_back(Ok("{}".into()));
    provider(&fake)
        .set_issue_state(&scope(), "i-a", "s-prog")
        .await
        .unwrap();
    let calls = fake.calls();
    assert_eq!(
        fake.tools(),
        ["list_issues", "list_issue_statuses", "save_issue"]
    );
    assert_eq!(calls[2].2, json!({"id":"i-a","state":"s-prog"}));
}

#[test]
fn save_issue_arg_keys_are_exactly_id_and_state() {
    let args = save_issue_args("i", "s");
    let keys: Vec<_> = args.as_object().unwrap().keys().cloned().collect();
    assert_eq!(keys, ["id", "state"]);
    for forbidden in [
        "labels",
        "labelIds",
        "assignee",
        "assigneeId",
        "comment",
        "description",
        "title",
        "priority",
        "project",
        "team",
    ] {
        assert!(args.get(forbidden).is_none(), "{forbidden}");
    }
}

#[test]
fn tool_names_are_the_fixed_allowlist() {
    let all = [Tool::ListIssueStatuses, Tool::ListIssues, Tool::SaveIssue];
    let names: Vec<_> = all.iter().map(|t| t.name()).collect();
    assert_eq!(names, ["list_issue_statuses", "list_issues", "save_issue"]);
}

#[tokio::test]
async fn save_refuses_issue_outside_scope_without_writing() {
    let fake = Arc::new(Fake::default());
    fake.issues
        .lock()
        .unwrap()
        .push_back(Ok(issues_page(ISSUE_OTHER_TEAM, None)));
    let err = provider(&fake)
        .set_issue_state(&scope(), "i-brown", "s-prog")
        .await
        .unwrap_err();
    assert!(matches!(err, BoardError::Provider(_)));
    assert_eq!(fake.tools(), ["list_issues"]);
}

#[tokio::test]
async fn write_transport_failure_is_uncertain_and_tool_error_is_provider() {
    let fake = Arc::new(Fake::default());
    fake.issues
        .lock()
        .unwrap()
        .push_back(Ok(issues_page(ISSUE_A, None)));
    fake.statuses
        .lock()
        .unwrap()
        .push_back(Ok(STATUSES_ARRAY.into()));
    fake.save
        .lock()
        .unwrap()
        .push_back(Err(anyhow::anyhow!("timed out")));
    assert!(matches!(
        provider(&fake)
            .set_issue_state(&scope(), "i-a", "s-prog")
            .await,
        Err(BoardError::Uncertain(_))
    ));
    let fake = Arc::new(Fake::default());
    fake.issues
        .lock()
        .unwrap()
        .push_back(Ok(issues_page(ISSUE_A, None)));
    fake.statuses
        .lock()
        .unwrap()
        .push_back(Ok(STATUSES_ARRAY.into()));
    fake.save
        .lock()
        .unwrap()
        .push_back(Err(ToolError("invalid state".into()).into()));
    assert!(matches!(
        provider(&fake)
            .set_issue_state(&scope(), "i-a", "s-prog")
            .await,
        Err(BoardError::Provider(_))
    ));
    let fake = Arc::new(Fake::default());
    fake.issues
        .lock()
        .unwrap()
        .push_back(Ok(issues_page(ISSUE_A, None)));
    fake.statuses
        .lock()
        .unwrap()
        .push_back(Ok(STATUSES_ARRAY.into()));
    fake.save
        .lock()
        .unwrap()
        .push_back(Err(anyhow::anyhow!("401 unauthorized")));
    assert!(matches!(
        provider(&fake)
            .set_issue_state(&scope(), "i-a", "s-prog")
            .await,
        Err(BoardError::Unauthorized(_))
    ));
}

#[tokio::test]
async fn issue_state_reads_live_state_inside_scope_only() {
    let fake = Arc::new(Fake::default());
    fake.issues
        .lock()
        .unwrap()
        .push_back(Ok(issues_page(ISSUE_A, None)));
    assert_eq!(
        provider(&fake).issue_state(&scope(), "i-a").await.unwrap(),
        "s-todo"
    );
    let fake = Arc::new(Fake::default());
    fake.issues
        .lock()
        .unwrap()
        .push_back(Ok(issues_page(ISSUE_A, None)));
    assert!(
        provider(&fake)
            .issue_state(&scope(), "missing")
            .await
            .is_err()
    );
}

#[tokio::test]
async fn issue_without_team_id_fails_the_read_instead_of_being_dropped() {
    let fake = Arc::new(Fake::default());
    let items = [ISSUE_A, ISSUE_NO_TEAM].join(",");
    fake.issues
        .lock()
        .unwrap()
        .push_back(Ok(issues_page(&items, None)));
    let err = provider(&fake).issues(&scope()).await.unwrap_err();
    assert!(
        matches!(&err, BoardError::Provider(m) if m.contains("cannot verify issue team")),
        "{err:?}"
    );
    // Team as an unparsed display string: every issue lacks a team id.
    let fake = Arc::new(Fake::default());
    fake.issues.lock().unwrap().push_back(Ok(issues_page(
        r#"{"id":"i","identifier":"X-1","title":"t","state":{"id":"s","name":"Todo"},"team":"Priceless"}"#,
        None,
    )));
    assert!(matches!(
        provider(&fake).issues(&scope()).await,
        Err(BoardError::Provider(_))
    ));
}

#[tokio::test]
async fn project_binding_fails_closed_on_issue_without_project_id() {
    let fake = Arc::new(Fake::default());
    let items = [ISSUE_A, ISSUE_NO_PROJECT].join(",");
    fake.issues
        .lock()
        .unwrap()
        .push_back(Ok(issues_page(&items, None)));
    let err = provider(&fake).issues(&project_scope()).await.unwrap_err();
    assert!(
        matches!(&err, BoardError::Provider(m) if m.contains("project")),
        "{err:?}"
    );
}

#[tokio::test]
async fn has_next_page_without_cursor_is_an_error() {
    let fake = Arc::new(Fake::default());
    fake.issues.lock().unwrap().push_back(Ok(format!(
        r#"{{"issues":[{ISSUE_A}],"pageInfo":{{"hasNextPage":true}}}}"#
    )));
    let err = provider(&fake).issues(&scope()).await.unwrap_err();
    assert!(
        matches!(&err, BoardError::Provider(m) if m.contains("next page without cursor")),
        "{err:?}"
    );
}

#[tokio::test]
async fn full_page_without_paging_info_is_an_error() {
    let full = vec![ISSUE_A; PAGE_LIMIT as usize].join(",");
    // No pageInfo and no recognised cursor key (an unknown key is ignored).
    let fake = Arc::new(Fake::default());
    fake.issues
        .lock()
        .unwrap()
        .push_back(Ok(format!(r#"{{"issues":[{full}],"after":"zzz"}}"#)));
    let err = provider(&fake).issues(&scope()).await.unwrap_err();
    assert!(matches!(err, BoardError::Provider(_)), "{err:?}");

    // A short page with no paging information is complete.
    let fake = Arc::new(Fake::default());
    fake.issues
        .lock()
        .unwrap()
        .push_back(Ok(format!(r#"{{"issues":[{ISSUE_A}]}}"#)));
    assert_eq!(provider(&fake).issues(&scope()).await.unwrap().len(), 1);

    // A parsed `nextCursor` continues; an explicit null one ends the list.
    let fake = Arc::new(Fake::default());
    fake.issues
        .lock()
        .unwrap()
        .push_back(Ok(format!(r#"{{"issues":[{full}],"nextCursor":"n1"}}"#)));
    fake.issues
        .lock()
        .unwrap()
        .push_back(Ok(format!(r#"{{"issues":[{ISSUE_A}],"nextCursor":null}}"#)));
    assert_eq!(provider(&fake).issues(&scope()).await.unwrap().len(), 101);
    assert_eq!(fake.calls()[1].2["cursor"], json!("n1"));
}

#[test]
fn unauthorized_status_codes_match_whole_tokens_only() {
    assert!(looks_unauthorized("HTTP 401"));
    assert!(looks_unauthorized("status=403: nope"));
    assert!(looks_unauthorized("Unauthorized"));
    assert!(!looks_unauthorized("issue ABC-4015 not found"));
    assert!(!looks_unauthorized("request id 14013"));
    assert!(!looks_unauthorized("connection reset"));
}

#[tokio::test]
async fn issue_id_containing_401_is_not_unauthorized_on_write() {
    let fake = Arc::new(Fake::default());
    fake.issues
        .lock()
        .unwrap()
        .push_back(Ok(issues_page(ISSUE_A, None)));
    fake.statuses
        .lock()
        .unwrap()
        .push_back(Ok(STATUSES_ARRAY.into()));
    fake.save
        .lock()
        .unwrap()
        .push_back(Err(anyhow::anyhow!("timed out saving ABC-4015")));
    assert!(matches!(
        provider(&fake)
            .set_issue_state(&scope(), "i-a", "s-prog")
            .await,
        Err(BoardError::Uncertain(_))
    ));
}

#[tokio::test]
async fn set_issue_state_refuses_a_state_outside_the_workflow_without_writing() {
    let fake = Arc::new(Fake::default());
    fake.issues
        .lock()
        .unwrap()
        .push_back(Ok(issues_page(ISSUE_A, None)));
    fake.statuses
        .lock()
        .unwrap()
        .push_back(Ok(STATUSES_ARRAY.into()));
    let err = provider(&fake)
        .set_issue_state(&scope(), "i-a", "s-foreign")
        .await
        .unwrap_err();
    assert!(matches!(err, BoardError::Provider(_)), "{err:?}");
    assert_eq!(fake.tools(), ["list_issues", "list_issue_statuses"]);
}

/// `McpManagerCaller::call`'s result handling, with a fake `ToolCallResult`.
#[test]
fn tool_result_text_blocks_join_and_is_error_becomes_tool_error() {
    use crate::mcp::{ContentBlock, ToolCallResult};
    let text = |t: &str| ContentBlock::Text { text: t.into() };
    let result = ToolCallResult {
        content: vec![text("a"), text("b")],
        is_error: false,
    };
    assert_eq!(tool_result_text(result).unwrap(), "a\nb");
    let result = ToolCallResult {
        content: vec![text("bad"), text("input")],
        is_error: true,
    };
    let err = tool_result_text(result).unwrap_err();
    assert_eq!(err.downcast_ref::<ToolError>().unwrap().0, "bad\ninput");
}

#[test]
fn next_cursor_handles_every_paging_shape() {
    use serde_json::json;
    let full = PAGE_LIMIT as usize;
    // pageInfo without hasNextPage on a full page is not proof of the end.
    assert!(next_cursor(&json!({"pageInfo": {"endCursor": "c"}}), full).is_err());
    assert_eq!(
        next_cursor(&json!({"pageInfo": {"endCursor": "c"}}), 3).unwrap(),
        None
    );
    // Top-level hasNextPage is honored in both directions.
    assert_eq!(
        next_cursor(&json!({"hasNextPage": false}), full).unwrap(),
        None,
        "a last page that is exactly full must not be an error"
    );
    assert_eq!(
        next_cursor(&json!({"hasNextPage": true, "cursor": "c2"}), full).unwrap(),
        Some("c2".to_string())
    );
    assert!(next_cursor(&json!({"hasNextPage": true}), full).is_err());
    // A bare null `cursor` proves nothing on a full page; `nextCursor: null` does.
    assert!(next_cursor(&json!({"cursor": null}), full).is_err());
    assert_eq!(
        next_cursor(&json!({"nextCursor": null}), full).unwrap(),
        None
    );
}

#[test]
fn pre_send_manager_failures_are_definitive_not_uncertain() {
    assert!(failed_before_send(
        "MCP server 'Linear' failed to connect: boom"
    ));
    assert!(failed_before_send("MCP server 'Linear' not connected"));
    assert!(!failed_before_send(
        "connection reset while reading the reply"
    ));
}

// ---- captured shapes -------------------------------------------------------
// One read-only capture from the real Linear MCP (list_issue_statuses and
// list_issues for a team), descriptions trimmed. These replace the earlier
// "assumed shape" guesses as the reference shape.

const REAL_STATUSES: &str = include_str!("fixtures/linear_list_issue_statuses.json");
const REAL_ISSUES: &str = include_str!("fixtures/linear_list_issues.json");
const REAL_TEAM_ID: &str = "a465c3cf-3a11-4b7a-a2a0-439a489281f5";

fn real_scope() -> BoardScope {
    BoardScope {
        team: REAL_TEAM_ID.into(),
        ..scope()
    }
}

#[tokio::test]
async fn real_capture_states_come_in_type_order_and_keep_duplicate_type_provider_order() {
    let fake = Arc::new(Fake::default());
    fake.statuses
        .lock()
        .unwrap()
        .push_back(Ok(REAL_STATUSES.into()));
    let states = provider(&fake)
        .workflow_states(&real_scope())
        .await
        .unwrap();
    let names: Vec<_> = states.iter().map(|s| s.name.as_str()).collect();
    // triage, backlog, unstarted (Ready, Todo), started (In Review, In Progress),
    // completed, then the terminal types. Same-type states keep the order the
    // provider listed them.
    let kinds: Vec<_> = states.iter().map(|s| s.kind.as_str()).collect();
    assert_eq!(
        kinds,
        [
            "triage",
            "backlog",
            "unstarted",
            "unstarted",
            "started",
            "started",
            "completed",
            "canceled",
            "duplicate"
        ],
        "{kinds:?}"
    );
    assert_eq!(
        names,
        [
            "Triage",
            "Backlog",
            "Ready",
            "Todo",
            "In Review",
            "In Progress",
            "Done",
            "Canceled",
            "Duplicate"
        ],
        "{names:?}"
    );
}

#[tokio::test]
async fn real_capture_issues_resolve_to_column_ids_and_page_on_the_top_level_cursor() {
    let fake = Arc::new(Fake::default());
    fake.statuses
        .lock()
        .unwrap()
        .push_back(Ok(REAL_STATUSES.into()));
    // Page 1 says hasNextPage with a cursor; page 2 is the same capture
    // without the flag so the read ends.
    let mut last: serde_json::Value = serde_json::from_str(REAL_ISSUES).unwrap();
    last["hasNextPage"] = serde_json::json!(false);
    fake.issues
        .lock()
        .unwrap()
        .push_back(Ok(REAL_ISSUES.into()));
    fake.issues.lock().unwrap().push_back(Ok(last.to_string()));
    let issues = provider(&fake).issues(&real_scope()).await.unwrap();
    assert_eq!(issues.len(), 10, "two pages of five");
    let calls = fake.calls();
    let second = calls
        .iter()
        .filter(|c| c.1 == "list_issues")
        .nth(1)
        .unwrap();
    assert_eq!(second.2["cursor"], "4f6d0ab8-fb97-4a6c-b1f8-861603d042eb");
    // Each issue's column id is a real workflow-state id, found by the state
    // NAME because the real issue carries `status`/`statusType` strings only.
    let todo = issues.iter().find(|i| i.identifier == "PRICE-58").unwrap();
    assert_eq!(todo.state_id, "9681f608-8b08-44c3-8744-3f38fcdf5092");
    let triage = issues.iter().find(|i| i.identifier == "PRICE-60").unwrap();
    assert_eq!(triage.state_id, "37af9304-9868-4748-bbd6-4216267afb32");
}

#[tokio::test]
async fn real_capture_with_a_team_key_binding_fails_closed_and_says_why() {
    // The real issue carries a team uuid (`teamId`) and a display name (`team`).
    // A binding written as the key or the display name must not select issues,
    // and the board must say what is wrong instead of showing an empty board.
    for team in ["PRICE", "Priceless"] {
        let fake = Arc::new(Fake::default());
        fake.statuses
            .lock()
            .unwrap()
            .push_back(Ok(REAL_STATUSES.into()));
        let mut last: serde_json::Value = serde_json::from_str(REAL_ISSUES).unwrap();
        last["hasNextPage"] = serde_json::json!(false);
        fake.issues.lock().unwrap().push_back(Ok(last.to_string()));
        let scope = BoardScope {
            team: team.into(),
            ..scope()
        };
        let error = provider(&fake).issues(&scope).await.unwrap_err();
        let text = error.to_string();
        assert!(text.contains("team id (uuid)"), "{team}: {text}");
    }
}

#[tokio::test]
async fn a_genuinely_empty_team_is_still_an_empty_board_not_an_error() {
    let fake = Arc::new(Fake::default());
    fake.issues
        .lock()
        .unwrap()
        .push_back(Ok(issues_page("", None)));
    assert!(provider(&fake).issues(&scope()).await.unwrap().is_empty());
}
