# crates/jcode-provider-openai-runtime/src/openai_tests/http_idle.rs

- http_transport_idle_case · function · L2-L112 — async fn http_transport_idle_case(heartbeats: bool, complete: bool)
- http_idle_heartbeats_keep_response_alive_until_completion · function · L115-L117 — async fn http_idle_heartbeats_keep_response_alive_until_completion()
- http_idle_completion_does_not_wait_for_eof · function · L120-L122 — async fn http_idle_completion_does_not_wait_for_eof()
- http_idle_silent_transport_still_times_out · function · L125-L127 — async fn http_idle_silent_transport_still_times_out()
- http_idle_stopped_heartbeats_still_time_out · function · L130-L132 — async fn http_idle_stopped_heartbeats_still_time_out()
- http_terminal_auth_case · function · L139-L219 — async fn http_terminal_auth_case(
- http_second_401_after_refresh_is_auth_expired_without_another_refresh · function · L222-L224 — async fn http_second_401_after_refresh_is_auth_expired_without_another_refresh()
- http_quota_rejection_skips_refresh_and_shows_quota_remedy · function · L227-L234 — async fn http_quota_rejection_skips_refresh_and_shows_quota_remedy()
