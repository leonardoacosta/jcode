# crates/jcode-base/src/mobile_sync_direct_count_tests.rs

- legacy_count · function · L4-L18 — fn legacy_count(value: &impl Serialize) -> serde_json::Result<usize>
- Counter · struct · L5-L5 — struct Counter(usize);
- write · function · L7-L10 — fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize>
- flush · function · L11-L13 — fn flush(&mut self) -> std::io::Result<()>
- direct_count_matches_serialized_bytes_and_errors · function · L21-L48 — fn direct_count_matches_serialized_bytes_and_errors()
- FailAfterPrefix · struct · L33-L33 — struct FailAfterPrefix;
- serialize · function · L35-L40 — fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error>
- direct_count_abba · function · L52-L102 — fn direct_count_abba()
