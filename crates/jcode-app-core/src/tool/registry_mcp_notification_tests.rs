//! End to end: a real stdio MCP server that opted in (global config) changes
//! its tool catalog and sends `notifications/tools/list_changed`. The model
//! facing registry must then expose the new tool without a reconnect.
use super::Registry;
use std::time::Duration;

const SERVER: &str = r#"
D="$1"
(
  while [ ! -f "$D/go" ]; do sleep 0.05; done
  printf '{"jsonrpc":"2.0","method":"notifications/tools/list_changed"}\n'
  printf '{"jsonrpc":"2.0","method":"notifications/prompts/list_changed"}\n'
) &
while IFS= read -r line; do
  id=$(printf '%s' "$line" | sed -n 's/.*"id":\([0-9][0-9]*\).*/\1/p')
  case "$line" in
    *'"initialize"'*)
      printf '{"jsonrpc":"2.0","id":%s,"result":{"protocolVersion":"2025-11-25","capabilities":{"tools":{"listChanged":true},"prompts":{"listChanged":true}},"serverInfo":{"name":"fake","version":"0"}}}\n' "$id"
      ;;
    *'"tools/list"'*)
      if [ -f "$D/v2" ]; then
        printf '{"jsonrpc":"2.0","id":%s,"result":{"tools":[{"name":"two","description":"second","inputSchema":{"type":"object"}}]}}\n' "$id"
      else
        printf '{"jsonrpc":"2.0","id":%s,"result":{"tools":[{"name":"one","description":"first","inputSchema":{"type":"object"}}]}}\n' "$id"
      fi
      ;;
    *'"prompts/list"'*)
      if [ -f "$D/v2" ]; then
        printf '{"jsonrpc":"2.0","id":%s,"result":{"prompts":[{"name":"p1"}]}}\n' "$id"
      else
        printf '{"jsonrpc":"2.0","id":%s,"result":{"prompts":[]}}\n' "$id"
      fi
      ;;
  esac
done
"#;

async fn wait_for(registry: &Registry, name: &str, present: bool) {
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            if registry.tool_names().await.iter().any(|n| n == name) == present {
                return;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .unwrap_or_else(|_| panic!("registry never reached {name} present={present}"));
}

async fn mcp_list(registry: &Registry) -> String {
    let ctx = crate::tool::ToolContext {
        session_id: "test-session".to_string(),
        message_id: "m".to_string(),
        tool_call_id: "c".to_string(),
        working_dir: None,
        stdin_request_tx: None,
        graceful_shutdown_signal: None,
        execution_mode: crate::tool::ToolExecutionMode::Direct,
    };
    registry
        .execute("mcp", serde_json::json!({"action": "list"}), ctx)
        .await
        .unwrap()
        .output
}

async fn wait_for_list(registry: &Registry, needle: &str) {
    tokio::time::timeout(Duration::from_secs(10), async {
        while !mcp_list(registry).await.contains(needle) {
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .unwrap_or_else(|_| panic!("mcp list never showed {needle}"));
}

async fn run(opted_in: bool) {
    let sandbox = crate::auth::test_sandbox::AuthTestSandbox::new().unwrap();
    let state = tempfile::tempdir().unwrap();
    let mut server = serde_json::json!({
        "command": "/bin/sh",
        "args": ["-c", SERVER, "fake", state.path()],
        "shared": false,
    });
    if opted_in {
        server["notifications"] =
            serde_json::json!({"refresh": ["tools_list_changed", "prompts_list_changed"]});
    }
    std::fs::write(
        sandbox.root().join("mcp.json"),
        serde_json::json!({"mcpServers": {"fake": server}}).to_string(),
    )
    .unwrap();

    let registry = Registry::empty();
    registry
        .register_mcp_tools_for_dir(None, None, None, None)
        .await;
    wait_for(&registry, "mcp__fake__one", true).await;

    std::fs::write(state.path().join("v2"), "").unwrap();
    std::fs::write(state.path().join("go"), "").unwrap();

    if opted_in {
        wait_for(&registry, "mcp__fake__two", true).await;
        wait_for(&registry, "mcp__fake__one", false).await;
        // Prompts re-read from the same server; surfaced as a count in `mcp list`.
        wait_for_list(&registry, "1 prompts").await;
    } else {
        // Opted out: the notification is ignored, the catalog stays as first read.
        tokio::time::sleep(Duration::from_millis(800)).await;
        let names = registry.tool_names().await;
        assert!(names.iter().any(|n| n == "mcp__fake__one"));
        assert!(!names.iter().any(|n| n == "mcp__fake__two"));
        assert!(!mcp_list(&registry).await.contains("notifications:"));
    }
}

#[tokio::test]
async fn opted_in_tools_list_changed_refreshes_the_model_facing_registry() {
    run(true).await;
}

#[tokio::test]
async fn opted_out_server_keeps_its_first_catalog() {
    run(false).await;
}
