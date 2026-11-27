# crates/jcode-base/src/mobile_runner_events_tests.rs

- HINT · constant · L9-L9 — const HINT: &str = r#"{"type":"commands_available"}"#;
- FALLBACK · constant · L10-L10 — const FALLBACK: Duration = Duration::from_secs(5);
- fixture · function · L12-L42 — async fn fixture() -> (Wakeups, WebSocketStream<TcpStream>)
- changed_source_closes_old_socket_and_authenticates_only_new_endpoint · function · L45-L116 — async fn changed_source_closes_old_socket_and_authenticates_only_new_endpoint()
- request_uses_bearer_header_without_credentials_in_url · function · L119-L140 — fn request_uses_bearer_header_without_credentials_in_url()
- hints_accept_only_the_exact_non_authoritative_shape · function · L143-L156 — fn hints_accept_only_the_exact_non_authoritative_shape()
- notification_wakes_before_five_second_fallback · function · L159-L165 — async fn notification_wakes_before_five_second_fallback()
- notification_drives_a_real_poll_loop_claim_without_waiting_five_seconds · function · L168-L206 — async fn notification_drives_a_real_poll_loop_claim_without_waiting_five_seconds()
- notification_is_buffered_and_bursts_coalesce · function · L209-L231 — async fn notification_is_buffered_and_bursts_coalesce()
- dropped_socket_keeps_timer_fallback · function · L234-L251 — async fn dropped_socket_keeps_timer_fallback()
- dropping_wakeups_cancels_connection_task_and_socket · function · L254-L269 — async fn dropping_wakeups_cancels_connection_task_and_socket()
- malformed_and_oversize_frames_fail_closed · function · L272-L290 — async fn malformed_and_oversize_frames_fail_closed()
- error_backoff_cannot_be_bypassed_by_buffered_hint · function · L293-L303 — async fn error_backoff_cannot_be_bypassed_by_buffered_hint()
- unavailable_endpoint_does_not_hot_loop_and_fallback_remains_live · function · L306-L337 — async fn unavailable_endpoint_does_not_hot_loop_and_fallback_remains_live()
