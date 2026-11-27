# crates/jcode-sdk/src/launch_upgrade_tests.rs

- sequenced_server · function · L5-L35 — fn sequenced_server(
- deferred_busy_upgrade_retries_then_stops_when_runtime_is_current · function · L38-L45 — fn deferred_busy_upgrade_retries_then_stops_when_runtime_is_current()
- busy_retry_is_bounded_and_reports_reopen_app_after_idle · function · L48-L57 — fn busy_retry_is_bounded_and_reports_reopen_app_after_idle()
- reconnect_confirmation_requires_version_decision_not_socket_connect · function · L60-L67 — fn reconnect_confirmation_requires_version_decision_not_socket_connect()
- custom_remote_or_path_fallback_never_qualifies_for_local_upgrade · function · L70-L86 — fn custom_remote_or_path_fallback_never_qualifies_for_local_upgrade()
- negotiate · function · L88-L119 — fn negotiate(replies: Vec<Value>) -> (Outcome, Value)
- busy_runtime_defers_with_clear_status_without_reload_fallback · function · L122-L131 — fn busy_runtime_defers_with_clear_status_without_reload_fallback()
- old_runtime_unknown_request_is_compatible_and_does_not_force_restart · function · L134-L143 — fn old_runtime_unknown_request_is_compatible_and_does_not_force_restart()
- no_downgrade_or_unchanged_runtime_does_not_restart · function · L146-L151 — fn no_downgrade_or_unchanged_runtime_does_not_restart()
- failed_exec_preserves_old_runtime_status · function · L154-L160 — fn failed_exec_preserves_old_runtime_status()
- accepted_upgrade_requires_exec_connection_close_before_reconnect · function · L163-L185 — fn accepted_upgrade_requires_exec_connection_close_before_reconnect()
- upgrade_negotiation_is_bounded_for_unresponsive_old_runtime · function · L188-L200 — fn upgrade_negotiation_is_bounded_for_unresponsive_old_runtime()
