# crates/jcode-tui/src/tui/app/tests/reload_recovery_replay.rs

- reload_replay_history · function · L1-L55 — fn reload_replay_history(
- reload_replay_context · function · L57-L65 — fn reload_replay_context(session_id: &str) -> ReloadContext
- reload_recovery_execution_retries_are_not_delivery_replays · function · L68-L160 — fn reload_recovery_execution_retries_are_not_delivery_replays()
- reload_recovery_cloud_retry_marks_final_wire_reminder · function · L163-L237 — fn reload_recovery_cloud_retry_marks_final_wire_reminder()
- reload_recovery_same_route_tick_marks_execution_retry · function · L240-L277 — fn reload_recovery_same_route_tick_marks_execution_retry()
- reload_recovery_cloud_only_retry_marks_final_reminder_once · function · L280-L328 — fn reload_recovery_cloud_only_retry_marks_final_reminder_once()
- reload_recovery_prepared_send_failure_restores_draft_without_exiting · function · L331-L372 — fn reload_recovery_prepared_send_failure_restores_draft_without_exiting()
- reload_recovery_keyboard_send_failure_keeps_client_and_draft_alive · function · L375-L398 — fn reload_recovery_keyboard_send_failure_keeps_client_and_draft_alive()
- reload_recovery_deferred_send_failure_preserves_newer_draft · function · L401-L444 — fn reload_recovery_deferred_send_failure_preserves_newer_draft()
- reload_recovery_preserves_visible_turn_timer_through_dispatch · function · L447-L483 — fn reload_recovery_preserves_visible_turn_timer_through_dispatch()
- reload_recovery_delayed_history_after_dispatch_does_not_send_again · function · L486-L522 — fn reload_recovery_delayed_history_after_dispatch_does_not_send_again()
- reload_recovery_delayed_history_after_completion_does_not_restart_turn · function · L525-L557 — fn reload_recovery_delayed_history_after_completion_does_not_restart_turn()
- reload_recovery_history_then_local_context_preserves_active_turn · function · L560-L590 — fn reload_recovery_history_then_local_context_preserves_active_turn()
- reload_recovery_dedup_is_session_scoped_and_resets_for_new_reload · function · L593-L635 — fn reload_recovery_dedup_is_session_scoped_and_resets_for_new_reload()
