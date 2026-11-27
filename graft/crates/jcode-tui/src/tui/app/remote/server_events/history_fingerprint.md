# crates/jcode-tui/src/tui/app/remote/server_events/history_fingerprint.rs

- AppliedHistoryFingerprint · struct · L5-L8 — pub(super) struct AppliedHistoryFingerprint
- LAST_APPLIED_HISTORY · constant · L19-L21 — static LAST_APPLIED_HISTORY: std::sync::LazyLock<
- last_applied_history_fingerprint · function · L23-L30 — pub(super) fn last_applied_history_fingerprint(
- record_applied_history_fingerprint · function · L32-L52 — pub(super) fn record_applied_history_fingerprint(
- hash_json_value · function · L56-L88 — pub(super) fn hash_json_value(value: &serde_json::Value, hasher: &mut impl std::hash::Hasher)
- history_payload_fingerprint · function · L97-L128 — pub(super) fn history_payload_fingerprint(messages: &[crate::protocol::HistoryMessage]) -> u64
- should_skip_identical_history_payload · function · L135-L146 — pub(super) fn should_skip_identical_history_payload(
- history_images_match_retained · function · L152-L164 — pub(super) fn history_images_match_retained(
