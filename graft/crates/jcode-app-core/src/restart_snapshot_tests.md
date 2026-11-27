# crates/jcode-app-core/src/restart_snapshot_tests.rs

- TestEnvGuard · struct · L9-L13 — struct TestEnvGuard
- new · function · L16-L28 — fn new() -> anyhow::Result<Self>
- drop · function · L32-L38 — fn drop(&mut self)
- capture_current_snapshot_includes_active_sessions_only · function · L42-L58 — fn capture_current_snapshot_includes_active_sessions_only()
- save_and_load_snapshot_round_trip · function · L61-L74 — fn save_and_load_snapshot_round_trip()
- set_auto_restore_updates_saved_snapshot · function · L77-L88 — fn set_auto_restore_updates_saved_snapshot()
- clear_snapshot_removes_saved_file · function · L91-L101 — fn clear_snapshot_removes_saved_file()
- arm_auto_restore_from_recent_crashes_captures_dead_active_sessions · function · L104-L141 — fn arm_auto_restore_from_recent_crashes_captures_dead_active_sessions()
- arm_auto_restore_from_recent_crashes_ignores_old_crashes · function · L144-L179 — fn arm_auto_restore_from_recent_crashes_ignores_old_crashes()
