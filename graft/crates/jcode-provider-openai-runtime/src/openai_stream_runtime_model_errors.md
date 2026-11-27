# crates/jcode-provider-openai-runtime/src/openai_stream_runtime_model_errors.rs

- should_refresh_token · function · L3-L22 — pub(super) fn should_refresh_token(status: StatusCode, body: &str) -> bool
- format_http_error · function · L26-L38 — pub(super) fn format_http_error(status: StatusCode, body: &str) -> String
- with_failure_remedy · function · L42-L67 — pub(super) fn with_failure_remedy(event: StreamEvent) -> StreamEvent
- claim_auth_refresh · function · L71-L77 — pub(super) fn claim_auth_refresh(status: StatusCode, body: &str, attempted: &mut bool) -> bool
- maybe_record_runtime_model_unavailable_from_stream_error · function · L79-L94 — pub(super) fn maybe_record_runtime_model_unavailable_from_stream_error(
- classify_unavailable_model_error · function · L96-L134 — pub(super) fn classify_unavailable_model_error(status: StatusCode, body: &str) -> Option<String>
- is_retryable_error · function · L137-L159 — pub(in super::super) fn is_retryable_error(error_str: &str) -> bool
- is_openai_transient_message · function · L164-L184 — fn is_openai_transient_message(error_str: &str) -> bool
