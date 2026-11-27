# crates/jcode-base/src/provider/failover.rs

- provider_is_configured · function · L5-L7 — pub(super) fn provider_is_configured(&self, provider: ActiveProvider) -> bool
- provider_precheck_unavailable_reason · function · L9-L22 — pub(super) fn provider_precheck_unavailable_reason(
- build_failover_prompt · function · L24-L42 — pub(super) fn build_failover_prompt(
- fallback_sequence · function · L44-L46 — pub(super) fn fallback_sequence(active: ActiveProvider) -> Vec<ActiveProvider>
- summarize_error · function · L48-L55 — pub(super) fn summarize_error(err: &anyhow::Error) -> String
- classify_failover_error · function · L57-L59 — pub(super) fn classify_failover_error(err: &anyhow::Error) -> FailoverDecision
- additional_no_provider_guidance · function · L61-L66 — pub(super) fn additional_no_provider_guidance(&self) -> Vec<String>
- no_provider_available_error · function · L68-L81 — pub(super) fn no_provider_available_error(&self, notes: &[String]) -> anyhow::Error
