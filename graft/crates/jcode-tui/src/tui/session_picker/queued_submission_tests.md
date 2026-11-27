# crates/jcode-tui/src/tui/session_picker/queued_submission_tests.rs

- session · function · L7-L44 — fn session(id: &str) -> SessionInfo
- pending_picker · function · L46-L65 — fn pending_picker() -> (SessionPicker, mpsc::Sender<(String, SearchRank)>)
- better_result · function · L67-L78 — fn better_result(sender: &mpsc::Sender<(String, SearchRank)>)
- enter · function · L80-L84 — fn enter(picker: &mut SessionPicker) -> OverlayAction
- selected_id · function · L86-L100 — fn selected_id(action: Option<OverlayAction>) -> String
- immediate_type_enter_waits_without_blocking_then_opens_best_on_completion · function · L103-L114 — fn immediate_type_enter_waits_without_blocking_then_opens_best_on_completion()
- deadline_opens_best_available_even_if_worker_is_still_pending · function · L117-L124 — fn deadline_opens_best_available_even_if_worker_is_still_pending()
- repeated_enter_cannot_extend_query_deadline_or_open_stale_selection · function · L127-L134 — fn repeated_enter_cannot_extend_query_deadline_or_open_stale_selection()
- editing_or_manual_navigation_cancels_queued_enter · function · L137-L154 — fn editing_or_manual_navigation_cancels_queued_enter()
- direct_navigation_and_preview_scroll_cancel_queued_enter · function · L157-L175 — fn direct_navigation_and_preview_scroll_cancel_queued_enter()
- manual_selection_submits_immediately_without_following_background_results · function · L178-L183 — fn manual_selection_submits_immediately_without_following_background_results()
- no_matches_at_deadline_keeps_query_and_does_not_submit_later · function · L186-L201 — fn no_matches_at_deadline_keeps_query_and_does_not_submit_later()
- loading_picker_never_queues_enter · function · L204-L212 — fn loading_picker_never_queues_enter()
