# crates/jcode-harness-api-server/src/runtime_command_compat_tests.rs

- next_json · function · L7-L17 — async fn next_json(reader: &mut (impl AsyncBufRead + Unpin)) -> Value
- reply_to · function · L19-L26 — async fn reply_to(reader: &mut (impl AsyncBufRead + Unpin), id: u64) -> Value
- runtime_capability_case · function · L28-L186 — async fn runtime_capability_case(initial_timeout: bool, refreshed_support: Option<bool>)
- runtime_commands_recover_from_a_timed_out_hello_probe_without_reattaching · function · L189-L191 — async fn runtime_commands_recover_from_a_timed_out_hello_probe_without_reattaching()
- runtime_commands_refresh_an_old_capability_snapshot_without_replaying_mutations · function · L194-L196 — async fn runtime_commands_refresh_an_old_capability_snapshot_without_replaying_mutations()
- runtime_commands_keep_older_daemons_connected_and_explain_safe_upgrade · function · L199-L201 — async fn runtime_commands_keep_older_daemons_connected_and_explain_safe_upgrade()
- runtime_commands_distinguish_probe_failure_from_unsupported_runtime · function · L204-L206 — async fn runtime_commands_distinguish_probe_failure_from_unsupported_runtime()
