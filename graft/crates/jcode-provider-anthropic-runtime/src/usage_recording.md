# crates/jcode-provider-anthropic-runtime/src/usage_recording.rs

- update_usage · function · L5-L31 — pub(super) fn update_usage(state: &mut SseStreamState, usage: UsageInfo)
- send_final_usage · function · L33-L59 — pub(super) async fn send_final_usage(
- matching_label · function · L61-L71 — fn matching_label(token: &str, accounts: &[auth::claude::AnthropicAccount]) -> Option<String>
- credential_label · function · L73-L92 — pub(super) fn credential_label(is_oauth: bool, token: &str) -> Option<String>
- record · function · L94-L129 — pub(super) async fn record(label: Option<&str>, requested_model: &str, state: &SseStreamState)
- tests · module · L132-L368 — mod tests
- account · function · L135-L145 — fn account(label: &str, token: &str) -> auth::claude::AnthropicAccount
- terminal_http_stream_records_once_and_only_for_oauth · function · L148-L328 — fn terminal_http_stream_records_once_and_only_for_oauth()
- Restore · struct · L152-L152 — struct Restore(Option<std::ffi::OsString>);
- drop · function · L154-L159 — fn drop(&mut self)
- cumulative_usage_deltas_preserve_missing_fields · function · L331-L353 — fn cumulative_usage_deltas_preserve_missing_fields()
- matches_request_credential_not_active_account · function · L356-L367 — fn matches_request_credential_not_active_account()
