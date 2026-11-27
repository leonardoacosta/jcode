# crates/jcode-tui/src/tui/ui_tests/compact_agent_dms.rs

- assert_dm_rebuild_matches_full · function · L6-L61 — fn assert_dm_rebuild_matches_full(
- dm · function · L63-L65 — fn dm(sender: &str, body: &str) -> DisplayMessage
- token_rows · function · L67-L73 — fn token_rows(body: &PreparedMessages) -> Vec<&str>
- compact_dm_await_started_is_hidden_but_finished_result_stays_visible · function · L76-L103 — fn compact_dm_await_started_is_hidden_but_finished_result_stays_visible()
- compact_dm_single_and_group_have_only_one_total_and_badge · function · L106-L148 — fn compact_dm_single_and_group_have_only_one_total_and_badge()
- compact_dm_group_chrome_stays_neutral_for_large_total · function · L151-L166 — fn compact_dm_group_chrome_stays_neutral_for_large_total()
- compact_dm_prepend_merges_with_existing_first_dm · function · L169-L188 — fn compact_dm_prepend_merges_with_existing_first_dm()
- compact_dm_consecutive_append_matches_full_prepare · function · L191-L215 — fn compact_dm_consecutive_append_matches_full_prepare()
- compact_dm_body_edit_matches_full_prepare · function · L218-L251 — fn compact_dm_body_edit_matches_full_prepare()
- compact_dm_assistant_breaks_group_matches_full_prepare · function · L254-L270 — fn compact_dm_assistant_breaks_group_matches_full_prepare()
- compact_dm_expanded_message_excluded_from_group_matches_full_prepare · function · L273-L301 — fn compact_dm_expanded_message_excluded_from_group_matches_full_prepare()
- compact_dm_narrow_unicode_rows_match_full_prepare_without_wrapping · function · L304-L329 — fn compact_dm_narrow_unicode_rows_match_full_prepare_without_wrapping()
