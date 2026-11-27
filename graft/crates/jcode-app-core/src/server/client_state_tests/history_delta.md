# crates/jcode-app-core/src/server/client_state_tests/history_delta.rs

- delta_test_messages · function · L1-L13 — fn delta_test_messages() -> Vec<crate::protocol::HistoryMessage>
- delta_test_base · function · L15-L29 — fn delta_test_base(
- history_base_match_sends_only_the_tail · function · L32-L39 — fn history_base_match_sends_only_the_tail()
- history_base_full_match_sends_empty_tail · function · L42-L48 — fn history_base_full_match_sends_empty_tail()
- history_base_mismatch_or_overlong_sends_full_transcript · function · L51-L67 — fn history_base_mismatch_or_overlong_sends_full_transcript()
- history_images_base_omits_only_matching_image_sets · function · L70-L96 — fn history_images_base_omits_only_matching_image_sets()
- recent_history_bounds_seven_thousand_message_bootstrap · function · L99-L127 — fn recent_history_bounds_seven_thousand_message_bootstrap()
- recent_history_preserves_archive_prompt_offset_and_rebases_images · function · L130-L173 — fn recent_history_preserves_archive_prompt_offset_and_rebases_images()
- recent_history_never_mixes_with_validated_or_stale_prefix · function · L176-L196 — fn recent_history_never_mixes_with_validated_or_stale_prefix()
- recent_history_never_truncates_one_large_message · function · L199-L205 — fn recent_history_never_truncates_one_large_message()
- recent_history_scopes_tool_images_and_rebases_append_boundaries · function · L208-L234 — fn recent_history_scopes_tool_images_and_rebases_append_boundaries()
- recent_history_live_and_busy_bootstrap_and_expansion · function · L241-L369 — async fn recent_history_live_and_busy_bootstrap_and_expansion()
