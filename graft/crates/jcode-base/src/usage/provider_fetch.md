# crates/jcode-base/src/usage/provider_fetch.rs

- is_transient_transport_error · function · L4-L6 — fn is_transient_transport_error(error: &reqwest::Error) -> bool
- usage_percent_from_used_limit · function · L8-L13 — pub(super) fn usage_percent_from_used_limit(used: f64, limit: f64) -> f32
- usage_percent_from_remaining_limit · function · L15-L20 — pub(super) fn usage_percent_from_remaining_limit(remaining: f64, limit: f64) -> f32
- fetch_anthropic_usage_for_token · function · L22-L92 — pub(super) async fn fetch_anthropic_usage_for_token(
- fetch_all_openai_usage_reports · function · L94-L141 — pub(super) async fn fetch_all_openai_usage_reports() -> Vec<ProviderUsage>
- fetch_openai_usage_report · function · L143-L148 — pub(super) async fn fetch_openai_usage_report() -> Option<ProviderUsage>
- fetch_openai_usage_for_account · function · L150-L376 — pub(super) async fn fetch_openai_usage_for_account(
- fetch_openrouter_usage_report · function · L378-L481 — pub(super) async fn fetch_openrouter_usage_report() -> Option<ProviderUsage>
- openrouter_api_key · function · L483-L498 — pub(super) fn openrouter_api_key() -> Option<String>
- fetch_antigravity_usage_report · function · L503-L580 — pub(super) async fn fetch_antigravity_usage_report() -> Option<ProviderUsage>
- fetch_gemini_usage_report · function · L585-L622 — pub(super) async fn fetch_gemini_usage_report() -> Option<ProviderUsage>
- fetch_cursor_usage_report · function · L628-L667 — pub(super) async fn fetch_cursor_usage_report() -> Option<ProviderUsage>
- cursor_plan_usage_report · function · L669-L717 — pub(super) fn cursor_plan_usage_report(json: &serde_json::Value) -> ProviderUsage
- cursor_key_status_report · function · L722-L790 — async fn cursor_key_status_report() -> ProviderUsage
- fetch_copilot_usage_report · function · L792-L913 — pub(super) async fn fetch_copilot_usage_report() -> Option<ProviderUsage>
- fetch_jcode_usage_report · function · L918-L927 — pub(super) async fn fetch_jcode_usage_report() -> Option<ProviderUsage>
- JCODE_UPGRADE_HINT_PERCENT · constant · L930-L930 — pub(crate) const JCODE_UPGRADE_HINT_PERCENT: f32 = 80.0;
- jcode_usage_report · function · L932-L1001 — pub(crate) fn jcode_usage_report(me: &crate::subscription_api::SubscriptionMe) -> ProviderUsage
