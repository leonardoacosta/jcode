# crates/jcode-tui/src/tui/app/remote/turn_identity_tests.rs

- identity · function · L5-L10 — fn identity(generation: &str, turn_id: u64) -> crate::protocol::TurnIdentity
- start · function · L11-L19 — fn start(app: &mut App, remote: &mut RemoteConnection, generation: &str, turn_id: u64)
- scoped · function · L20-L25 — fn scoped(generation: &str, turn_id: u64, event: ServerEvent) -> ServerEvent
- completed_turn_busy_continuation_retries_from_idle_without_another_terminal · function · L28-L111 — fn completed_turn_busy_continuation_retries_from_idle_without_another_terminal()
- completed_turn_busy_user_send_restores_prompt_without_overwriting_new_draft · function · L114-L175 — fn completed_turn_busy_user_send_restores_prompt_without_overwriting_new_draft()
- idle_bootstrap_settles_observer_after_disconnected_completion · function · L178-L202 — fn idle_bootstrap_settles_observer_after_disconnected_completion()
- local_send_rejection_preserves_running_observer_and_restores_prompt · function · L205-L260 — fn local_send_rejection_preserves_running_observer_and_restores_prompt()
- local_preacceptance_error_and_control_done_settle_without_active_turn · function · L263-L293 — fn local_preacceptance_error_and_control_done_settle_without_active_turn()
- paced_done_keeps_its_identity_across_reconnect_and_cannot_finish_new_turn · function · L296-L339 — fn paced_done_keeps_its_identity_across_reconnect_and_cannot_finish_new_turn()
- explicit_session_switch_can_revisit_previous_runtime · function · L342-L367 — fn explicit_session_switch_can_revisit_previous_runtime()
- old_completion_and_error_cannot_idle_newer_external_turn · function · L370-L399 — fn old_completion_and_error_cannot_idle_newer_external_turn()
- old_runtime_start_output_and_terminal_rejected_after_bootstrap · function · L402-L442 — fn old_runtime_start_output_and_terminal_rejected_after_bootstrap()
- reconnect_same_turn_accepts_current_terminal_but_idle_snapshot_blocks_replay · function · L445-L482 — fn reconnect_same_turn_accepts_current_terminal_but_idle_snapshot_blocks_replay()
- legacy_observer_stream_and_done_still_work · function · L485-L502 — fn legacy_observer_stream_and_done_still_work()
- unscoped_terminal_cannot_idle_scoped_observer · function · L505-L513 — fn unscoped_terminal_cannot_idle_scoped_observer()
