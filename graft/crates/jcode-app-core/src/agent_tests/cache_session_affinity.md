# crates/jcode-app-core/src/agent_tests/cache_session_affinity.rs

- SessionAffinityProvider · struct · L5-L8 — struct SessionAffinityProvider
- set_session_id · function · L12-L14 — fn set_session_id(&self, session_id: &str)
- prewarm · function · L16-L24 — async fn prewarm(&self, _tools: &[ToolDefinition], _system: &str)
- complete · function · L26-L34 — async fn complete(
- name · function · L36-L38 — fn name(&self) -> &'static str
- fork · function · L40-L42 — fn fork(&self) -> Arc<dyn Provider>
- cache_session_affinity_binds_new_and_restored_agents_before_prewarm · function · L46-L83 — async fn cache_session_affinity_binds_new_and_restored_agents_before_prewarm()
