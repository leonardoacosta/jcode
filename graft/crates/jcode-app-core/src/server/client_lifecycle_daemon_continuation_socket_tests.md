# crates/jcode-app-core/src/server/client_lifecycle_daemon_continuation_socket_tests.rs

- ContinuationProvider · struct · L7-L11 — struct ContinuationProvider
- complete · function · L15-L33 — async fn complete(
- name · function · L35-L37 — fn name(&self) -> &str
- supports_compaction · function · L39-L41 — fn supports_compaction(&self) -> bool
- fork · function · L43-L45 — fn fork(&self) -> Arc<dyn Provider>
- send · function · L48-L53 — async fn send(writer: &mut crate::transport::WriteHalf, value: serde_json::Value)
- receive · function · L55-L63 — async fn receive(reader: &mut BufReader<crate::transport::ReadHalf>) -> ServerEvent
- ordinary_socket_auto_poke_is_bounded_and_has_one_final_done · function · L66-L68 — async fn ordinary_socket_auto_poke_is_bounded_and_has_one_final_done()
- second_socket_can_disable_auto_poke_during_a_continuation · function · L71-L73 — async fn second_socket_can_disable_auto_poke_during_a_continuation()
- continuation_socket_workflow · function · L75-L252 — async fn continuation_socket_workflow(disable_from_observer: bool)
