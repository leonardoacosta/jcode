# crates/jcode-provider-openrouter-runtime/src/request_headers.rs

- KIMI_CODING_USER_AGENT · constant · L4-L4 — pub(crate) const KIMI_CODING_USER_AGENT: &str = "claude-cli/1.0.0";
- KIMI_CODING_X_APP · constant · L5-L5 — pub(crate) const KIMI_CODING_X_APP: &str = "cli";
- is_kimi_coding_api_base · function · L7-L13 — pub(crate) fn is_kimi_coding_api_base(api_base: &str) -> bool
- is_coding_agent_api_base · function · L15-L25 — pub(crate) fn is_coding_agent_api_base(api_base: &str) -> bool
- is_kimi_model_name · function · L27-L29 — pub(crate) fn is_kimi_model_name(model: &str) -> bool
- should_send_kimi_coding_agent_headers · function · L31-L33 — pub(crate) fn should_send_kimi_coding_agent_headers(api_base: &str, model: Option<&str>) -> bool
- apply_kimi_coding_agent_headers · function · L35-L46 — pub(crate) fn apply_kimi_coding_agent_headers(
- is_opencode_api_base · function · L49-L57 — pub(crate) fn is_opencode_api_base(api_base: &str) -> bool
- new_conversation_id · function · L59-L61 — pub(crate) fn new_conversation_id() -> String
- apply_opencode_session_header · function · L65-L75 — pub(crate) fn apply_opencode_session_header(
- OPENCODE_SESSION_HEADER · constant · L77-L77 — pub(crate) const OPENCODE_SESSION_HEADER: &str = "x-opencode-session";
