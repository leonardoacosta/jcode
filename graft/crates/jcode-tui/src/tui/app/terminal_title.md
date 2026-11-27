# crates/jcode-tui/src/tui/app/terminal_title.rs

- TerminalTitleState · struct · L11-L17 — pub(super) struct TerminalTitleState
- edit_line_counts · function · L19-L28 — pub(super) fn edit_line_counts(message: &DisplayMessage) -> (usize, usize)
- work_label · function · L30-L39 — fn work_label(duration: Duration) -> String
- title_with_metrics · function · L41-L61 — fn title_with_metrics(
- sync_herdr_agent_state · function · L67-L77 — pub(super) fn sync_herdr_agent_state(&self)
- set_terminal_title_base · function · L79-L89 — pub(super) fn set_terminal_title_base(&self, session_id: &str, base: String)
- refresh_terminal_title_metrics · function · L93-L123 — pub(super) fn refresh_terminal_title_metrics(&self)
- remember_terminal_title_work · function · L125-L134 — pub(super) fn remember_terminal_title_work(&self)
- tests · module · L138-L237 — mod tests
- terminal_title_counts_follow_append_remove_and_history_replace · function · L143-L166 — fn terminal_title_counts_follow_append_remove_and_history_replace()
- terminal_title_work_freezes_and_resets_on_session_switch · function · L169-L184 — fn terminal_title_work_freezes_and_resets_on_session_switch()
- title_metrics_show_edits_and_current_or_last_turn · function · L187-L202 — fn title_metrics_show_edits_and_current_or_last_turn()
- work_duration_boundaries_are_compact · function · L205-L216 — fn work_duration_boundaries_are_compact()
- edit_counts_ignore_failures_and_non_edit_output · function · L219-L236 — fn edit_counts_ignore_failures_and_non_edit_output()
