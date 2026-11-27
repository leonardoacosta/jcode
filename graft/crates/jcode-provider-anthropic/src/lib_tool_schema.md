# crates/jcode-provider-anthropic/src/lib_tool_schema.rs

- anthropic_input_schema · function · L14-L16 — pub(super) fn anthropic_input_schema(schema: &Value) -> Value
- format_tools · function · L25-L41 — pub fn format_tools(tools: &[ToolDefinition], is_oauth: bool, cache_ttl_1h: bool) -> Vec<ApiTool>
- finish_tool_list · function · L51-L65 — pub(super) fn finish_tool_list(tools: Vec<ApiTool>, cache_ttl_1h: bool) -> Vec<ApiTool>
