# crates/jcode-base/src/mobile_sync_incremental_cache_tests.rs

- legacy · function · L3-L13 — fn legacy(session: &Session, progress: &SessionProgress) -> (Vec<WireMessage>, u64)
- text · function · L15-L20 — fn text(text: &str) -> ContentBlock
- fixture · function · L21-L41 — fn fixture() -> Session
- progress · function · L42-L49 — fn progress(session: &Session) -> SessionProgress
- check · function · L50-L52 — fn check(cache: &mut PrefixCache, session: &Session, progress: &SessionProgress)
- prefix_cache_matches_legacy_after_reload_append_edits_and_truncation · function · L55-L98 — fn prefix_cache_matches_legacy_after_reload_append_edits_and_truncation()
- external_markdown_images_bypass_prefix_cache_including_missing_files · function · L101-L124 — fn external_markdown_images_bypass_prefix_cache_including_missing_files()
- every_wire_metadata_and_tool_field_invalidates_the_prefix · function · L127-L165 — fn every_wire_metadata_and_tool_field_invalidates_the_prefix()
- source_validation_distinguishes_json_spelling_and_inline_images · function · L168-L193 — fn source_validation_distinguishes_json_spelling_and_inline_images()
- failed_upload_progress_never_reuses_an_unuploaded_prefix · function · L196-L211 — fn failed_upload_progress_never_reuses_an_unuploaded_prefix()
- real_inline_image_replacement_invalidates_cached_prefix · function · L214-L239 — fn real_inline_image_replacement_invalidates_cached_prefix()
- bounded_validation_preserves_truncated_results_and_rejects_expensive_sources · function · L242-L277 — fn bounded_validation_preserves_truncated_results_and_rejects_expensive_sources()
- cache_retains_only_bounded_session_metadata · function · L280-L293 — fn cache_retains_only_bounded_session_metadata()
- benchmark_large_truncated_prefix · function · L297-L352 — fn benchmark_large_truncated_prefix()
- benchmark_validated_mobile_prefix · function · L356-L403 — fn benchmark_validated_mobile_prefix()
