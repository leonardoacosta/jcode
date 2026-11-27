# crates/jcode-app-core/src/restart_snapshot.rs

- AUTO_RESTORE_CRASH_MAX_AGE_HOURS · constant · L7-L7 — const AUTO_RESTORE_CRASH_MAX_AGE_HOURS: i64 = 24;
- RestartSnapshot · struct · L10-L16 — pub struct RestartSnapshot
- RestartSnapshotSession · struct · L19-L26 — pub struct RestartSnapshotSession
- RestoreLaunchOutcome · struct · L29-L33 — pub struct RestoreLaunchOutcome
- RestoreSnapshotResult · struct · L36-L39 — pub struct RestoreSnapshotResult
- snapshot_path · function · L41-L43 — pub fn snapshot_path() -> Result<PathBuf>
- save_current_snapshot · function · L45-L49 — pub fn save_current_snapshot() -> Result<RestartSnapshot>
- write_snapshot · function · L51-L53 — pub fn write_snapshot(snapshot: &RestartSnapshot) -> Result<()>
- load_snapshot · function · L55-L57 — pub fn load_snapshot() -> Result<RestartSnapshot>
- clear_snapshot · function · L59-L66 — pub fn clear_snapshot() -> Result<bool>
- set_auto_restore_on_next_start · function · L68-L76 — pub fn set_auto_restore_on_next_start(enabled: bool) -> Result<bool>
- arm_auto_restore_from_recent_crashes · function · L78-L134 — pub fn arm_auto_restore_from_recent_crashes() -> Result<Option<RestartSnapshot>>
- capture_current_snapshot · function · L136-L182 — pub fn capture_current_snapshot() -> Result<RestartSnapshot>
- restore_snapshot · function · L184-L216 — pub fn restore_snapshot(exe: &Path) -> Result<RestoreSnapshotResult>
- resolve_session_cwd · function · L218-L224 — fn resolve_session_cwd(configured: Option<&str>) -> PathBuf
- shell_escape · function · L226-L228 — fn shell_escape(text: &str) -> String
- restore_command_display · function · L230-L237 — pub fn restore_command_display(exe: &Path, session: &RestartSnapshotSession) -> String
- restart_snapshot_tests · module · L241-L241 — mod restart_snapshot_tests;
