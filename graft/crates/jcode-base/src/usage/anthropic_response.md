# crates/jcode-base/src/usage/anthropic_response.rs

- window · function · L5-L25 — fn window(headers: &reqwest::header::HeaderMap, name: &str) -> Option<(f32, String)>
- record_anthropic_response_limits · function · L31-L42 — pub fn record_anthropic_response_limits(
- credential_error · function · L46-L56 — pub(super) fn credential_error(error: anyhow::Error) -> anyhow::Error
- invalidate_auth_failure · function · L60-L71 — pub(crate) fn invalidate_auth_failure(label: &str)
- is_auth_failure · function · L73-L78 — pub(super) fn is_auth_failure(error: &str) -> bool
- tests · module · L81-L306 — mod tests
- with_home · function · L84-L102 — fn with_home(f: impl FnOnce(tokio::runtime::Runtime))
- Restore · struct · L87-L87 — struct Restore(Option<std::ffi::OsString>);
- drop · function · L89-L94 — fn drop(&mut self)
- headers · function · L104-L116 — fn headers() -> reqwest::header::HeaderMap
- response_limits_recover_bars_without_bypassing_endpoint_backoff · function · L119-L163 — fn response_limits_recover_bars_without_bypassing_endpoint_backoff()
- partial_invalid_and_expired_headers_never_create_zero_usage · function · L166-L191 — fn partial_invalid_and_expired_headers_never_create_zero_usage()
- first_failed_account_probe_keeps_the_shared_retry_after · function · L194-L214 — fn first_failed_account_probe_keeps_the_shared_retry_after()
- permanent_refresh_errors_are_actionable_but_network_errors_are_not_auth_failures · function · L217-L234 — fn permanent_refresh_errors_are_actionable_but_network_errors_are_not_auth_failures()
- expired_rejected_account_stops_refreshing_and_relogin_clears_auth_cache · function · L237-L305 — fn expired_rejected_account_stops_refreshing_and_relogin_clears_auth_cache()
