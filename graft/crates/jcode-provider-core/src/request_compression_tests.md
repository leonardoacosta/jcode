# crates/jcode-provider-core/src/request_compression_tests.rs

- UPLINK_TEST_LOCK · constant · L5-L5 — static UPLINK_TEST_LOCK: Mutex<()> = Mutex::new(());
- jsonish · function · L7-L10 — fn jsonish(len: usize) -> Vec<u8>
- deterministic_noise · function · L12-L22 — fn deterministic_noise(len: usize) -> Vec<u8>
- large_gzip_bodies_roundtrip_and_selected_bodies_have_bounded_capacity · function · L25-L54 — fn large_gzip_bodies_roundtrip_and_selected_bodies_have_bounded_capacity()
- gzip_allocation_benchmark · function · L64-L164 — fn gzip_allocation_benchmark()
- encode · function · L65-L88 — fn encode(raw: &[u8], policy: usize) -> Vec<u8>
- json_uploads_match_original_wire_bytes_and_headers · function · L169-L262 — async fn json_uploads_match_original_wire_bytes_and_headers()
- parses_modes · function · L265-L276 — fn parses_modes()
- only_verified_endpoints_are_eligible · function · L279-L307 — fn only_verified_endpoints_are_eligible()
- break_even_matches_measurements · function · L310-L318 — fn break_even_matches_measurements()
- decide_respects_mode_size_and_endpoint · function · L321-L352 — fn decide_respects_mode_size_and_endpoint()
- incompressible_body_stays_raw_in_auto · function · L355-L376 — fn incompressible_body_stays_raw_in_auto()
- sampler_records_rate_after_skip_window · function · L379-L397 — fn sampler_records_rate_after_skip_window()
- ewma_moves_toward_new_samples · function · L400-L407 — fn ewma_moves_toward_new_samples()
- endpoint_encodings_match_live_probes · function · L410-L432 — fn endpoint_encodings_match_live_probes()
- zstd_roundtrips_and_beats_gzip_ratio · function · L435-L474 — fn zstd_roundtrips_and_beats_gzip_ratio()
- zstd_respects_openai_expansion_limit · function · L477-L515 — fn zstd_respects_openai_expansion_limit()
