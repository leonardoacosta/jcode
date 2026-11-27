# crates/jcode-app-core/src/server/client_session_tests/reload.rs

- restored_active_pending_tool_counts_as_interrupted · function · L5-L27 — fn restored_active_pending_tool_counts_as_interrupted()
- restored_active_completed_assistant_does_not_count_as_interrupted · function · L30-L49 — fn restored_active_completed_assistant_does_not_count_as_interrupted()
- detects_reload_interrupted_generation_text · function · L52-L68 — fn detects_reload_interrupted_generation_text()
- detects_reload_interrupted_tool_result · function · L71-L88 — fn detects_reload_interrupted_tool_result()
- detects_reload_skipped_tool_result · function · L91-L108 — fn detects_reload_skipped_tool_result()
- detects_selfdev_reload_tool_result_even_when_not_marked_error · function · L111-L128 — fn detects_selfdev_reload_tool_result_even_when_not_marked_error()
- ignores_normal_tool_errors · function · L131-L148 — fn ignores_normal_tool_errors()
- restored_closed_session_with_reload_marker_still_counts_as_interrupted · function · L151-L171 — fn restored_closed_session_with_reload_marker_still_counts_as_interrupted()
- restored_closed_session_with_pending_user_message_during_reload_should_count_as_interrupted · function · L174-L219 — fn restored_closed_session_with_pending_user_message_during_reload_should_count_as_interrupted()
- restored_closed_session_with_pending_user_message_during_socket_ready_handoff_counts_as_interrupted · function · L222-L267 — fn restored_closed_session_with_pending_user_message_during_socket_ready_handoff_counts_as_interrupted()
- restored_closed_session_with_pending_user_message_without_reload_marker_is_not_interrupted · function · L270-L304 — fn restored_closed_session_with_pending_user_message_without_reload_marker_is_not_interrupted()
- restored_closed_session_without_reload_marker_is_not_interrupted · function · L307-L327 — fn restored_closed_session_without_reload_marker_is_not_interrupted()
- mark_remote_reload_started_writes_starting_marker · function · L330-L350 — fn mark_remote_reload_started_writes_starting_marker() -> Result<()>
- handle_reload_queues_signal_for_canary_session · function · L353-L466 — fn handle_reload_queues_signal_for_canary_session() -> Result<()>
- handle_reload_does_not_wait_for_busy_agent_lock · function · L469-L532 — async fn handle_reload_does_not_wait_for_busy_agent_lock() -> Result<()>
- rename_shutdown_signal_moves_registration_to_restored_session · function · L535-L552 — async fn rename_shutdown_signal_moves_registration_to_restored_session() -> Result<()>
