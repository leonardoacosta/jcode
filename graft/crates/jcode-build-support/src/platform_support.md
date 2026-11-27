# crates/jcode-build-support/src/platform_support.rs

- LAUNCHER_BACKUP_PREFIX · constant · L10-L10 — const LAUNCHER_BACKUP_PREFIX: &str = ".jcode-launcher-old-";
- STALE_LAUNCHER_BACKUP_AGE · constant · L14-L14 — const STALE_LAUNCHER_BACKUP_AGE: Duration = Duration::from_secs(60 * 60);
- set_permissions_executable · function · L18-L30 — pub fn set_permissions_executable(path: &Path) -> std::io::Result<()>
- atomic_symlink_swap · function · L38-L103 — pub fn atomic_symlink_swap(src: &Path, dst: &Path, temp: &Path) -> std::io::Result<()>
- sweep_stale_launcher_backups · function · L111-L139 — fn sweep_stale_launcher_backups(dir: &Path, max_age: Duration)
- backup_is_stale · function · L146-L154 — fn backup_is_stale(file_name: &str, now: SystemTime, max_age: Duration) -> bool
- backup_operation_time · function · L158-L167 — fn backup_operation_time(file_name: &str) -> Option<SystemTime>
- tests · module · L170-L220 — mod tests
- windows_swap_replaces_existing_launcher_via_staged_file · function · L175-L192 — fn windows_swap_replaces_existing_launcher_via_staged_file()
- sweep_removes_only_old_launcher_backups · function · L195-L219 — fn sweep_removes_only_old_launcher_backups()
