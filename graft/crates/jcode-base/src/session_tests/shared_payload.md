# crates/jcode-base/src/session_tests/shared_payload.rs

- request_snapshot_allocator_probe · function · L6-L47 — fn request_snapshot_allocator_probe()
- live · function · L7-L12 — fn live() -> u64
- fixture · function · L49-L66 — fn fixture() -> Session
- assert_canonical_sharing · function · L68-L80 — fn assert_canonical_sharing(session: &mut Session)
- provider_cache_and_multiple_requests_share_canonical_bodies · function · L83-L108 — fn provider_cache_and_multiple_requests_share_canonical_bodies()
- request_mutation_detaches_only_changed_message_and_preserves_transcript · function · L111-L141 — fn request_mutation_detaches_only_changed_message_and_preserves_transcript()
- append_replace_and_truncate_refresh_cache_without_changing_inflight_snapshot · function · L144-L179 — fn append_replace_and_truncate_refresh_cache_without_changing_inflight_snapshot()
- shared_transcript_content_preserves_legacy_json_arrays · function · L182-L205 — fn shared_transcript_content_preserves_legacy_json_arrays() -> anyhow::Result<()>
- session_image_recovery_detaches_changed_body_and_invalidates_provider_cache · function · L208-L236 — fn session_image_recovery_detaches_changed_body_and_invalidates_provider_cache()
