# crates/jcode-app-core/src/agent/turn_execution_tests.rs

- plan_mode_nonempty_retry_does_not_reset_judge_budget · function · L6-L31 — async fn plan_mode_nonempty_retry_does_not_reset_judge_budget()
- judge_feedback_budget_resets_only_for_genuine_user_turns · function · L34-L108 — async fn judge_feedback_budget_resets_only_for_genuine_user_turns()
- MockProvider · struct · L111-L113 — struct MockProvider
- complete · function · L117-L131 — async fn complete(
- name · function · L132-L134 — fn name(&self) -> &str
- model · function · L135-L137 — fn model(&self) -> String
- fork · function · L138-L140 — fn fork(&self) -> Arc<dyn Provider>
- mock_agent · function · L143-L152 — async fn mock_agent() -> (Agent, Arc<AtomicUsize>)
- texts · function · L154-L166 — fn texts(agent: &Agent, role: Role) -> Vec<String>
- run_once_capture_returns_streamed_text_and_records_turn · function · L169-L185 — async fn run_once_capture_returns_streamed_text_and_records_turn()
- append_user_context_message_persists_without_model_call · function · L188-L203 — async fn append_user_context_message_persists_without_model_call()
- continue_turn_after_failover_runs_streaming_turn · function · L206-L229 — async fn continue_turn_after_failover_runs_streaming_turn()
- continue_turn_after_failover_noops_on_graceful_shutdown · function · L232-L245 — async fn continue_turn_after_failover_noops_on_graceful_shutdown()
- fresh_session_has_valid_lease_and_is_not_stale · function · L248-L253 — async fn fresh_session_has_valid_lease_and_is_not_stale()
- every_agent_entry_point_refuses_stopped_session_without_new_generation · function · L256-L288 — async fn every_agent_entry_point_refuses_stopped_session_without_new_generation()
