# crates/jcode-provider-openai-runtime/src/native_web_search.rs

- FIRST_PARTY_API_BASE · constant · L17-L17 — const FIRST_PARTY_API_BASE: &str = "https://api.openai.com/v1";
- model_supports_web_search · function · L23-L25 — pub(crate) fn model_supports_web_search(model_id: &str) -> bool
- hosted_tool · function · L28-L40 — pub(crate) fn hosted_tool(config: &WebSearchConfig) -> Value
- first_party_backend · function · L44-L48 — pub(crate) fn first_party_backend(is_chatgpt_mode: bool) -> bool
- session_offers_websearch · function · L54-L56 — pub(crate) fn session_offers_websearch(tools: &[ToolDefinition]) -> bool
- hosted_tools_with_config · function · L60-L75 — pub(crate) fn hosted_tools_with_config(
- hosted_tools_for_request · function · L77-L88 — pub(crate) fn hosted_tools_for_request(
- without_local_websearch · function · L93-L107 — pub(crate) fn without_local_websearch<'a>(
- tests · module · L110-L217 — mod tests
- native_config · function · L115-L120 — fn native_config() -> WebSearchConfig
- hosted_tool_only_when_native_and_not_codex · function · L123-L146 — fn hosted_tool_only_when_native_and_not_codex()
- hosted_tool_respects_session_policy_and_gateway · function · L149-L158 — fn hosted_tool_respects_session_policy_and_gateway()
- sdk_custom_websearch_is_never_replaced_by_hosted_tool · function · L161-L172 — fn sdk_custom_websearch_is_never_replaced_by_hosted_tool()
- web_search_calls_downgrade_to_text · function · L175-L190 — fn web_search_calls_downgrade_to_text()
- allowed_domains_become_filters · function · L193-L200 — fn allowed_domains_become_filters()
- local_websearch_dropped_only_with_hosted_tool · function · L203-L216 — fn local_websearch_dropped_only_with_hosted_tool()
