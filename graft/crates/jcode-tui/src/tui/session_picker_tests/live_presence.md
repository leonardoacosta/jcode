# crates/jcode-tui/src/tui/session_picker_tests/live_presence.rs

- live_presence · function · L1-L10 — fn live_presence(session_id: &str, streaming: bool) -> crate::session::SessionPresence
- test_active_filter_shows_only_live_sessions_ready_before_working · function · L13-L43 — fn test_active_filter_shows_only_live_sessions_ready_before_working()
- test_running_rows_show_spinner_state_and_ready_rows_show_age · function · L46-L113 — fn test_running_rows_show_spinner_state_and_ready_rows_show_age()
- test_current_session_row_is_labeled · function · L116-L127 — fn test_current_session_row_is_labeled()
- test_recent_window_close_boosts_old_transcript_above_newer_messages_and_saved_rows · function · L130-L158 — fn test_recent_window_close_boosts_old_transcript_above_newer_messages_and_saved_rows()
- test_recent_window_close_promotes_orphan_above_server_group_without_duplicates · function · L161-L189 — fn test_recent_window_close_promotes_orphan_above_server_group_without_duplicates()
- test_recent_window_close_expires_and_rejects_future_timestamps_and_current_session · function · L192-L210 — fn test_recent_window_close_expires_and_rejects_future_timestamps_and_current_session()
- test_maybe_refresh_live_presence_throttles_and_detects_changes · function · L213-L220 — fn test_maybe_refresh_live_presence_throttles_and_detects_changes()
