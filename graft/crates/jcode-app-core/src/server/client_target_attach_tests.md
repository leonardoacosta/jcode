# crates/jcode-app-core/src/server/client_target_attach_tests.rs

- NoRequests · struct · L7-L7 — struct NoRequests;
- complete · function · L10-L18 — async fn complete(
- name · function · L19-L21 — fn name(&self) -> &str
- fork · function · L22-L24 — fn fork(&self) -> Arc<dyn Provider>
- Home · struct · L27-L30 — struct Home
- new · function · L32-L37 — fn new() -> Self
- drop · function · L40-L46 — fn drop(&mut self)
- subscribe · function · L49-L69 — fn subscribe(target: &str) -> Request
- live_agent · function · L71-L79 — async fn live_agent(id: &str, root: &str) -> Arc<Mutex<Agent>>
- target_subscribe_uses_live_unsaved_root_without_changing_it · function · L82-L104 — async fn target_subscribe_uses_live_unsaved_root_without_changing_it()
- target_subscribe_uses_persisted_root_when_no_live_agent_exists · function · L107-L125 — async fn target_subscribe_uses_persisted_root_when_no_live_agent_exists()
- target_subscribe_live_root_wins_over_stale_persisted_root · function · L128-L150 — async fn target_subscribe_live_root_wins_over_stale_persisted_root()
- target_subscribe_busy_live_agent_uses_member_root_without_waiting · function · L153-L201 — async fn target_subscribe_busy_live_agent_uses_member_root_without_waiting()
- target_subscribe_unknown_target_never_uses_process_working_dir · function · L204-L223 — async fn target_subscribe_unknown_target_never_uses_process_working_dir()
- target_subscribe_reports_corrupt_persisted_state_without_using_process_cwd · function · L226-L246 — async fn target_subscribe_reports_corrupt_persisted_state_without_using_process_cwd()
- target_subscribe_preserves_explicit_directory_and_its_validation · function · L249-L276 — async fn target_subscribe_preserves_explicit_directory_and_its_validation()
