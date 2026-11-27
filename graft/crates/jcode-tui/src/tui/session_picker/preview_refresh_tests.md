# crates/jcode-tui/src/tui/session_picker/preview_refresh_tests.rs

- fixture · function · L4-L40 — fn fixture() -> SessionInfo
- message · function · L42-L52 — fn message(role: &str, content: &str) -> PreviewMessage
- populated_transcript_still_loads_missing_preview_metadata · function · L55-L63 — fn populated_transcript_still_loads_missing_preview_metadata()
- preview_metadata_refreshes_after_freshness_window · function · L66-L85 — fn preview_metadata_refreshes_after_freshness_window()
- metadata_failure_does_not_invalidate_a_populated_transcript · function · L88-L98 — fn metadata_failure_does_not_invalidate_a_populated_transcript()
- workspace_change_keeps_transcript_but_rejects_old_metadata_and_pending_result · function · L101-L126 — fn workspace_change_keeps_transcript_but_rejects_old_metadata_and_pending_result()
- refresh_equality_includes_duration_and_title · function · L129-L144 — fn refresh_equality_includes_duration_and_title()
- deliver · function · L146-L163 — fn deliver(picker: &mut SessionPicker, messages: Vec<PreviewMessage>)
- render · function · L165-L175 — fn render(picker: &mut SessionPicker) -> String
- loaded_conversation_survives_metadata_only_refresh_and_group_change · function · L178-L214 — fn loaded_conversation_survives_metadata_only_refresh_and_group_change()
- changed_transcript_invalidates_loaded_state_and_reloads_new_messages · function · L217-L247 — fn changed_transcript_invalidates_loaded_state_and_reloads_new_messages()
- refresh_discards_stale_inflight_load_but_preserves_unchanged_one · function · L250-L268 — fn refresh_discards_stale_inflight_load_but_preserves_unchanged_one()
- genuinely_empty_completed_preview_remains_loaded_and_removed_sessions_release_extras · function · L271-L284 — fn genuinely_empty_completed_preview_remains_loaded_and_removed_sessions_release_extras()
- resume_time_modes_pick_expected_timestamp · function · L287-L331 — fn resume_time_modes_pick_expected_timestamp()
- running_session_shows_spinner_state_instead_of_age · function · L334-L359 — fn running_session_shows_spinner_state_instead_of_age()
