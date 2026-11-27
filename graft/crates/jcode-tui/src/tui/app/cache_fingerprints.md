# crates/jcode-tui/src/tui/app/cache_fingerprints.rs

- stable_hash_str · function · L3-L7 — pub(super) fn stable_hash_str(value: &str) -> u64
- stable_hash_json · function · L9-L19 — pub(super) fn stable_hash_json<T: serde::Serialize + ?Sized>(value: &T) -> u64
- stable_json_len · function · L21-L31 — pub(super) fn stable_json_len<T: serde::Serialize + ?Sized>(value: &T) -> usize
- message_hashes · function · L33-L38 — pub(super) fn message_hashes(messages: &[Message]) -> Vec<u64>
- ratio_pct · function · L40-L48 — pub(super) fn ratio_pct(numerator: u64, denominator: u64) -> u8
- tests · module · L52-L52 — mod tests;
