# crates/jcode-app-core/src/server/comm_control_tests.rs

- RuntimeEnvGuard · struct · L25-L28 — struct RuntimeEnvGuard
- new · function · L31-L43 — fn new() -> (Self, tempfile::TempDir)
- drop · function · L47-L53 — fn drop(&mut self)
- member · function · L56-L80 — fn member(session_id: &str, swarm_id: &str, status: &str) -> SwarmMember
- owned_member · function · L86-L90 — fn owned_member(session_id: &str, swarm_id: &str, status: &str, owner: &str) -> SwarmMember
- plan_item · function · L92-L103 — fn plan_item(id: &str, status: &str, priority: &str, blocked_by: &[&str]) -> PlanItem
- swarm_event · function · L105-L115 — fn swarm_event(session_id: &str, swarm_id: &str, event: SwarmEventType) -> SwarmEvent
- TestProvider · struct · L118-L118 — struct TestProvider;
- complete · function · L122-L132 — async fn complete(
- name · function · L134-L136 — fn name(&self) -> &str
- fork · function · L138-L140 — fn fork(&self) -> Arc<dyn Provider>
- test_agent · function · L143-L147 — async fn test_agent() -> Arc<Mutex<Agent>>
