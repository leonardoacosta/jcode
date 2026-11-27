# crates/jcode-app-core/src/server/client_processing.rs

- reconcile_processing_completions · function · L7-L57 — pub(super) async fn reconcile_processing_completions(
- record_processing_completion · function · L59-L122 — pub(super) async fn record_processing_completion(
- append_context_message · function · L124-L170 — pub(super) async fn append_context_message(
- delivered_replay_is_reminder_only · function · L175-L177 — fn delivered_replay_is_reminder_only(content: &str, image_count: usize) -> bool
- start_processing_message · function · L180-L512 — pub(super) async fn start_processing_message(
- cancel_processing_message · function · L514-L754 — pub(super) async fn cancel_processing_message(
- try_available_models_snapshot · function · L756-L762 — pub(super) async fn try_available_models_snapshot(agent: &Arc<Mutex<Agent>>) -> Option<Arc<str>>
- queue_soft_interrupt · function · L764-L793 — pub(super) fn queue_soft_interrupt(
- cancel_soft_interrupts_by_id · function · L795-L823 — pub(super) fn cancel_soft_interrupts_by_id(
- clear_soft_interrupts · function · L825-L851 — pub(super) fn clear_soft_interrupts(
- move_tool_to_background · function · L853-L883 — pub(super) fn move_tool_to_background(
- delivered_replay_tests · module · L886-L903 — mod delivered_replay_tests
- reminder_only_replay_is_suppressed · function · L890-L893 — fn reminder_only_replay_is_suppressed()
- queued_followup_with_delivered_reminder_is_not_suppressed · function · L896-L902 — fn queued_followup_with_delivered_reminder_is_not_suppressed()
