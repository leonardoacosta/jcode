# crates/jcode-app-core/src/agent_tests/mcp_remote_cache.rs

- RemotePeer · struct · L9-L15 — struct RemotePeer
- drop · function · L18-L20 — fn drop(&mut self)
- new · function · L24-L59 — async fn new() -> Self
- wait_for_initialize · function · L61-L67 — async fn wait_for_initialize(&self)
- serve_request · function · L70-L143 — async fn serve_request(
- context · function · L145-L155 — fn context(agent: &Agent) -> crate::tool::ToolContext
- wait_for_tool · function · L157-L168 — async fn wait_for_tool(registry: &Registry, name: &str)
- cache_tools_signature · function · L170-L181 — fn cache_tools_signature(tools: &[ToolDefinition]) -> (u64, usize, usize)
- remote_deferred_tool_changes_do_not_change_cache_telemetry · function · L184-L225 — fn remote_deferred_tool_changes_do_not_change_cache_telemetry()
- remote_startup_does_not_wait_and_publishes_fast_tools_before_slow_peer · function · L228-L232 — async fn remote_startup_does_not_wait_and_publishes_fast_tools_before_slow_peer()
- remote_startup_case · function · L234-L387 — async fn remote_startup_case(shared: bool)
- remote_empty_catalog_replaces_obsolete_cached_schemas · function · L390-L454 — async fn remote_empty_catalog_replaces_obsolete_cached_schemas()
- remote_connect_disconnect_reconnect_preserves_prompt_and_cache_baselines · function · L457-L622 — async fn remote_connect_disconnect_reconnect_preserves_prompt_and_cache_baselines()
