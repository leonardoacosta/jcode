# crates/jcode-base/src/auth/oauth_tests/openai_refresh_deadline.rs

- read_request · function · L10-L29 — async fn read_request(stream: &mut tokio::net::TcpStream) -> String
- token_url · function · L31-L33 — fn token_url(listener: &TcpListener) -> String
- success_body · function · L35-L42 — fn success_body(access: &str, refresh: &str) -> String
- http_response · function · L44-L49 — fn http_response(status: u16, body: &str) -> String
- sandbox · function · L51-L53 — fn sandbox() -> crate::auth::test_sandbox::AuthTestSandbox
- seed_account · function · L56-L59 — fn seed_account(refresh: &str) -> String
- stored_refresh_token · function · L61-L63 — fn stored_refresh_token(label: &str) -> Option<String>
- rejected_cached_token_does_not_block_newly_imported_account · function · L66-L115 — async fn rejected_cached_token_does_not_block_newly_imported_account()
- refresh_deadline_bounds_withheld_headers · function · L118-L143 — async fn refresh_deadline_bounds_withheld_headers()
- refresh_deadline_bounds_incomplete_body · function · L146-L175 — async fn refresh_deadline_bounds_incomplete_body()
- refresh_deadline_failure_leaves_stored_token_and_releases_lock · function · L178-L221 — async fn refresh_deadline_failure_leaves_stored_token_and_releases_lock()
- refresh_cancelled_after_rotation_still_persists · function · L224-L267 — async fn refresh_cancelled_after_rotation_still_persists()
- refresh_waiter_cancellation_does_not_disturb_in_flight_owner · function · L270-L323 — async fn refresh_waiter_cancellation_does_not_disturb_in_flight_owner()
- refresh_malformed_json_does_not_echo_body_or_persist · function · L326-L350 — async fn refresh_malformed_json_does_not_echo_body_or_persist()
- refresh_error_response_is_reported_and_marks_rejection · function · L353-L381 — async fn refresh_error_response_is_reported_and_marks_rejection()
