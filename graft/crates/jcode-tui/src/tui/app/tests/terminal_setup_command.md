# crates/jcode-tui/src/tui/app/tests/terminal_setup_command.rs

- decode_key_event_via_pty · function · L28-L116 — fn decode_key_event_via_pty(bytes: &[u8]) -> Option<(crossterm::event::KeyCode, u8)>
- shift_enter_csi_u_sequence_decodes_to_enter_plus_shift · function · L120-L138 — fn shift_enter_csi_u_sequence_decodes_to_enter_plus_shift()
- bare_carriage_return_decodes_without_shift · function · L142-L157 — fn bare_carriage_return_decodes_without_shift()
- decoded_shift_enter_inserts_a_newline_instead_of_submitting · function · L160-L178 — fn decoded_shift_enter_inserts_a_newline_instead_of_submitting()
- terminal_setup_command_is_dispatched_and_reports_something_actionable · function · L188-L209 — fn terminal_setup_command_is_dispatched_and_reports_something_actionable()
- terminal_setup_is_offered_in_the_command_palette · function · L213-L223 — fn terminal_setup_is_offered_in_the_command_palette()
- unrelated_commands_are_not_swallowed_by_the_terminal_setup_handler · function · L226-L243 — fn unrelated_commands_are_not_swallowed_by_the_terminal_setup_handler()
- falling_back_to_a_backslash_newline_points_at_terminal_setup_once · function · L246-L275 — fn falling_back_to_a_backslash_newline_points_at_terminal_setup_once()
- working_shift_enter_never_triggers_the_terminal_setup_tip · function · L278-L291 — fn working_shift_enter_never_triggers_the_terminal_setup_tip()
