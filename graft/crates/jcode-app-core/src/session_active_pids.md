# crates/jcode-app-core/src/session_active_pids.rs

- active_pids_dir · function · L3-L5 — pub(super) fn active_pids_dir() -> Option<std::path::PathBuf>
- register_active_pid · function · L7-L12 — pub(super) fn register_active_pid(session_id: &str, pid: u32)
- unregister_active_pid · function · L14-L18 — pub(super) fn unregister_active_pid(session_id: &str)
- find_active_session_id_by_pid · function · L21-L32 — pub fn find_active_session_id_by_pid(pid: u32) -> Option<String>
- active_session_ids · function · L35-L47 — pub fn active_session_ids() -> Vec<String>
