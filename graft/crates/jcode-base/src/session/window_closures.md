# crates/jcode-base/src/session/window_closures.rs

- Closure · struct · L12-L15 — struct Closure
- marker_path · function · L17-L25 — fn marker_path(dir: &Path, session_id: &str) -> Option<PathBuf>
- directory · function · L27-L29 — fn directory() -> PathBuf
- record_window_closed · function · L32-L48 — pub fn record_window_closed(session_id: &str)
- clear_window_closed · function · L51-L58 — pub fn clear_window_closed(session_id: &str)
- snapshot · function · L60-L142 — fn snapshot(dir: &Path, now: DateTime<Utc>, window: Duration) -> HashMap<String, DateTime<Utc>>
- recent_window_closures · function · L145-L147 — pub fn recent_window_closures(window: Duration) -> HashMap<String, DateTime<Utc>>
- tests · module · L150-L188 — mod tests
- closure_window_uses_real_event_time_and_rejects_future_or_disabled · function · L154-L180 — fn closure_window_uses_real_event_time_and_rejects_future_or_disabled()
- identifiers_cannot_escape_runtime_directory · function · L183-L187 — fn identifiers_cannot_escape_runtime_directory()
