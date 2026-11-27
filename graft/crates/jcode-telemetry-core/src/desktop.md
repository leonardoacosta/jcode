# crates/jcode-telemetry-core/src/desktop.rs

- ACTIVE_RECORDED_FILE · constant · L13-L13 — const ACTIVE_RECORDED_FILE: &str = "desktop_active_recorded";
- VERSION_RECORDED_FILE · constant · L14-L14 — const VERSION_RECORDED_FILE: &str = "desktop_version_recorded";
- MAX_TOKEN_LEN · constant · L15-L15 — const MAX_TOKEN_LEN: usize = 64;
- read_state · function · L17-L22 — fn read_state(name: &str) -> Option<String>
- write_state · function · L24-L28 — fn write_state(name: &str, value: &str)
- sanitize_token · function · L33-L55 — pub(super) fn sanitize_token(value: &str) -> String
- sanitize_version · function · L58-L65 — fn sanitize_version(value: &str) -> String
- base_payload · function · L67-L82 — fn base_payload(id: String, event: &str, version: &str) -> Value
- record_desktop_active · function · L87-L92 — pub fn record_desktop_active(desktop_version: &str)
- record_desktop_active_on · function · L94-L126 — pub(super) fn record_desktop_active_on(desktop_version: &str, utc_date: &str)
- record_desktop_update · function · L131-L153 — pub fn record_desktop_update(
