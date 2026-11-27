# crates/jcode-base/src/usage/display.rs

- reset_timestamp_passed · function · L6-L8 — pub(super) fn reset_timestamp_passed(timestamp: Option<&str>) -> bool
- display_snapshot · function · L12-L27 — pub fn display_snapshot(&self) -> Self
- display_snapshot · function · L32-L72 — pub fn display_snapshot(&self) -> Self
- provider_usage_cache_is_fresh · function · L75-L93 — pub(super) fn provider_usage_cache_is_fresh(
- format_token_count · function · L95-L103 — pub(super) fn format_token_count(tokens: u64) -> String
- humanize_key · function · L105-L121 — pub(super) fn humanize_key(key: &str) -> String
- parse_reset_timestamp · function · L123-L133 — fn parse_reset_timestamp(timestamp: &str) -> Option<chrono::DateTime<chrono::Utc>>
- usage_reset_passed · function · L135-L144 — pub(super) fn usage_reset_passed<'a>(
- format_reset_time · function · L146-L174 — pub fn format_reset_time(timestamp: &str) -> String
- format_usage_bar · function · L176-L184 — pub fn format_usage_bar(percent: f32, width: usize) -> String
