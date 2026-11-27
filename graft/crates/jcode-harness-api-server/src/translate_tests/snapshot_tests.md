# crates/jcode-harness-api-server/src/translate_tests/snapshot_tests.rs

- snapshot_dir · function · L5-L7 — fn snapshot_dir() -> PathBuf
- fixture · function · L9-L12 — fn fixture(name: &str) -> Vec<Value>
- assert_snapshot · function · L15-L40 — pub(super) fn assert_snapshot(name: &str, actual: &Value)
- redact · function · L44-L59 — fn redact(value: &mut Value)
- attached_state · function · L61-L67 — fn attached_state() -> BridgeState
- translate_request_frames_match_snapshot · function · L70-L92 — fn translate_request_frames_match_snapshot()
- translate_event_frames_match_snapshot · function · L95-L106 — fn translate_event_frames_match_snapshot()
- translate_snapshot_drift_fails_with_diff · function · L110-L115 — fn translate_snapshot_drift_fails_with_diff()
