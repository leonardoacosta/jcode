# crates/jcode-base/src/provider/account_failover.rs

- provider_uses_oauth_accounts · function · L7-L28 — pub(super) fn provider_uses_oauth_accounts(&self, provider: ActiveProvider) -> bool
- provider_runtime_cooldown_key · function · L30-L38 — pub(super) fn provider_runtime_cooldown_key(&self, provider: ActiveProvider) -> &'static str
- multi_account_provider_kind · function · L41-L49 — pub(super) fn multi_account_provider_kind(
- account_usage_probe · function · L51-L56 — pub(super) fn account_usage_probe(
- same_provider_account_failover_enabled · function · L58-L62 — pub(super) fn same_provider_account_failover_enabled() -> bool
- active_account_label_for_provider · function · L64-L70 — pub(super) fn active_account_label_for_provider(provider: ActiveProvider) -> Option<String>
- same_provider_account_rotation · function · L72-L102 — fn same_provider_account_rotation(provider: ActiveProvider) -> Vec<String>
- same_provider_account_candidates · function · L104-L118 — pub(super) fn same_provider_account_candidates(provider: ActiveProvider) -> Vec<String>
- has_eligible_alternate_account · function · L121-L130 — pub fn has_eligible_alternate_account(provider: &str) -> bool
- has_alternate_account_in_pool · function · L135-L144 — pub fn has_alternate_account_in_pool(provider: &str) -> bool
- account_switch_guidance · function · L146-L162 — pub(super) fn account_switch_guidance(provider: ActiveProvider) -> Option<String>
- error_looks_like_usage_limit · function · L164-L183 — fn error_looks_like_usage_limit(summary: &str) -> bool
- maybe_annotate_limit_summary · function · L185-L196 — pub(super) fn maybe_annotate_limit_summary(provider: ActiveProvider, summary: String) -> String
