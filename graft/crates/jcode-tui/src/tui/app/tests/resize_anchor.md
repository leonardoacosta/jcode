# crates/jcode-tui/src/tui/app/tests/resize_anchor.rs

- anchored_scroll_test_app · function · L10-L23 — fn anchored_scroll_test_app() -> crate::tui::app::App
- anchored_chat_area · function · L26-L36 — fn anchored_chat_area(terminal: &ratatui::Terminal<ratatui::backend::TestBackend>) -> String
- top_token · function · L39-L46 — fn top_token(chat: &str) -> Option<String>
- resize_keeps_the_anchored_message_under_a_paused_reader · function · L49-L92 — fn resize_keeps_the_anchored_message_under_a_paused_reader()
- resize_back_to_the_original_width_returns_to_the_same_message · function · L95-L123 — fn resize_back_to_the_original_width_returns_to_the_same_message()
- tail_following_resize_captures_no_anchor · function · L126-L145 — fn tail_following_resize_captures_no_anchor()
- user_scroll_supersedes_a_pending_resize_anchor · function · L148-L165 — fn user_scroll_supersedes_a_pending_resize_anchor()
- resuming_the_tail_drops_a_pending_resize_anchor · function · L168-L193 — fn resuming_the_tail_drops_a_pending_resize_anchor()
- a_prepend_invalidates_a_pending_resize_anchor · function · L196-L218 — fn a_prepend_invalidates_a_pending_resize_anchor()
- a_pending_prepend_does_not_block_a_resize_capture · function · L221-L242 — fn a_pending_prepend_does_not_block_a_resize_capture()
- top_message_hash · function · L245-L254 — fn top_message_hash() -> Option<u64>
- resize_during_a_pending_prepend_keeps_the_same_message · function · L257-L295 — fn resize_during_a_pending_prepend_keeps_the_same_message()
- resize_anchor_filler · function · L298-L300 — fn resize_anchor_filler(index: usize) -> DisplayMessage
- resize_then_prepend_of_a_duplicate_does_not_teleport_the_reader · function · L307-L384 — fn resize_then_prepend_of_a_duplicate_does_not_teleport_the_reader()
- resize_keeps_a_reader_parked_in_live_output_on_the_same_text · function · L390-L443 — fn resize_keeps_a_reader_parked_in_live_output_on_the_same_text()
