# crates/jcode-base/src/provider/multi_provider.rs

- is_failover_route_pool_approved · function · L7-L77 — pub fn is_failover_route_pool_approved(route: &str) -> bool
- account_failover_prompt · function · L79-L103 — fn account_failover_prompt(
- cross_provider_pool_prompt · function · L105-L310 — pub(super) fn cross_provider_pool_prompt(
- try_same_provider_account_failover · function · L312-L361 — pub(super) async fn try_same_provider_account_failover(
- candidate_model · function · L364-L370 — fn candidate_model(candidate: &(String, ActiveProvider, String, String)) -> &str
- candidate_matches_configured_model · function · L372-L394 — fn candidate_matches_configured_model(
- saved_account_can_serve_model · function · L396-L447 — fn saved_account_can_serve_model(provider: &str, label: &str, model: &str) -> bool
- saved_account_labels · function · L449-L467 — fn saved_account_labels(provider: &str) -> Vec<String>
- claude_saved_account_readiness_uses_live_access_or_usable_refresh · function · L471-L497 — fn claude_saved_account_readiness_uses_live_access_or_usable_refresh()
