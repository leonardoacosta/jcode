# crates/jcode-base/src/usage/account_fetch.rs

- ACCOUNT_PROBE_TIMEOUT · constant · L7-L7 — const ACCOUNT_PROBE_TIMEOUT: Duration = Duration::from_secs(20);
- cache_anthropic_account_failure · function · L9-L24 — pub(super) fn cache_anthropic_account_failure(
- bounded_probe · function · L26-L44 — async fn bounded_probe<F>(label: String, email: Option<String>, probe: F) -> AccountUsageSnapshot
- fetch_oauth_account_usage · function · L48-L124 — pub async fn fetch_oauth_account_usage(
- tests · module · L127-L238 — mod tests
- healthy · function · L130-L140 — fn healthy(label: &str) -> AccountUsageSnapshot
- stalled_accounts_do_not_discard_healthy_usage_or_serialize_deadlines · function · L143-L174 — async fn stalled_accounts_do_not_discard_healthy_usage_or_serialize_deadlines()
- successful_probes_do_not_wait_for_the_deadline · function · L177-L183 — async fn successful_probes_do_not_wait_for_the_deadline()
- cached_usage_does_not_clear_runtime_quota_failures · function · L186-L237 — async fn cached_usage_does_not_clear_runtime_quota_failures()
