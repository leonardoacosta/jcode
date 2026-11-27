# crates/jcode-base/src/session_telemetry.rs

- SessionTelemetry · struct · L7-L26 — pub struct SessionTelemetry
- path · function · L28-L41 — fn path(session_id: &str) -> Option<std::path::PathBuf>
- save_context · function · L43-L105 — pub fn save_context(
- load_context · function · L107-L124 — pub fn load_context(session: &Session) -> Option<SessionTelemetry>
- matches_session · function · L127-L133 — pub fn matches_session(&self, session: &Session) -> bool
- tests · module · L137-L191 — mod tests
- durable_measurement_roundtrips_and_rejects_changed_identity · function · L141-L182 — fn durable_measurement_roundtrips_and_rejects_changed_identity()
- HomeGuard · struct · L143-L143 — struct HomeGuard(Option<std::ffi::OsString>);
- drop · function · L145-L151 — fn drop(&mut self)
- session_ids_cannot_escape_snapshot_directory · function · L185-L190 — fn session_ids_cannot_escape_snapshot_directory()
