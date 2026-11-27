# crates/jcode-app-core/src/server/history_timing.rs

- NEXT_OPERATION · constant · L4-L4 — static NEXT_OPERATION: AtomicU64 = AtomicU64::new(1);
- Timing · struct · L7-L16 — pub(super) struct Timing
- new · function · L19-L34 — pub(super) fn new(kind: &'static str, request_id: Option<u64>, session_id: &str) -> Self
- fields · function · L36-L49 — fn fields(&self, stage: &str, outcome: &str, duration_ms: u128) -> String
- stage · function · L51-L59 — pub(super) fn stage(&mut self, stage: &'static str)
- fallback · function · L61-L73 — fn fallback<T: Default, E>(&self, result: std::result::Result<T, E>) -> T
- result · function · L75-L77 — pub(super) fn result<T, E>(&mut self, result: &std::result::Result<T, E>)
- drop · function · L81-L87 — fn drop(&mut self)
- send_resume_todos · function · L91-L116 — pub(in crate::server) async fn send_resume_todos(
- tests · module · L119-L177 — mod tests
- resume_todos_reports_disconnected_writer · function · L123-L133 — async fn resume_todos_reports_disconnected_writer()
- stages_keep_correlation_and_do_not_mark_cancellation_successful · function · L136-L154 — fn stages_keep_correlation_and_do_not_mark_cancellation_successful()
- timing_fields_are_correlated_numeric_and_content_free · function · L157-L176 — fn timing_fields_are_correlated_numeric_and_content_free()
