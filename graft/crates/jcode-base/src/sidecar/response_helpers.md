# crates/jcode-base/src/sidecar/response_helpers.rs

- resolve_openai_request_model · function · L3-L21 — pub(super) fn resolve_openai_request_model(
- build_openai_request · function · L23-L55 — pub(super) fn build_openai_request(
- classify_openai_model_unavailable · function · L57-L95 — pub(super) fn classify_openai_model_unavailable(status: StatusCode, body: &str) -> Option<String>
- is_openai_model_unavailable · function · L97-L99 — pub(super) fn is_openai_model_unavailable(status: StatusCode, body: &str) -> bool
- collect_openai_sse_text · function · L105-L162 — pub(super) async fn collect_openai_sse_text(
- parse_openai_usage · function · L168-L186 — pub(super) fn parse_openai_usage(
- openai_sidecar_response_model · function · L188-L194 — pub(super) fn openai_sidecar_response_model(response: Option<&serde_json::Value>) -> Option<&str>
- record_openai_sidecar_usage · function · L196-L214 — pub(super) fn record_openai_sidecar_usage(
- extract_openai_response_text · function · L217-L237 — pub(super) fn extract_openai_response_text(result: &serde_json::Value) -> Result<String>
- build_claude_system_param · function · L239-L252 — pub(super) fn build_claude_system_param(system: &str) -> Option<ClaudeApiSystem<'_>>
- build_claude_api_key_system_param · function · L259-L267 — pub(super) fn build_claude_api_key_system_param(system: &str) -> Option<ClaudeApiSystem<'_>>
- anthropic_sidecar_prefers_api_key · function · L274-L282 — pub(super) fn anthropic_sidecar_prefers_api_key() -> bool
- is_anthropic_oauth_forbidden · function · L286-L291 — pub(super) fn is_anthropic_oauth_forbidden(err: &anyhow::Error) -> bool
