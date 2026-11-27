# crates/jcode-base/src/mobile_sync/wire_payload_tests.rs

- message · function · L5-L16 — fn message(seq: usize, text: String) -> WireMessage
- legacy_chunks · function · L18-L37 — fn legacy_chunks(messages: &[WireMessage]) -> Vec<&[WireMessage]>
- counted_chunk_boundaries_match_allocating_oracle · function · L40-L80 — fn counted_chunk_boundaries_match_allocating_oracle()
- consuming_rgb_conversion_preserves_pixels_and_jpeg_bytes · function · L83-L109 — fn consuming_rgb_conversion_preserves_pixels_and_jpeg_bytes()
- image_fragment_preflight_preserves_original_fragment_boundaries · function · L112-L129 — fn image_fragment_preflight_preserves_original_fragment_boundaries()
- benchmark_wire_payload_allocations · function · L133-L176 — fn benchmark_wire_payload_allocations()
