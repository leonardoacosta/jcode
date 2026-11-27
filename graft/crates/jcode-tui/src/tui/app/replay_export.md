# crates/jcode-tui/src/tui/app/replay_export.rs

- with_expanded_replay_todos · function · L13-L16 — pub fn with_expanded_replay_todos(mut self) -> Self
- restore_replay_todos_json · function · L18-L22 — pub(super) fn restore_replay_todos_json(&mut self, output: &str)
- render_replay_states · function · L25-L36 — pub fn render_replay_states(
- render_replay_frames · function · L43-L107 — pub fn render_replay_frames(
- replay_frame_ansi · function · L111-L129 — pub fn replay_frame_ansi(buffer: &Buffer) -> Result<Vec<u8>>
- normalize_todo_snapshot · function · L132-L201 — pub(super) fn normalize_todo_snapshot(value: Value) -> Result<String>
- todo_snapshot_from_output · function · L205-L242 — pub(super) fn todo_snapshot_from_output(output: &str) -> Option<String>
- historical_export_tests · module · L246-L246 — mod historical_export_tests;
- tests · module · L249-L390 — mod tests
- task · function · L253-L255 — fn task(label: &str) -> Value
- app · function · L257-L263 — fn app() -> App
- snapshots_validate_arrays_full_cards_and_public_tool_output · function · L266-L279 — fn snapshots_validate_arrays_full_cards_and_public_tool_output()
- selected_frame_is_exact_production_draw_and_repeated_times_match · function · L282-L313 — async fn selected_frame_is_exact_production_draw_and_repeated_times_match()
- todo_events_restore_public_names_and_ignore_errors_and_live_refresh · function · L316-L358 — async fn todo_events_restore_public_names_and_ignore_errors_and_live_refresh()
- invalid_dimensions_and_unordered_histories_are_rejected · function · L361-L373 — async fn invalid_dimensions_and_unordered_histories_are_rejected()
- ansi_preserves_unicode_and_blank_cell_backgrounds · function · L376-L389 — fn ansi_preserves_unicode_and_blank_cell_backgrounds()
