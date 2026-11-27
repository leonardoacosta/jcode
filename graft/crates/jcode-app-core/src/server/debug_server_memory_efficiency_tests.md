# crates/jcode-app-core/src/server/debug_server_memory_efficiency_tests.rs

- PrivateHome · struct · L4-L8 — struct PrivateHome
- new · function · L11-L22 — fn new() -> Self
- drop · function · L26-L36 — fn drop(&mut self)
- NoRequests · struct · L39-L39 — struct NoRequests;
- complete · function · L43-L51 — async fn complete(
- name · function · L53-L55 — fn name(&self) -> &str
- fork · function · L57-L59 — fn fork(&self) -> Arc<dyn crate::provider::Provider>
- populated_agent · function · L62-L87 — fn populated_agent(index: usize) -> Agent
- session_only_profile_matches_full_agent_profile_and_removes_repeated_process_samples · function · L90-L151 — fn session_only_profile_matches_full_agent_profile_and_removes_repeated_process_samples()
- memory_payload · function · L153-L180 — async fn memory_payload(sessions: &SessionAgents) -> serde_json::Value
- server_memory_preserves_session_output_and_skips_busy_agents · function · L184-L223 — async fn server_memory_preserves_session_output_and_skips_busy_agents()
