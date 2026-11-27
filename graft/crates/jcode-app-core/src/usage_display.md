# crates/jcode-app-core/src/usage_display.rs

- reset_timestamp_passed · function · L6-L8 — pub(super) fn reset_timestamp_passed(timestamp: Option<&str>) -> bool
- display_snapshot · function · L12-L27 — pub fn display_snapshot(&self) -> Self
- display_snapshot · function · L32-L65 — pub fn display_snapshot(&self) -> Self
- provider_usage_cache_is_fresh · function · L68-L86 — pub(super) fn provider_usage_cache_is_fresh(
- format_token_count · function · L88-L96 — pub(super) fn format_token_count(tokens: u64) -> String
- humanize_key · function · L98-L114 — pub(super) fn humanize_key(key: &str) -> String
- parse_reset_timestamp · function · L116-L126 — fn parse_reset_timestamp(timestamp: &str) -> Option<chrono::DateTime<chrono::Utc>>
- usage_reset_passed · function · L128-L137 — pub(super) fn usage_reset_passed<'a>(
- format_reset_time · function · L139-L167 — pub fn format_reset_time(timestamp: &str) -> String
- format_usage_bar · function · L169-L175 — pub fn format_usage_bar(percent: f32, width: usize) -> String
