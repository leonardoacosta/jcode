# crates/jcode-provider-copilot/src/lib.rs

- COPILOT_API_VERSION · constant · L9-L9 — pub const COPILOT_API_VERSION: &str = "2025-04-01";
- DEFAULT_MODEL · constant · L15-L15 — pub const DEFAULT_MODEL: &str = "claude-sonnet-4.6";
- FALLBACK_MODELS · constant · L17-L37 — pub const FALLBACK_MODELS: &[&str] = &[
- PersistedCatalog · struct · L40-L43 — pub struct PersistedCatalog
- is_known_display_model · function · L45-L47 — pub fn is_known_display_model(model: &str) -> bool
- max_token_parameter_for_model · function · L49-L56 — pub fn max_token_parameter_for_model(model: &str) -> &'static str
- add_max_token_parameter · function · L58-L60 — pub fn add_max_token_parameter(body: &mut Value, model: &str, max_tokens: u32)
- build_messages · function · L67-L221 — pub fn build_messages(system: &str, messages: &[ChatMessage]) -> Vec<Value>
- build_tools · function · L224-L248 — pub fn build_tools(tools: &[ToolDefinition]) -> Vec<Value>
- tests · module · L251-L355 — mod tests
- missing_tool_outputs_remain_independent_in_wire_payload · function · L255-L293 — fn missing_tool_outputs_remain_independent_in_wire_payload()
- swarm_shaped_tool · function · L295-L326 — fn swarm_shaped_tool() -> ToolDefinition
- copilot_tool_serialization_flattens_swarm_top_level_combiners · function · L329-L340 — fn copilot_tool_serialization_flattens_swarm_top_level_combiners()
- copilot_tool_payload_preserves_swarm_properties_and_required_fields · function · L343-L354 — fn copilot_tool_payload_preserves_swarm_properties_and_required_fields()
