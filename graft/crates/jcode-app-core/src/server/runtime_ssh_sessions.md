# crates/jcode-app-core/src/server/runtime_ssh_sessions.rs

- handles · function · L6-L11 — pub(super) fn handles(command: &str) -> bool
- summary · function · L13-L18 — fn summary(session: &Session) -> Value
- list_sessions · function · L20-L108 — fn list_sessions(current: Value, command: &str, args: &[String]) -> Result<(Value, String)>
- execute · function · L110-L151 — pub(super) async fn execute(
- tests · module · L154-L175 — mod tests
- session_runtime_names_are_exact · function · L157-L164 — fn session_runtime_names_are_exact()
- current_session_summary_preserves_remote_identity · function · L166-L174 — fn current_session_summary_preserves_remote_identity()
