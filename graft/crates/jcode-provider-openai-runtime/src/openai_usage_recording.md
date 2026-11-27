# crates/jcode-provider-openai-runtime/src/openai_usage_recording.rs

- OAuthUsageRecorder · struct · L5-L10 — pub(super) struct OAuthUsageRecorder
- credential_label · function · L14-L30 — fn credential_label(creds: &CodexCredentials, accounts: &[OpenAiAccount]) -> Option<String>
- capture · function · L33-L55 — pub(super) fn capture(creds: &CodexCredentials, request: &Value) -> Self
- observe · function · L57-L95 — pub(super) async fn observe(&mut self, event: &StreamEvent)
- tests · module · L99-L202 — mod tests
- creds · function · L102-L110 — fn creds(access: &str, refresh: &str) -> CodexCredentials
- account · function · L111-L121 — fn account(label: &str, credentials: &CodexCredentials) -> OpenAiAccount
- attribution_uses_request_identity_not_active_account · function · L124-L138 — fn attribution_uses_request_identity_not_active_account()
- api_credentials_do_not_record · function · L141-L156 — async fn api_credentials_do_not_record()
- terminal_usage_recorded_once_before_delivery · function · L159-L201 — async fn terminal_usage_recorded_once_before_delivery()
