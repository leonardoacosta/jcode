# crates/jcode-base/src/mobile_sync_byte_count_tests.rs

- fixture · function · L3-L18 — fn fixture(text: &str, copies: usize) -> (Vec<WireToolCall>, Vec<WireToolResult>)
- exact_json_bytes_match_legacy_for_escaping_unicode_and_large_payloads · function · L21-L37 — fn exact_json_bytes_match_legacy_for_escaping_unicode_and_large_payloads()
- byte_limit_is_inclusive_and_matches_legacy · function · L40-L79 — fn byte_limit_is_inclusive_and_matches_legacy()
- prose_keeps_utf16_boundary_semantics · function · L82-L101 — fn prose_keeps_utf16_boundary_semantics()
- serializer_errors_remain_errors · function · L104-L115 — fn serializer_errors_remain_errors()
- Failure · struct · L105-L105 — struct Failure;
- serialize · function · L107-L109 — fn serialize<S: serde::Serializer>(&self, _: S) -> Result<S::Ok, S::Error>
- paired_byte_count_benchmark · function · L119-L173 — fn paired_byte_count_benchmark()
