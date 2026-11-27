# crates/jcode-base/src/session/maintenance.rs

- BACKUP_RETENTION_DAYS · constant · L23-L23 — const BACKUP_RETENTION_DAYS: i64 = 30;
- PRUNE_INTERVAL_SECS · constant · L32-L32 — const PRUNE_INTERVAL_SECS: u64 = 24 * 60 * 60;
- prune_old_session_backups · function · L40-L48 — pub fn prune_old_session_backups()
- claim_prune_slot · function · L56-L67 — fn claim_prune_slot(base: &Path) -> bool
- prune_old_session_backups_in · function · L71-L91 — fn prune_old_session_backups_in(sessions_dir: &Path, now: DateTime<Local>)
- tests · module · L94-L184 — mod tests
- claim_prune_slot_rate_limits_within_interval_and_reclaims_after · function · L101-L136 — fn claim_prune_slot_rate_limits_within_interval_and_reclaims_after()
- prunes_only_old_bak_files · function · L139-L183 — fn prunes_only_old_bak_files()
