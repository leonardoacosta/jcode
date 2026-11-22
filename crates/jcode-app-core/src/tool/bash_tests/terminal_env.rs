use super::super::{apply_tool_terminal_env, build_shell_command, shell_single_quote};
use super::{BashTool, Tool, make_agent_ctx, make_ctx};
use crate::bus::BackgroundTaskStatus;
use serde_json::json;
use std::collections::HashMap;

#[test]
fn cargo_wrapper_path_is_shell_quoted() {
    assert_eq!(shell_single_quote("a'b"), "'a'\"'\"'b'");
}

#[tokio::test]
async fn build_shell_command_applies_only_client_terminal_environment() {
    crate::hooks::with_client_terminal_env(
        vec![
            ("HERDR_PANE_ID".into(), "client-pane".into()),
            ("PATH".into(), "/untrusted".into()),
            ("LD_PRELOAD".into(), "/untrusted.so".into()),
        ],
        async {
            let mut command = build_shell_command("true");
            apply_tool_terminal_env(
                command.as_std_mut(),
                &crate::terminal_launch::client_terminal_env_snapshot(),
            );
            let env: HashMap<_, _> = command
                .as_std()
                .get_envs()
                .map(|(key, value)| {
                    (
                        key.to_string_lossy().into_owned(),
                        value.map(|value| value.to_string_lossy().into_owned()),
                    )
                })
                .collect();
            assert_eq!(env.get("HERDR_PANE_ID"), Some(&Some("client-pane".into())));
            assert_eq!(env.get("HERDR_ENV"), Some(&None));
            assert_eq!(env.get("TMUX"), Some(&None));
            assert!(!env.contains_key("PATH"));
            assert!(!env.contains_key("LD_PRELOAD"));
        },
    )
    .await;
}

fn herdr_client_env() -> Vec<(String, String)> {
    vec![
        ("HERDR_ENV".into(), "1".into()),
        ("HERDR_PANE_ID".into(), "client-pane".into()),
        ("HERDR_SOCKET_PATH".into(), "/tmp/client-herdr.sock".into()),
    ]
}

/// Prints the Herdr variables a nested agent integration or a Herdr control
/// command would read.
const HERDR_PROBE: &str = "printf 'env=%s pane=%s sock=%s client=%s\\n' \"${HERDR_ENV-unset}\" \"${HERDR_PANE_ID-unset}\" \"${HERDR_SOCKET_PATH-unset}\" \"${JCODE_CLIENT_HERDR_ENV-unset}\"";
const HERDR_EXPECTED: &str = "env=unset pane=client-pane sock=/tmp/client-herdr.sock client=1";

#[tokio::test]
async fn foreground_tool_command_drops_herdr_env_but_keeps_pane_routing() {
    let output = crate::hooks::with_client_terminal_env(herdr_client_env(), async {
        BashTool::new()
            .execute(json!({ "command": HERDR_PROBE }), make_ctx(None))
            .await
            .unwrap()
            .output
    })
    .await;
    assert!(output.contains(HERDR_EXPECTED), "{output}");
}

#[cfg(unix)]
#[tokio::test]
async fn detached_tool_command_drops_herdr_env_but_keeps_pane_routing() {
    let ctx = make_agent_ctx(jcode_agent_runtime::InterruptSignal::new());
    let output = crate::hooks::with_client_terminal_env(herdr_client_env(), async {
        BashTool::new()
            .execute(json!({ "command": HERDR_PROBE }), ctx)
            .await
            .unwrap()
            .output
    })
    .await;
    assert!(output.contains(HERDR_EXPECTED), "{output}");
}

#[tokio::test]
async fn background_tool_command_drops_herdr_env_but_keeps_pane_routing() {
    let result = crate::hooks::with_client_terminal_env(herdr_client_env(), async {
        BashTool::new()
            .execute(
                json!({ "command": HERDR_PROBE, "run_in_background": true, "notify": false }),
                make_ctx(None),
            )
            .await
            .unwrap()
    })
    .await;
    let task_id = result.metadata.unwrap()["task_id"]
        .as_str()
        .unwrap()
        .to_string();
    tokio::time::timeout(std::time::Duration::from_secs(10), async {
        while crate::background::global()
            .status(&task_id)
            .await
            .unwrap()
            .status
            == BackgroundTaskStatus::Running
        {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    let output = crate::background::global().output(&task_id).await.unwrap();
    assert!(output.contains(HERDR_EXPECTED), "{output}");
}

#[tokio::test]
async fn tool_command_without_client_herdr_keeps_no_herdr_identity() {
    let output = crate::hooks::with_client_terminal_env(Vec::new(), async {
        BashTool::new()
            .execute(json!({ "command": HERDR_PROBE }), make_ctx(None))
            .await
            .unwrap()
            .output
    })
    .await;
    assert!(
        output.contains("env=unset pane=unset sock=unset client=unset"),
        "{output}"
    );
}
