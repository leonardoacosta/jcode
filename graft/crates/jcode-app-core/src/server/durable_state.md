# crates/jcode-app-core/src/server/durable_state.rs

- now_unix_ms · function · L7-L12 — pub(super) fn now_unix_ms() -> u64
- sanitize_session_id · function · L14-L25 — pub(super) fn sanitize_session_id(session_id: &str) -> String
- hashed_request_key · function · L27-L39 — pub(super) fn hashed_request_key(session_id: &str, action: &str, components: &[String]) -> String
- state_dir · function · L41-L43 — pub(super) fn state_dir(dir_name: &str) -> PathBuf
- state_path · function · L45-L47 — pub(super) fn state_path(dir_name: &str, key: &str) -> PathBuf
- load_json_state · function · L49-L61 — pub(super) fn load_json_state<T, F>(dir_name: &str, key: &str, is_stale: F) -> Option<T>
- save_json_state · function · L63-L71 — pub(super) fn save_json_state<T>(dir_name: &str, key: &str, state: &T, label: &str)
- elapsed_exceeds · function · L73-L75 — pub(super) fn elapsed_exceeds(created_at_unix_ms: u64, ttl: Duration) -> bool
