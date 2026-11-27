# crates/jcode-app-core/src/server/client_lifecycle_tests/control_requests.rs

- cancel_soft_interrupts_by_id_reports_each_requested_id_exactly_once · function · L4-L46 — fn cancel_soft_interrupts_by_id_reports_each_requested_id_exactly_once()
- session_control_handle_does_not_wait_for_busy_agent_lock · function · L49-L85 — async fn session_control_handle_does_not_wait_for_busy_agent_lock()
- refreshed_session_control_handle_does_not_wait_for_busy_agent_lock · function · L88-L130 — async fn refreshed_session_control_handle_does_not_wait_for_busy_agent_lock()
- busy_session_background_tool_signal_fires_via_registry_fallback · function · L133-L177 — async fn busy_session_background_tool_signal_fires_via_registry_fallback()
- busy_agent_request_rejection_does_not_wait_for_agent_lock · function · L180-L221 — async fn busy_agent_request_rejection_does_not_wait_for_agent_lock()
- context_message_persists_without_starting_turn · function · L224-L272 — async fn context_message_persists_without_starting_turn()
- context_message_rejects_while_busy_without_waiting_for_agent_lock · function · L275-L307 — async fn context_message_rejects_while_busy_without_waiting_for_agent_lock()
- cancel_without_local_task_still_signals_session_control · function · L310-L378 — async fn cancel_without_local_task_still_signals_session_control()
- deferred_cancel_reset_does_not_erase_newer_cancel · function · L386-L458 — async fn deferred_cancel_reset_does_not_erase_newer_cancel()
