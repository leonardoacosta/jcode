# crates/jcode-provider-anthropic-runtime/src/reasoning_request.rs

- BINDING_BETA · constant · L6-L6 — const BINDING_BETA: &str = "thinking-binding-controls-2026-08-01";
- adaptive_thinking · function · L8-L17 — pub(super) fn adaptive_thinking(model: &str) -> ApiThinking
- fallback_temperature · function · L19-L21 — pub(super) fn fallback_temperature(model: &str, is_oauth: bool) -> Option<f32>
- with_binding_beta · function · L23-L40 — pub(super) fn with_binding_beta(base: &str, thinking: &Option<ApiThinking>) -> String
- RetrySettings · struct · L44-L49 — pub(super) struct RetrySettings
- from_provider · function · L52-L63 — pub(super) fn from_provider(provider: &AnthropicProvider) -> Self
- reshape · function · L65-L89 — pub(super) fn reshape(&self, request: &mut ApiRequest, model: &str, is_oauth: bool)
- recover_rejected_reasoning · function · L95-L115 — pub(super) fn recover_rejected_reasoning(
- tests · module · L118-L296 — mod tests
- request · function · L122-L136 — fn request() -> ApiRequest
- settings · function · L138-L145 — fn settings(effort: Option<&str>) -> RetrySettings
- model_not_found_retry_rebuilds_mandatory_thinking_wire_payload · function · L148-L178 — fn model_not_found_retry_rebuilds_mandatory_thinking_wire_payload()
- fable_quota_retry_rebuilds_target_defaults_and_removes_binding · function · L181-L198 — fn fable_quota_retry_rebuilds_target_defaults_and_removes_binding()
- retry_preserves_explicit_budget_and_effort_preferences · function · L201-L212 — fn retry_preserves_explicit_budget_and_effort_preferences()
- always_on_rejection_keeps_bound_summaries_and_retries_only_once · function · L215-L234 — fn always_on_rejection_keeps_bound_summaries_and_retries_only_once()
- legacy_rejection_still_removes_unsupported_reasoning_once · function · L237-L255 — fn legacy_rejection_still_removes_unsupported_reasoning_once()
- opus_55_and_fable_51_request_bound_thinking_and_progress_summaries · function · L258-L275 — fn opus_55_and_fable_51_request_bound_thinking_and_progress_summaries()
- older_models_keep_original_payload_and_headers · function · L278-L295 — fn older_models_keep_original_payload_and_headers()
