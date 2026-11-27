# crates/jcode-base/src/usage/flight_tests.rs

- with_home · function · L5-L23 — fn with_home(f: impl FnOnce(tokio::runtime::Runtime))
- Home · struct · L8-L8 — struct Home(Option<std::ffi::OsString>);
- drop · function · L10-L15 — fn drop(&mut self)
- sample · function · L25-L31 — fn sample() -> UsageData
- reset_offers_reuse_the_single_coordinated_usage_response · function · L34-L80 — fn reset_offers_reuse_the_single_coordinated_usage_response()
- response_diagnostics_capture_only_bounded_printable_retry_hints · function · L83-L99 — fn response_diagnostics_capture_only_bounded_printable_retry_hints()
- concurrent_callers_make_one_probe · function · L102-L121 — fn concurrent_callers_make_one_probe()
- different_accounts_can_probe_concurrently · function · L124-L145 — fn different_accounts_can_probe_concurrently()
- cancelled_probe_persists_backoff_and_releases_locks · function · L148-L175 — fn cancelled_probe_persists_backoff_and_releases_locks()
- deadline_is_cached_before_the_outer_account_timeout · function · L178-L204 — fn deadline_is_cached_before_the_outer_account_timeout()
- cancelled_lock_waiter_does_not_poison_the_account · function · L207-L226 — fn cancelled_lock_waiter_does_not_poison_the_account()
- received_429_is_cached_without_waiting_for_error_body · function · L229-L251 — fn received_429_is_cached_without_waiting_for_error_body()
- aborted_http_body_read_persists_backoff · function · L254-L281 — fn aborted_http_body_read_persists_backoff()
