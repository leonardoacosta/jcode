# crates/jcode-base/tests/mcp_stdio_collision_integration.rs

- CHILD_MARKER · constant · L16-L16 — const CHILD_MARKER: &str = "JCODE_MCP_STDIO_TEST_CHILD";
- PYTHON · constant · L17-L17 — const PYTHON: &str = "JCODE_MCP_STDIO_TEST_PYTHON";
- TEST_NAME · constant · L18-L18 — const TEST_NAME: &str = "real_stdio_collision_aliases_preserve_original_targets";
- SERVERS · constant · L19-L22 — const SERVERS: [(&str, [&str; 2]); 2] = [
- SERVER · constant · L27-L55 — const SERVER: &str = r#"
- real_stdio_collision_aliases_preserve_original_targets · function · L58-L116 — fn real_stdio_collision_aliases_preserve_original_targets()
- config · function · L118-L136 — fn config() -> McpConfig
- cached_definitions · function · L138-L156 — fn cached_definitions() -> Vec<(String, McpToolDef)>
- Proxies · type · L158-L158 — type Proxies = Vec<(String, Arc<dyn Tool>)>;
- Targets · type · L159-L159 — type Targets = BTreeMap<(String, String), String>;
- call · function · L161-L180 — async fn call(tool: &Arc<dyn Tool>) -> Value
- targets · function · L182-L217 — async fn targets(proxies: &Proxies) -> Targets
- exercise_proxies · function · L219-L293 — async fn exercise_proxies()
