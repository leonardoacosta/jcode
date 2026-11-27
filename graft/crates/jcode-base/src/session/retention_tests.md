# crates/jcode-base/src/session/retention_tests.rs

- policy · function · L4-L9 — fn policy() -> SessionRetentionConfig
- meta · function · L11-L21 — fn meta(saved: bool, debug: bool, msgs: usize, live: bool, age_days: i64) -> SessionMeta
- saved_debug_tiny_old_is_kept · function · L24-L27 — fn saved_debug_tiny_old_is_kept()
- old_debug_is_candidate · function · L30-L33 — fn old_debug_is_candidate()
- recent_debug_is_kept · function · L36-L39 — fn recent_debug_is_kept()
- old_tiny_is_candidate_but_big_old_kept · function · L42-L52 — fn old_tiny_is_candidate_but_big_old_kept()
- unparseable_is_kept · function · L55-L64 — fn unparseable_is_kept()
- live_is_kept · function · L67-L70 — fn live_is_kept()
- temp_dir · function · L72-L83 — fn temp_dir() -> PathBuf
- write_session · function · L85-L95 — fn write_session(dir: &Path, id: &str, saved: bool, debug: bool)
- gc_dry_run_changes_nothing_and_apply_removes_only_candidates · function · L98-L126 — fn gc_dry_run_changes_nothing_and_apply_removes_only_candidates()
