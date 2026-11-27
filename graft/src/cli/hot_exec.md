# src/cli/hot_exec.rs

- has_requested_action · function · L8-L14 — pub fn has_requested_action(run_result: &RunResult) -> bool
- execute_requested_action · function · L16-L41 — pub fn execute_requested_action(run_result: &RunResult) -> Result<()>
- hot_restart · function · L43-L60 — pub fn hot_restart(session_id: &str) -> Result<()>
- hot_reload · function · L62-L145 — pub fn hot_reload(session_id: &str) -> Result<()>
- hot_update · function · L147-L233 — pub fn hot_update(session_id: &str) -> Result<()>
- get_repo_dir · function · L235-L237 — pub fn get_repo_dir() -> Option<std::path::PathBuf>
- UPDATE_FETCH_INTERVAL_SECS · constant · L244-L244 — const UPDATE_FETCH_INTERVAL_SECS: u64 = 15 * 60;
- claim_update_fetch_slot · function · L246-L261 — fn claim_update_fetch_slot() -> bool
- check_for_updates · function · L264-L268 — pub fn check_for_updates() -> Result<Option<bool>>
- check_for_updates_in · function · L270-L293 — pub(super) fn check_for_updates_in(
- source_update_git · function · L295-L311 — fn source_update_git(repo_dir: &std::path::Path, args: &[&str]) -> Result<String>
- local_commits_ahead_of_upstream · function · L316-L331 — pub fn local_commits_ahead_of_upstream() -> Option<bool>
- run_auto_update · function · L333-L409 — pub fn run_auto_update() -> Result<()>
- run_update · function · L413-L461 — pub fn run_update(channel: Option<&str>) -> Result<()>
- repair_stale_shared_server_after_update_check · function · L463-L488 — fn repair_stale_shared_server_after_update_check() -> bool
- reload_server_after_update · function · L490-L524 — fn reload_server_after_update(reason: &str)
