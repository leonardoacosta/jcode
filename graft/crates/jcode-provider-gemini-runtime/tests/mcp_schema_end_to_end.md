# crates/jcode-provider-gemini-runtime/tests/mcp_schema_end_to_end.rs

- PLAYWRIGHT_TOOLS_LIST · constant · L17-L37 — const PLAYWRIGHT_TOOLS_LIST: &str = r#"
- playwright_tool_definitions · function · L39-L56 — fn playwright_tool_definitions() -> Vec<ToolDefinition>
- ToolsList · struct · L41-L43 — struct ToolsList
- contains_key · function · L60-L66 — fn contains_key(value: &serde_json::Value, key: &str) -> bool
- playwright_schema_reaches_gemini_without_the_keyword_that_caused_issue_754 · function · L69-L95 — fn playwright_schema_reaches_gemini_without_the_keyword_that_caused_issue_754()
- playwright_schema_reaches_every_antigravity_route_cleanly · function · L98-L116 — fn playwright_schema_reaches_every_antigravity_route_cleanly()
- playwright_schema_reaches_openai_without_an_unsupported_construct · function · L119-L132 — fn playwright_schema_reaches_openai_without_an_unsupported_construct()
- the_normalized_playwright_schema_conforms_to_every_dialect · function · L138-L167 — fn the_normalized_playwright_schema_conforms_to_every_dialect()
- recovery_triggers_on_the_error_string_the_runtime_really_builds · function · L178-L195 — fn recovery_triggers_on_the_error_string_the_runtime_really_builds()
- GEMINI_400_BODY · constant · L200-L200 — const GEMINI_400_BODY: &str = r#"{"error":{"code":400,"message":"Invalid JSON payload received. Unknown name \"propertyNames\" at 'request.tools[0].function_declarations[32].parameters.properties[0].value': Cannot find field.","status":"INVALID_ARGUMENT"}}"#;
- exotic_mcp_tool_definitions · function · L205-L244 — fn exotic_mcp_tool_definitions() -> Vec<ToolDefinition>
- assert_provider_safe · function · L247-L256 — fn assert_provider_safe(provider: &str, name: &str)
- wire_names · function · L258-L269 — fn wire_names(wire: &serde_json::Value, out: &mut Vec<String>)
- exotic_mcp_tool_names_are_accepted_by_every_provider · function · L272-L311 — fn exotic_mcp_tool_names_are_accepted_by_every_provider()
