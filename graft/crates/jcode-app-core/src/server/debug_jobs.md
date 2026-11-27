# crates/jcode-app-core/src/server/debug_jobs.rs

- serialization_ownership_tests · module · L11-L96 — mod serialization_ownership_tests
- debug_job_payload_preserves_legacy_fields_and_optional_values · function · L15-L63 — fn debug_job_payload_preserves_legacy_fields_and_optional_values()
- debug_job_large_output_keeps_source_and_result_independent · function · L66-L95 — fn debug_job_large_output_keeps_source_and_result_independent()
- DebugJobStatus · enum · L99-L104 — pub(super) enum DebugJobStatus
- as_str · function · L107-L114 — pub(super) fn as_str(&self) -> &'static str
- DebugJob · struct · L118-L128 — pub(super) struct DebugJob
- summary_payload · function · L131-L148 — pub(super) fn summary_payload(&self) -> Value
- status_payload · function · L150-L157 — pub(super) fn status_payload(&self) -> Value
- maybe_start_async_debug_job · function · L160-L230 — pub(super) async fn maybe_start_async_debug_job(
- maybe_handle_job_command · function · L232-L349 — pub(super) async fn maybe_handle_job_command(
- create_job · function · L351-L379 — async fn create_job(
- mark_job_running · function · L381-L387 — async fn mark_job_running(debug_jobs: &Arc<RwLock<HashMap<String, DebugJob>>>, job_id: &str)
- finish_job · function · L389-L412 — async fn finish_job(
