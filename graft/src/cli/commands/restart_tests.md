# src/cli/commands/restart_tests.rs

- TestEnvGuard · struct · L8-L14 — struct TestEnvGuard
- new · function · L17-L35 — fn new() -> anyhow::Result<Self>
- drop · function · L39-L55 — fn drop(&mut self)
- restart_save_writes_empty_snapshot_with_auto_restore_flag · function · L59-L69 — async fn restart_save_writes_empty_snapshot_with_auto_restore_flag()
- pending_restore_returns_false_for_unarmed_snapshot · function · L72-L85 — async fn pending_restore_returns_false_for_unarmed_snapshot()
- pending_restore_does_not_auto_restore_recent_crash_without_snapshot · function · L88-L113 — async fn pending_restore_does_not_auto_restore_recent_crash_without_snapshot()
- restart_clear_removes_saved_snapshot · function · L116-L125 — async fn restart_clear_removes_saved_snapshot()
