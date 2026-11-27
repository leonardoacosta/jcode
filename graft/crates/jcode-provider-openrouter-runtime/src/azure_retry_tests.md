# crates/jcode-provider-openrouter-runtime/src/azure_retry_tests.rs

- COMPLETE · constant · L8-L11 — const COMPLETE: &str = concat!(
- Server · struct · L13-L17 — struct Server
- new · function · L20-L78 — fn new(bodies: Vec<String>) -> Self
- finish · function · L80-L83 — fn finish(mut self) -> Vec<Instant>
- drop · function · L87-L92 — fn drop(&mut self)
- run_azure · function · L95-L130 — fn run_azure(base: String) -> Vec<Result<StreamEvent>>
- azure_clean_eof_retries_with_rollback_before_replayed_output · function · L133-L177 — fn azure_clean_eof_retries_with_rollback_before_replayed_output()
- azure_permanent_structured_failure_is_not_retried_and_keeps_hint · function · L180-L194 — fn azure_permanent_structured_failure_is_not_retried_and_keeps_hint()
- azure_transient_structured_failure_honors_retry_after · function · L197-L213 — fn azure_transient_structured_failure_honors_retry_after()
- azure_unknown_failure_does_not_inherit_stream_error_retry_heuristic · function · L216-L228 — fn azure_unknown_failure_does_not_inherit_stream_error_retry_heuristic()
