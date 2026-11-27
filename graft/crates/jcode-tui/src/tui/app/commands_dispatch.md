# crates/jcode-tui/src/tui/app/commands_dispatch.rs

- ssh_presentation_command · function · L15-L61 — pub(super) fn ssh_presentation_command(input: &str) -> bool
- handle_ssh_presentation_command · function · L63-L100 — pub(super) fn handle_ssh_presentation_command(app: &mut App, input: &str) -> bool
- ssh_unsupported_command · function · L104-L178 — pub(super) fn ssh_unsupported_command(input: &str) -> bool
- ssh_local_action_blocked · function · L181-L194 — pub(super) fn ssh_local_action_blocked(app: &mut App, action: &str) -> bool
- handle_ssh_unsupported_command · function · L197-L206 — pub(super) fn handle_ssh_unsupported_command(app: &mut App, input: &str) -> bool
- contains_registered_slash_command · function · L213-L216 — pub(super) fn contains_registered_slash_command(input: &str) -> bool
- dispatch_local_command · function · L221-L229 — pub(super) fn dispatch_local_command(app: &mut App, trimmed: &str) -> bool
- dispatch_command_sequence · function · L231-L263 — fn dispatch_command_sequence(
- dispatch_single_local_command · function · L265-L309 — fn dispatch_single_local_command(app: &mut App, trimmed: &str) -> bool
- ssh_test_runs_in_child · function · L314-L335 — pub(super) fn ssh_test_runs_in_child(test_name: &str) -> bool
- tests · module · L338-L603 — mod tests
- ssh_ui_blocks_local_actions_and_preserves_text_and_cancel · function · L340-L399 — fn ssh_ui_blocks_local_actions_and_preserves_text_and_cancel()
- ssh_blocks_local_side_effects_but_preserves_wire_commands · function · L402-L464 — fn ssh_blocks_local_side_effects_but_preserves_wire_commands()
- both_entry_points_use_the_shared_dispatch_table · function · L470-L491 — fn both_entry_points_use_the_shared_dispatch_table()
- ssh_presentation_commands_are_not_blocked_or_prefix_matched · function · L494-L544 — fn ssh_presentation_commands_are_not_blocked_or_prefix_matched()
- ssh_display_preferences_work_without_remote_or_local_session_operations · function · L547-L586 — fn ssh_display_preferences_work_without_remote_or_local_session_operations()
- ssh_clear_view_preserves_session_context · function · L589-L602 — fn ssh_clear_view_preserves_session_context()
