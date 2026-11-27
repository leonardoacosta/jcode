# crates/jcode-tui/src/tui/app/remote/queue_recovery.rs

- SOFT_INTERRUPT_RECOVERY_TIMEOUT · constant · L5-L5 — const SOFT_INTERRUPT_RECOVERY_TIMEOUT: Duration = Duration::from_secs(5);
- track_pending_soft_interrupt · function · L8-L31 — pub(in crate::tui::app) fn track_pending_soft_interrupt(
- acknowledge_pending_soft_interrupt · function · L33-L58 — pub(super) fn acknowledge_pending_soft_interrupt(&mut self, request_id: u64) -> bool
- clear_pending_soft_interrupt_tracking · function · L60-L70 — pub(super) fn clear_pending_soft_interrupt_tracking(&mut self)
- forget_sent_soft_interrupt · function · L72-L80 — fn forget_sent_soft_interrupt(&mut self, content: &str)
- pending_soft_interrupt_client_ids · function · L84-L99 — fn pending_soft_interrupt_client_ids(&self) -> Option<Vec<String>>
- soft_interrupt_recovery_in_flight · function · L104-L107 — pub(super) fn soft_interrupt_recovery_in_flight(&self) -> bool
- finish_soft_interrupt_recovery · function · L112-L184 — pub(super) fn finish_soft_interrupt_recovery(
- mark_soft_interrupt_injected · function · L186-L214 — pub(super) fn mark_soft_interrupt_injected(&mut self, content: &str)
- mark_combined_soft_interrupt_injected · function · L216-L246 — fn mark_combined_soft_interrupt_injected(&mut self, content: &str) -> bool
- recover_undelivered_queued_continuation · function · L261-L263 — pub(super) fn recover_undelivered_queued_continuation(app: &mut App, reason: &str) -> bool
- recover_rejected_queued_continuation · function · L268-L270 — pub(super) fn recover_rejected_queued_continuation(app: &mut App) -> bool
- recover_queued_continuation · function · L272-L310 — fn recover_queued_continuation(app: &mut App, reason: &str, rejected: bool) -> bool
- recover_local_interleave_to_queue · function · L312-L331 — pub(super) fn recover_local_interleave_to_queue(app: &mut App, reason: &str) -> bool
- recover_stranded_soft_interrupts · function · L333-L427 — pub(super) async fn recover_stranded_soft_interrupts(
