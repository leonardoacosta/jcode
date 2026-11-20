//! Wire behavior of the Tasks extension against scripted stdio servers.
use super::*;
use crate::mcp::{McpClient, McpServerConfig};
use serde_json::json;
use std::time::Duration;

/// A modern server that advertises the Tasks extension. `tools/call` answers
/// with a task handle for `slow` and a plain result for `fast`. State of the
/// one task lives in `$D/state` (default working).
const TASK_SERVER: &str = r#"
D="$1"
while IFS= read -r line; do
  printf '%s\n' "$line" >> "$D/log"
  id=$(printf '%s' "$line" | sed -n 's/.*"id":\([0-9][0-9]*\).*/\1/p')
  state=working; [ -f "$D/state" ] && state=$(cat "$D/state")
  case "$line" in
    *'"server/discover"'*)
      printf '{"jsonrpc":"2.0","id":%s,"result":{"resultType":"complete","supportedVersions":["2026-07-28"],"capabilities":{"tools":{},"extensions":{"io.modelcontextprotocol/tasks":{}}}}}\n' "$id" ;;
    *'"initialize"'*)
      printf '{"jsonrpc":"2.0","id":%s,"result":{"protocolVersion":"2025-11-25","capabilities":{"tools":{}},"serverInfo":{"name":"tasky","version":"0"}}}\n' "$id" ;;
    *'"tools/list"'*)
      printf '{"jsonrpc":"2.0","id":%s,"result":{"tools":[{"name":"slow","inputSchema":{"type":"object"}},{"name":"fast","inputSchema":{"type":"object"}}]}}\n' "$id" ;;
    *'"tools/call"'*'"slow"'*)
      printf '{"jsonrpc":"2.0","id":%s,"result":{"resultType":"task","taskId":"t-1","status":"working","createdAt":"2025-11-25T10:30:00Z","lastUpdatedAt":"2025-11-25T10:30:00Z","ttlMs":3000000000000,"pollIntervalMs":1000}}\n' "$id" ;;
    *'"tools/call"'*)
      printf '{"jsonrpc":"2.0","id":%s,"result":{"content":[{"type":"text","text":"done"}],"isError":false}}\n' "$id" ;;
    *'"tasks/get"'*'"gone"'*)
      printf '{"jsonrpc":"2.0","id":%s,"error":{"code":-32602,"message":"Failed to retrieve task: Task not found"}}\n' "$id" ;;
    *'"tasks/get"'*)
      if [ "$state" = completed ]; then
        printf '{"jsonrpc":"2.0","id":%s,"result":{"resultType":"complete","taskId":"t-1","status":"completed","createdAt":"2025-11-25T10:30:00Z","lastUpdatedAt":"2025-11-25T10:50:00Z","ttlMs":3000000000000,"result":{"content":[{"type":"text","text":"Hello"}],"isError":false}}}\n' "$id"
      else
        printf '{"jsonrpc":"2.0","id":%s,"result":{"resultType":"complete","taskId":"t-1","status":"%s","createdAt":"2025-11-25T10:30:00Z","lastUpdatedAt":"2025-11-25T10:40:00Z","ttlMs":3000000000000}}\n' "$id" "$state"
      fi ;;
    *'"tasks/cancel"'*)
      printf '{"jsonrpc":"2.0","id":%s,"result":{"resultType":"complete"}}\n' "$id" ;;
  esac
done
"#;

fn config_for(dir: &std::path::Path, tasks: bool) -> McpServerConfig {
    serde_json::from_value(json!({
        "command": "/bin/sh", "args": ["-c", TASK_SERVER, "fake", dir],
        "shared": true, "timeout_secs": 10, "tasks": tasks,
    }))
    .unwrap()
}

fn log(dir: &std::path::Path) -> Vec<Value> {
    std::fs::read_to_string(dir.join("log"))
        .unwrap_or_default()
        .lines()
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect()
}

fn calls(dir: &std::path::Path, method: &str) -> Vec<Value> {
    log(dir)
        .into_iter()
        .filter(|v| v["method"] == method)
        .collect()
}

async fn connect(dir: &std::path::Path, tasks: bool) -> McpClient {
    McpClient::connect("srv".into(), &config_for(dir, tasks))
        .await
        .expect("connect")
}

#[tokio::test]
async fn opted_in_server_returns_a_task_handle_and_the_call_is_not_replayed() {
    let dir = tempfile::tempdir().unwrap();
    let client = connect(dir.path(), true).await;
    let handle = client.handle();
    assert!(handle.advertises_tasks());
    let outcome = handle.call_tool_outcome("slow", json!({})).await.unwrap();
    let CallOutcome::Task(task) = outcome else {
        panic!("expected a task")
    };
    assert_eq!(task.task_id, "t-1");
    assert_eq!(task.poll_interval_ms, Some(1000));
    assert_eq!(
        calls(dir.path(), "tools/call").len(),
        1,
        "one call, no replay"
    );
    // The call declared the extension.
    let call = &calls(dir.path(), "tools/call")[0];
    assert_eq!(
        call["params"]["_meta"]["io.modelcontextprotocol/clientCapabilities"]["extensions"]["io.modelcontextprotocol/tasks"],
        json!({})
    );
}

#[tokio::test]
async fn a_plain_result_is_still_a_plain_result_for_a_task_capable_server() {
    let dir = tempfile::tempdir().unwrap();
    let client = connect(dir.path(), true).await;
    let CallOutcome::Complete(r) = client
        .handle()
        .call_tool_outcome("fast", json!({}))
        .await
        .unwrap()
    else {
        panic!("expected a complete result")
    };
    assert!(!r.is_error);
}

#[tokio::test]
async fn a_server_that_did_not_opt_in_never_sees_the_extension() {
    let dir = tempfile::tempdir().unwrap();
    // Not opted in and not opted in to notifications: no probe, plain handshake.
    let client = connect(dir.path(), false).await;
    let handle = client.handle();
    assert!(!handle.advertises_tasks());
    assert!(
        calls(dir.path(), "server/discover").is_empty(),
        "no era probe"
    );
    // The server would hand out a task for `slow`; a client that never
    // declared the extension must refuse it rather than track it.
    let err = handle.call_tool("slow", json!({})).await.err();
    let _ = err;
    assert!(calls(dir.path(), "tasks/get").is_empty());
    assert!(calls(dir.path(), "tasks/cancel").is_empty());
    for line in log(dir.path()) {
        assert!(
            line["params"]["_meta"]["io.modelcontextprotocol/clientCapabilities"]["extensions"]
                .is_null(),
            "{line}"
        );
    }
}

#[tokio::test]
async fn a_task_handle_without_negotiated_support_is_a_protocol_error() {
    // The server answers `slow` with a task even though the client did not
    // declare the extension (config tasks=false but notifications opted in so
    // the modern era is probed).
    let dir = tempfile::tempdir().unwrap();
    let mut cfg = config_for(dir.path(), false);
    cfg.notifications = Some(crate::mcp::notifications::McpNotificationConfig {
        refresh: vec![crate::mcp::notifications::NotificationClass::ToolsListChanged],
        ..Default::default()
    });
    let client = McpClient::connect("srv".into(), &cfg)
        .await
        .expect("connect");
    let handle = client.handle();
    assert!(handle.is_modern() && !handle.advertises_tasks());
    let err = handle
        .call_tool_outcome("slow", json!({}))
        .await
        .expect_err("must reject");
    assert!(format!("{err:#}").contains("did not negotiate"), "{err:#}");
    // And no polling or cancelling is possible for that handle.
    assert!(handle.get_task("t-1").await.is_err());
    assert!(handle.cancel_task("t-1").await.is_err());
    assert!(calls(dir.path(), "tasks/get").is_empty());
    assert!(calls(dir.path(), "tasks/cancel").is_empty());
}

#[tokio::test]
async fn call_tool_for_callers_that_cannot_track_refuses_the_handle() {
    let dir = tempfile::tempdir().unwrap();
    let client = connect(dir.path(), true).await;
    let err = client
        .handle()
        .call_tool("slow", json!({}))
        .await
        .expect_err("refuse");
    assert!(format!("{err:#}").contains("did not negotiate"));
}

#[tokio::test]
async fn tasks_get_and_cancel_wire_shapes() {
    let dir = tempfile::tempdir().unwrap();
    let client = connect(dir.path(), true).await;
    let handle = client.handle();
    let TaskPoll::Task(t) = handle.get_task("t-1").await.unwrap() else {
        panic!()
    };
    assert_eq!(t.status, RemoteStatus::Working);
    std::fs::write(dir.path().join("state"), "completed").unwrap();
    let TaskPoll::Task(t) = handle.get_task("t-1").await.unwrap() else {
        panic!()
    };
    assert_eq!(t.status, RemoteStatus::Completed);
    assert_eq!(t.result.unwrap()["content"][0]["text"], "Hello");
    handle.cancel_task("t-1").await.unwrap();

    let get = &calls(dir.path(), "tasks/get")[0];
    assert_eq!(get["params"]["taskId"], "t-1");
    assert_eq!(
        get["params"]["_meta"]["io.modelcontextprotocol/clientCapabilities"]["extensions"]["io.modelcontextprotocol/tasks"],
        json!({}),
        "every tasks request declares the extension"
    );
    assert_eq!(
        calls(dir.path(), "tasks/cancel")[0]["params"]["taskId"],
        "t-1"
    );
    // Never the incompatible 2025-11-25 methods.
    assert!(calls(dir.path(), "tasks/result").is_empty());
}

#[tokio::test]
async fn unknown_task_is_not_found_not_a_transport_error() {
    let dir = tempfile::tempdir().unwrap();
    let client = connect(dir.path(), true).await;
    match client.handle().get_task("gone").await.unwrap() {
        TaskPoll::NotFound(msg) => assert!(msg.contains("not found")),
        other => panic!("{other:?}"),
    }
}

#[tokio::test]
async fn project_local_config_cannot_opt_in_to_tasks() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join(".mcp.json"),
        json!({"mcpServers": {"p": {"command": "/bin/true", "tasks": true}}}).to_string(),
    )
    .unwrap();
    let config = crate::mcp::McpConfig::load_for_dir(Some(dir.path()));
    if let Some(server) = config.servers.get("p") {
        assert!(
            !server.tasks,
            "a repository file must not grant the Tasks extension"
        );
    }
}

#[test]
fn server_identity_is_stable_secret_free_and_sensitive_to_what_matters() {
    let base: McpServerConfig = serde_json::from_value(json!({
        "type": "http", "url": "https://user:pw@example.com/mcp",
        "headers": {"Authorization": "Bearer SECRET-TOKEN"}, "tasks": true,
    }))
    .unwrap();
    let id = server_identity("srv", &base);
    assert_eq!(id, server_identity("srv", &base), "stable");
    assert_eq!(id.len(), 64);
    assert!(!id.contains("SECRET") && !id.contains("pw"));
    // The credential value changes the identity (a different auth context)...
    let mut rotated = base.clone();
    rotated
        .headers
        .insert("Authorization".into(), "Bearer OTHER".into());
    assert_ne!(id, server_identity("srv", &rotated));
    // ...as do the name and the endpoint.
    assert_ne!(id, server_identity("other", &base));
    let mut moved = base.clone();
    moved.url = Some("https://evil.example.com/mcp".into());
    assert_ne!(id, server_identity("srv", &moved));
    // Userinfo in the URL is not part of it.
    let mut no_userinfo = base.clone();
    no_userinfo.url = Some("https://example.com/mcp".into());
    assert_eq!(id, server_identity("srv", &no_userinfo));
    // Fields that do not change which server this is do not change identity.
    let mut flipped = base.clone();
    flipped.timeout_secs = Some(99);
    assert_eq!(id, server_identity("srv", &flipped));
    let _ = Duration::ZERO;
}

fn manager_for(dir: &std::path::Path, tasks: bool) -> crate::mcp::McpManager {
    let mut config = crate::mcp::McpConfig::default();
    config.servers.insert("srv".into(), config_for(dir, tasks));
    crate::mcp::McpManager::with_config(config)
}

fn tracker_for(
    manager: crate::mcp::McpManager,
    tmp: &std::path::Path,
) -> (
    crate::mcp::task_tracker::McpTaskTracker,
    std::sync::Arc<crate::background::BackgroundTaskManager>,
) {
    let background = std::sync::Arc::new(
        crate::background::BackgroundTaskManager::with_output_dir(tmp.join("bg")),
    );
    let source = std::sync::Arc::new(crate::mcp::ManagerTaskSource::new(std::sync::Arc::new(
        manager,
    )));
    (
        crate::mcp::task_tracker::McpTaskTracker::new(source, std::sync::Arc::clone(&background)),
        background,
    )
}

#[tokio::test]
async fn mcp_task_tracking_call_registers_a_background_task_and_never_replays() {
    let dir = tempfile::tempdir().unwrap();
    let bg_dir = tempfile::tempdir().unwrap();
    let (tracker, background) = tracker_for(manager_for(dir.path(), true), bg_dir.path());
    let caller = manager_for(dir.path(), true);
    let result = caller
        .call_tool_with_tracker(
            Some(tracker.clone()),
            "srv",
            "slow",
            json!({}),
            "sess",
            "call-1",
        )
        .await
        .expect("task handle is tracked");
    let crate::mcp::protocol::ContentBlock::Text { text } = &result.content[0] else {
        panic!("text note")
    };
    assert!(text.contains("background task"), "note: {text}");
    assert!(
        !text.contains("t-1"),
        "no server-supplied id in the model note"
    );
    let listed = background.list().await;
    assert_eq!(listed.len(), 1);
    let record = background.status(&listed[0].task_id).await.unwrap();
    let mcp = record.mcp_task.expect("durable mcp record");
    assert_eq!(mcp.remote_task_id, "t-1");
    assert_eq!(mcp.identity.len(), 64, "sha-256 fingerprint, not a secret");
    assert_eq!(calls(dir.path(), "tools/call").len(), 1, "never replayed");
    tracker.shutdown();
}

#[tokio::test]
async fn mcp_task_tracking_completes_through_polling_against_a_live_stub() {
    let dir = tempfile::tempdir().unwrap();
    let bg_dir = tempfile::tempdir().unwrap();
    let (tracker, background) = tracker_for(manager_for(dir.path(), true), bg_dir.path());
    let caller = manager_for(dir.path(), true);
    caller
        .call_tool_with_tracker(
            Some(tracker.clone()),
            "srv",
            "slow",
            json!({}),
            "sess",
            "call-1",
        )
        .await
        .unwrap();
    std::fs::write(dir.path().join("state"), "completed").unwrap();
    let id = background.list().await[0].task_id.clone();
    tracker.hint("srv", "t-1");
    let mut done = false;
    for _ in 0..100 {
        let status = background.status(&id).await.unwrap();
        if status.status == crate::bus::BackgroundTaskStatus::Completed {
            done = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert!(done, "hint woke a tasks/get that completed the task");
    tracker.shutdown();
}

#[tokio::test]
async fn mcp_task_tracking_without_a_tracker_refuses_a_handle_and_never_polls_it() {
    let dir = tempfile::tempdir().unwrap();
    let caller = manager_for(dir.path(), true);
    // No tracker: the extension is not declared. A server that hands out a task
    // anyway is refused, and nothing is polled.
    let err = caller
        .call_tool_with_tracker(None, "srv", "slow", json!({}), "sess", "call-1")
        .await
        .expect_err("untracked handle is an error");
    assert!(format!("{err:#}").contains("did not negotiate"), "{err:#}");
    assert!(!declared_tasks(&calls(dir.path(), "tools/call")[0]));
    assert!(calls(dir.path(), "tasks/get").is_empty());
}

#[tokio::test]
async fn mcp_task_tracking_source_refuses_servers_that_did_not_opt_in() {
    use crate::mcp::tasks::{ResolveError, TaskSource};
    let dir = tempfile::tempdir().unwrap();
    let source =
        crate::mcp::ManagerTaskSource::new(std::sync::Arc::new(manager_for(dir.path(), false)));
    assert!(matches!(
        source.resolve("srv").await,
        Err(ResolveError::Unknown)
    ));
    assert!(matches!(
        source.resolve("other").await,
        Err(ResolveError::Unknown)
    ));
}

fn declared_tasks(call: &Value) -> bool {
    call["params"]["_meta"]["io.modelcontextprotocol/clientCapabilities"]["extensions"]["io.modelcontextprotocol/tasks"]
        == json!({})
}

#[tokio::test]
async fn mcp_task_tracking_plain_call_tool_never_declares_the_extension() {
    let dir = tempfile::tempdir().unwrap();
    let client = connect(dir.path(), true).await;
    // A caller that cannot track gets a synchronous request: the extension is
    // not declared, so the stub (which would hand out a task) is never asked
    // to create one on its behalf.
    let _ = client.handle().call_tool("fast", json!({})).await;
    let call = &calls(dir.path(), "tools/call")[0];
    assert!(!declared_tasks(call), "plain call must not declare Tasks");
}

#[tokio::test]
async fn mcp_task_tracking_manager_without_tracker_does_not_declare_the_extension() {
    let dir = tempfile::tempdir().unwrap();
    let caller = manager_for(dir.path(), true);
    let _ = caller
        .call_tool_with_tracker(None, "srv", "fast", json!({}), "sess", "call-1")
        .await;
    let call = &calls(dir.path(), "tools/call")[0];
    assert!(!declared_tasks(call));
}

#[tokio::test]
async fn mcp_task_tracking_non_shared_server_never_opts_in() {
    let dir = tempfile::tempdir().unwrap();
    let mut cfg = config_for(dir.path(), true);
    cfg.shared = false;
    let client = McpClient::connect("srv".into(), &cfg)
        .await
        .expect("connect");
    assert!(!client.handle().advertises_tasks());
    let _ = client.handle().call_tool_outcome("fast", json!({})).await;
    assert!(!declared_tasks(&calls(dir.path(), "tools/call")[0]));
}

#[tokio::test]
async fn mcp_task_tracking_identity_comes_from_the_connection_config() {
    let dir = tempfile::tempdir().unwrap();
    let client = connect(dir.path(), true).await;
    assert_eq!(
        client.handle().identity(),
        crate::mcp::tasks::server_identity("srv", &config_for(dir.path(), true))
    );
    let other = tempfile::tempdir().unwrap();
    assert_ne!(
        client.handle().identity(),
        crate::mcp::tasks::server_identity("srv", &config_for(other.path(), true)),
        "a different launch config reads as a different server"
    );
}
