# crates/jcode-telemetry-core/src/categories.rs

- TelemetryCategory · enum · L11-L24 — pub enum TelemetryCategory
- ALL · constant · L28-L35 — pub const ALL: [TelemetryCategory; 6] = [
- key · function · L38-L47 — pub fn key(self) -> &'static str
- from_key · function · L49-L59 — pub fn from_key(key: &str) -> Option<Self>
- label · function · L61-L70 — pub fn label(self) -> &'static str
- caption · function · L72-L83 — pub fn caption(self) -> &'static str
- opt_out_marker · function · L85-L89 — fn opt_out_marker(self) -> Option<std::path::PathBuf>
- enabled · function · L92-L100 — pub fn enabled(self) -> bool
- persist_raw · function · L103-L128 — fn persist_raw(self, enabled: bool)
- for_event · function · L132-L143 — pub fn for_event(event: &str) -> Self
- category_settings · function · L147-L149 — pub fn category_settings() -> [bool; 6]
- set_category_settings · function · L153-L165 — pub fn set_category_settings(settings: [bool; 6])
- payload_allowed · function · L168-L175 — pub(crate) fn payload_allowed(payload: &serde_json::Value) -> bool
