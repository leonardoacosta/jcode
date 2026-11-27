# crates/jcode-provider-openai-runtime/src/openai_retry_diagnostics.rs

- OpenAIRetryDiagnostics · struct · L7-L9 — pub(super) struct OpenAIRetryDiagnostics
- record · function · L14-L30 — pub(super) fn record(&mut self, error: &str) -> Option<&'static str>
- annotate · function · L34-L42 — pub(super) fn annotate(&self, error: anyhow::Error) -> anyhow::Error
- tests · module · L46-L97 — mod tests
- overload_notice_is_explicit_and_deduplicated · function · L50-L56 — fn overload_notice_is_explicit_and_deduplicated()
- generic_failures_do_not_invent_overload · function · L59-L74 — fn generic_failures_do_not_invent_overload()
- overload_message_without_code_is_not_given_an_invented_code · function · L77-L82 — fn overload_message_without_code_is_not_given_an_invented_code()
- later_timeout_keeps_both_facts_and_error_chain · function · L85-L96 — fn later_timeout_keeps_both_facts_and_error_chain()
