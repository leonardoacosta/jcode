# crates/jcode-app-core/src/server/session_manager_tests.rs

- TestHome · struct · L7-L10 — struct TestHome
- new · function · L13-L21 — fn new() -> Self
- drop · function · L25-L31 — fn drop(&mut self)
- NoRequests · struct · L34-L34 — struct NoRequests;
- complete · function · L38-L46 — async fn complete(
- name · function · L47-L49 — fn name(&self) -> &str
- fork · function · L50-L52 — fn fork(&self) -> Arc<dyn Provider>
- agent · function · L55-L62 — async fn agent(session_id: &str) -> AgentHandle
- concurrent_get_or_start_runs_only_one_factory · function · L66-L85 — async fn concurrent_get_or_start_runs_only_one_factory() -> Result<()>
- failed_or_cancelled_factory_releases_reservation · function · L89-L125 — async fn failed_or_cancelled_factory_releases_reservation() -> Result<()>
- publication_never_overwrites_even_after_removal · function · L129-L169 — async fn publication_never_overwrites_even_after_removal() -> Result<()>
- reservations_are_per_manager_and_target · function · L172-L183 — async fn reservations_are_per_manager_and_target()
- rejected_rekey_keeps_both_original_owners · function · L187-L239 — async fn rejected_rekey_keeps_both_original_owners() -> Result<()>
- publication_rejects_wrong_session_identity · function · L243-L256 — async fn publication_rejects_wrong_session_identity() -> Result<()>
- observer_is_available_while_runtime_is_busy_and_retires_with_entry · function · L260-L287 — async fn observer_is_available_while_runtime_is_busy_and_retires_with_entry() -> Result<()>
- accepted_turn_blocks_restore_even_before_worker_locks_agent · function · L291-L313 — async fn accepted_turn_blocks_restore_even_before_worker_locks_agent() -> Result<()>
- failed_cold_restore_keeps_published_and_retiring_sources_unchanged · function · L317-L344 — async fn failed_cold_restore_keeps_published_and_retiring_sources_unchanged() -> Result<()>
