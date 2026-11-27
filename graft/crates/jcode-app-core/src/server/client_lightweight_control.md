# crates/jcode-app-core/src/server/client_lightweight_control.rs

- parse_swarm_spawn_mode · function · L39-L60 — pub(super) fn parse_swarm_spawn_mode(
- comm_request_session_id · function · L65-L99 — fn comm_request_session_id(request: &Request) -> Option<&str>
- ensure_lightweight_swarm_member · function · L107-L236 — async fn ensure_lightweight_swarm_member(
- LightweightControlContext · struct · L238-L258 — pub(super) struct LightweightControlContext<'a>
- handle_lightweight_control_request · function · L260-L1081 — pub(super) async fn handle_lightweight_control_request(
- lightweight_swarm_member_tests · module · L1084-L1219 — mod lightweight_swarm_member_tests
- HomeGuard · struct · L1087-L1091 — struct HomeGuard
- drop · function · L1094-L1099 — fn drop(&mut self)
- isolated_home · function · L1102-L1112 — fn isolated_home() -> HomeGuard
- Members · type · L1114-L1114 — type Members = Arc<RwLock<HashMap<String, SwarmMember>>>;
- Swarms · type · L1115-L1115 — type Swarms = Arc<RwLock<HashMap<String, HashSet<String>>>>;
- empty_state · function · L1117-L1123 — fn empty_state() -> (SessionAgents, Members, Swarms)
- persisted_headless_session_is_registered_with_its_working_dir · function · L1129-L1155 — async fn persisted_headless_session_is_registered_with_its_working_dir()
- unknown_session_is_not_registered · function · L1160-L1166 — async fn unknown_session_is_not_registered()
- corrupt_control_record_fails_closed_and_registers_no_member · function · L1169-L1192 — async fn corrupt_control_record_fails_closed_and_registers_no_member()
- stopped_stub_cannot_recreate_ownerless_membership · function · L1195-L1218 — async fn stopped_stub_cannot_recreate_ownerless_membership()
