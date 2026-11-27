# crates/jcode-sdk/src/diagnostics.rs

- Stage · enum · L18-L29 — pub enum Stage
- doing · function · L33-L41 — pub fn doing(self) -> &'static str
- SocketState · enum · L47-L50 — pub enum SocketState
- probe · function · L54-L59 — pub fn probe(path: &Path) -> Self
- describe_disconnect · function · L63-L108 — pub fn describe_disconnect(
- explain · function · L116-L130 — pub fn explain(message: &str) -> String
- OFFLINE · constant · L118-L125 — const OFFLINE: [&str; 6] = [
- human_duration · function · L133-L141 — pub fn human_duration(d: Duration) -> String
- tests · module · L144-L205 — mod tests
- an_offline_failure_is_explained_in_the_users_terms · function · L152-L164 — fn an_offline_failure_is_explained_in_the_users_terms()
- an_unrecognised_failure_is_passed_through · function · L169-L174 — fn an_unrecognised_failure_is_passed_through()
- a_live_socket_after_a_drop_points_at_a_replacement_bridge · function · L180-L190 — fn a_live_socket_after_a_drop_points_at_a_replacement_bridge()
- a_dead_socket_after_a_drop_points_at_an_exited_bridge · function · L195-L204 — fn a_dead_socket_after_a_drop_points_at_an_exited_bridge()
