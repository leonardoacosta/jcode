# crates/jcode-app-core/src/agent_tests/concurrency.rs

- IsolatedEnv · struct · L6-L10 — struct IsolatedEnv
- new · function · L13-L24 — fn new() -> Self
- drop · function · L28-L39 — fn drop(&mut self)
- assert_owns_current_session · function · L42-L49 — fn assert_owns_current_session(agent: &Agent)
- concurrency_guard_follows_clear_restore_and_close · function · L52-L95 — async fn concurrency_guard_follows_clear_restore_and_close()
- concurrency_guards_belong_to_each_agent_not_the_global_telemetry_slot · function · L98-L114 — async fn concurrency_guards_belong_to_each_agent_not_the_global_telemetry_slot()
- deferred_restore_save_persists_when_called · function · L120-L161 — async fn deferred_restore_save_persists_when_called()
