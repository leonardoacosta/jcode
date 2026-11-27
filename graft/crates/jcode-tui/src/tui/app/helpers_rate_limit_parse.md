# crates/jcode-tui/src/tui/app/helpers_rate_limit_parse.rs

- MAX_TURN_HOLD_SECS · constant · L9-L9 — pub(crate) const MAX_TURN_HOLD_SECS: u64 = 24 * 60 * 60;
- structured_retry_after_seconds · function · L14-L18 — fn structured_retry_after_seconds(error: &str) -> Option<u64>
- parse_rate_limit_error · function · L22-L122 — pub(crate) fn parse_rate_limit_error(error: &str) -> Option<Duration>
- rate_limit_parse_tests · module · L126-L187 — mod rate_limit_parse_tests
- account_pool_failover_prompt_never_schedules_same_account_retry · function · L131-L147 — fn account_pool_failover_prompt_never_schedules_same_account_retry()
- usage_limit_reset_in_days_does_not_schedule_bogus_short_retry · function · L150-L155 — fn usage_limit_reset_in_days_does_not_schedule_bogus_short_retry()
- unit_suffixed_reset_within_a_day_is_parsed · function · L158-L164 — fn unit_suffixed_reset_within_a_day_is_parsed()
- plain_retry_seconds_still_parse · function · L167-L170 — fn plain_retry_seconds_still_parse()
- openference_json_retry_after_seconds_is_parsed · function · L173-L176 — fn openference_json_retry_after_seconds_is_parsed()
- invalid_json_retry_after_does_not_use_max_rpm · function · L179-L186 — fn invalid_json_retry_after_does_not_use_max_rpm()
