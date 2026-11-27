# crates/jcode-harness-api-server/src/translate_tests/tool_controls.rs

- tool_state · function · L3-L9 — fn tool_state() -> BridgeState
- session_tool_controls_translate_and_ack_without_ending_turn · function · L12-L43 — fn session_tool_controls_translate_and_ack_without_ending_turn()
- session_tool_inventory_correlates_and_validates_daemon_response · function · L46-L92 — fn session_tool_inventory_correlates_and_validates_daemon_response()
- custom_tool_call_is_distinct_from_streaming_lifecycle · function · L95-L129 — fn custom_tool_call_is_distinct_from_streaming_lifecycle()
- session_tool_controls_require_attachment_and_negotiated_support · function · L132-L184 — fn session_tool_controls_require_attachment_and_negotiated_support()
- malformed_tool_controls_never_reach_daemon · function · L187-L211 — fn malformed_tool_controls_never_reach_daemon()
- tool_controls_validate_pending_attachment_and_required_session · function · L214-L237 — fn tool_controls_validate_pending_attachment_and_required_session()
- detach_waits_for_release_barrier_and_drops_late_callbacks · function · L240-L274 — fn detach_waits_for_release_barrier_and_drops_late_callbacks()
- abnormal_stop_precedes_error_and_done_and_preserves_specific_reason · function · L277-L317 — fn abnormal_stop_precedes_error_and_done_and_preserves_specific_reason()
- guardrail_stop_preserves_provider_reason_and_finishes_observer_turn · function · L320-L332 — fn guardrail_stop_preserves_provider_reason_and_finishes_observer_turn()
- legacy_interrupt_finishes_observer_turn_but_idle_cancel_is_not_a_stop · function · L335-L361 — fn legacy_interrupt_finishes_observer_turn_but_idle_cancel_is_not_a_stop()
- request_error_does_not_fabricate_a_turn_stop · function · L364-L375 — fn request_error_does_not_fabricate_a_turn_stop()
- tool_streaming_forwards_zero_argument_starts_and_keyed_interleaved_input · function · L378-L400 — fn tool_streaming_forwards_zero_argument_starts_and_keyed_interleaved_input()
- pre_stream_activity_invalidates_older_idle_history · function · L403-L455 — fn pre_stream_activity_invalidates_older_idle_history()
- transport_bookkeeping_does_not_start_a_turn · function · L458-L468 — fn transport_bookkeeping_does_not_start_a_turn()
