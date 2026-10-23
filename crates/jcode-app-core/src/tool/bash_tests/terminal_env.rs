use super::super::build_shell_command;
use super::super::shell_single_quote;
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
            crate::terminal_launch::apply_scoped_client_terminal_env(command.as_std_mut());
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
