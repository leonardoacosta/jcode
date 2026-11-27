# crates/jcode-provider-openai-runtime/src/openai_ws_cancel_recovery.rs

- CANCEL_DRAIN_TIMEOUT · constant · L8-L8 — const CANCEL_DRAIN_TIMEOUT: Duration = Duration::from_secs(10);
- RecoveryPhase · enum · L11-L15 — pub(super) enum RecoveryPhase
- has_unanswered_tool_calls · function · L21-L36 — pub(super) fn has_unanswered_tool_calls(output: &[Value], input: &[Value]) -> bool
- RecoverableAttempt · struct · L40-L44 — pub(super) struct RecoverableAttempt
- drop · function · L47-L93 — fn drop(&mut self)
- drain · function · L96-L129 — async fn drain(state: &mut PersistentWsState) -> bool
- tests · module · L132-L348 — mod tests
- completed_but_unseen_tools_require_full_replay · function · L136-L157 — fn completed_but_unseen_tools_require_full_replay()
- pair · function · L159-L184 — async fn pair() -> (
- attempt · function · L186-L200 — fn attempt(
- cancellation_drain_rejects_errors_malformed_frames_and_budgets · function · L203-L251 — async fn cancellation_drain_rejects_errors_malformed_frames_and_budgets()
- cancellation_drain_answers_ping_and_clears_entire_response_chain · function · L254-L287 — async fn cancellation_drain_answers_ping_and_clears_entire_response_chain()
- cancellation_retains_affinity_when_completion_takes_over_half_a_second · function · L290-L317 — async fn cancellation_retains_affinity_when_completion_takes_over_half_a_second()
- cancellation_after_terminal_consumed_does_not_drain_next_response · function · L320-L329 — async fn cancellation_after_terminal_consumed_does_not_drain_next_response()
- cancellation_guard_can_be_dropped_after_runtime_shutdown · function · L332-L347 — fn cancellation_guard_can_be_dropped_after_runtime_shutdown()
