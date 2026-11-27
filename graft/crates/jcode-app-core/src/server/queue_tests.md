# crates/jcode-app-core/src/server/queue_tests.rs

- TestProvider · struct · L17-L17 — struct TestProvider;
- complete · function · L21-L31 — async fn complete(
- name · function · L33-L35 — fn name(&self) -> &str
- fork · function · L37-L39 — fn fork(&self) -> Arc<dyn Provider>
- test_agent · function · L42-L46 — async fn test_agent() -> Arc<Mutex<Agent>>
- queue_soft_interrupt_for_session_uses_registered_queue_when_agent_busy · function · L49-L85 — async fn queue_soft_interrupt_for_session_uses_registered_queue_when_agent_busy()
- queue_soft_interrupt_for_session_registers_queue_on_fallback_lookup · function · L88-L123 — async fn queue_soft_interrupt_for_session_registers_queue_on_fallback_lookup()
- queue_soft_interrupt_for_session_persists_when_live_queue_is_unavailable · function · L126-L185 — async fn queue_soft_interrupt_for_session_persists_when_live_queue_is_unavailable()
- file_activity_notices_fold_into_the_pending_one_until_delivered · function · L188-L263 — async fn file_activity_notices_fold_into_the_pending_one_until_delivered()
- settled_stop_discards_queued_soft_interrupts_live_and_persisted · function · L266-L308 — async fn settled_stop_discards_queued_soft_interrupts_live_and_persisted()
