# crates/jcode-app-core/src/update_rate_limit.rs

- RATE_LIMIT_BACKOFF_FALLBACK · constant · L15-L15 — pub(super) const RATE_LIMIT_BACKOFF_FALLBACK: Duration = Duration::from_secs(60 * 60);
- RATE_LIMIT_BACKOFF_MAX · constant · L18-L18 — pub(super) const RATE_LIMIT_BACKOFF_MAX: Duration = Duration::from_secs(6 * 60 * 60);
- RATE_LIMIT_ERROR_PREFIX · constant · L21-L21 — pub const RATE_LIMIT_ERROR_PREFIX: &str = "GitHub API rate limit reached";
- rate_limit_error · function · L29-L49 — pub(super) fn rate_limit_error(response: &reqwest::blocking::Response) -> Option<anyhow::Error>
- rate_limit_backoff · function · L58-L85 — fn rate_limit_backoff(
- record_rate_limit_backoff · function · L89-L100 — fn record_rate_limit_backoff(backoff: Duration)
- clear_rate_limit_backoff · function · L103-L110 — pub(super) fn clear_rate_limit_backoff()
- is_rate_limit_error · function · L114-L116 — pub fn is_rate_limit_error(error: &str) -> bool
- tests · module · L119-L166 — mod tests
- ignores_non_rate_limit_statuses · function · L123-L130 — fn ignores_non_rate_limit_statuses()
- uses_reset_header · function · L133-L137 — fn uses_reset_header()
- prefers_retry_after_and_clamps · function · L140-L156 — fn prefers_retry_after_and_clamps()
- matches_wrapped_error_message · function · L159-L165 — fn matches_wrapped_error_message()
