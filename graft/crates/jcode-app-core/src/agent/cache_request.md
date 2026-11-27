# crates/jcode-app-core/src/agent/cache_request.rs

- stable_hash_str · function · L4-L8 — pub(super) fn stable_hash_str(value: &str) -> u64
- encode_json · function · L12-L14 — fn encode_json<T: serde::Serialize + ?Sized>(value: &T) -> String
- stable_hash_json · function · L17-L19 — fn stable_hash_json<T: serde::Serialize + ?Sized>(value: &T) -> u64
- stable_json_len · function · L22-L24 — fn stable_json_len<T: serde::Serialize + ?Sized>(value: &T) -> usize
- messages_json_len · function · L29-L35 — fn messages_json_len(messages: &[Message]) -> usize
- json_hash_and_len · function · L37-L40 — fn json_hash_and_len<T: serde::Serialize + ?Sized>(value: &T) -> (u64, usize)
- projected_message_hashes · function · L45-L61 — fn projected_message_hashes(messages: &[Message]) -> (u64, Vec<u64>)
- message_hashes · function · L64-L71 — fn message_hashes(messages: &[Message]) -> Vec<u64>
- kv_cache_request_event · function · L73-L106 — pub(super) fn kv_cache_request_event(
- tests · module · L110-L110 — mod tests;
