# crates/jcode-base/src/recent_session_index/resume_tests.rs

- HomeGuard · struct · L3-L3 — struct HomeGuard(Option<std::ffi::OsString>);
- set · function · L6-L10 — fn set(path: &std::path::Path) -> Self
- drop · function · L14-L19 — fn drop(&mut self)
- session · function · L22-L26 — fn session(id: &str, title: &str) -> Session
- migrates_old_schema_without_treating_metadata_as_resume_entries · function · L29-L67 — fn migrates_old_schema_without_treating_metadata_as_resume_entries() -> Result<()>
- save_populates_resume_index_before_any_resume_lookup · function · L70-L85 — fn save_populates_resume_index_before_any_resume_lookup() -> Result<()>
- generic_metadata_upsert_invalidates_all_resume_fields · function · L88-L116 — fn generic_metadata_upsert_invalidates_all_resume_fields() -> Result<()>
- malformed_fingerprint_does_not_discard_other_resume_entries · function · L119-L146 — fn malformed_fingerprint_does_not_discard_other_resume_entries() -> Result<()>
- malformed_timestamp_does_not_discard_other_resume_entries · function · L149-L176 — fn malformed_timestamp_does_not_discard_other_resume_entries() -> Result<()>
- concurrent_schema_migrations_are_idempotent · function · L179-L211 — fn concurrent_schema_migrations_are_idempotent() -> Result<()>
- resume_title_precedence_excludes_todo_title · function · L214-L266 — fn resume_title_precedence_excludes_todo_title() -> Result<()>
- metadata_only_row_is_backfilled_by_resume_file_scan · function · L269-L290 — fn metadata_only_row_is_backfilled_by_resume_file_scan() -> Result<()>
- resume_timestamp_roundtrip_preserves_nanoseconds · function · L293-L308 — fn resume_timestamp_roundtrip_preserves_nanoseconds() -> Result<()>
- stale_backfill_cannot_overwrite_a_newer_save · function · L311-L329 — fn stale_backfill_cannot_overwrite_a_newer_save() -> Result<()>
