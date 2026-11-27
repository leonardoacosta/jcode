# crates/jcode-app-core/src/agent/cache_request/tests.rs

- legacy_event · function · L3-L28 — fn legacy_event(
- tools · function · L30-L40 — fn tools() -> Vec<ToolDefinition>
- fused_request_matches_all_legacy_fields · function · L43-L65 — fn fused_request_matches_all_legacy_fields()
- projected_stream_hash_matches_string_hash_across_chunk_boundaries · function · L68-L86 — fn projected_stream_hash_matches_string_hash_across_chunk_boundaries()
- fused_json_failure_preserves_empty_hash_and_zero_length · function · L89-L97 — fn fused_json_failure_preserves_empty_hash_and_zero_length()
- Fail · struct · L90-L90 — struct Fail;
- serialize · function · L92-L94 — fn serialize<S: serde::Serializer>(&self, _: S) -> Result<S::Ok, S::Error>
- cached_message_lengths_match_serialization_after_metadata_and_content_edits · function · L100-L123 — fn cached_message_lengths_match_serialization_after_metadata_and_content_edits()
- benchmark_cached_request_size_accounting · function · L127-L155 — fn benchmark_cached_request_size_accounting()
- benchmark_request_diagnostics · function · L159-L201 — fn benchmark_request_diagnostics()
