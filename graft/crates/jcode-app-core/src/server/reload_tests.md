# crates/jcode-app-core/src/server/reload_tests.rs

- set_member_status · function · L12-L20 — fn set_member_status(members: &mut HashMap<String, SwarmMember>, session_id: &str, status: &str)
- member · function · L22-L46 — fn member(session_id: &str, status: &str) -> SwarmMember
- receive_reload_signal_consumes_already_pending_value · function · L49-L71 — async fn receive_reload_signal_consumes_already_pending_value()
- receive_reload_signal_waits_for_future_value_when_initially_empty · function · L74-L98 — async fn receive_reload_signal_waits_for_future_value_when_initially_empty()
- persist_reload_recovery_intents_records_running_peer_recovery · function · L101-L150 — fn persist_reload_recovery_intents_records_running_peer_recovery() -> anyhow::Result<()>
- graceful_shutdown_sessions_signals_all_running_sessions_including_initiator · function · L153-L223 — async fn graceful_shutdown_sessions_signals_all_running_sessions_including_initiator()
- graceful_shutdown_sessions_does_not_wait_for_triggering_session_checkpoint · function · L226-L297 — async fn graceful_shutdown_sessions_does_not_wait_for_triggering_session_checkpoint()
- graceful_shutdown_sessions_skips_idle_sessions · function · L300-L329 — async fn graceful_shutdown_sessions_skips_idle_sessions()
- graceful_shutdown_sessions_does_not_wait_on_running_sessions_without_signal · function · L332-L358 — async fn graceful_shutdown_sessions_does_not_wait_on_running_sessions_without_signal()
- graceful_shutdown_sessions_waits_until_target_status_change_arrives · function · L361-L427 — async fn graceful_shutdown_sessions_waits_until_target_status_change_arrives()
- graceful_shutdown_sessions_ignores_unrelated_events_until_target_leaves · function · L430-L506 — async fn graceful_shutdown_sessions_ignores_unrelated_events_until_target_leaves()
- graceful_shutdown_sessions_treats_member_left_as_unblocked · function · L509-L560 — async fn graceful_shutdown_sessions_treats_member_left_as_unblocked()
- graceful_shutdown_sessions_times_out_and_proceeds · function · L563-L599 — async fn graceful_shutdown_sessions_times_out_and_proceeds()
- graceful_shutdown_sessions_times_out_on_partial_checkpoint · function · L602-L668 — async fn graceful_shutdown_sessions_times_out_on_partial_checkpoint()
