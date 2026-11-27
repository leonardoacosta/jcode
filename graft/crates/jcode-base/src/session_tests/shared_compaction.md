# crates/jcode-base/src/session_tests/shared_compaction.rs

- large_state · function · L6-L16 — fn large_state(tag: &str) -> StoredCompactionState
- fixture · function · L18-L23 — fn fixture() -> Session
- assert_metadata_shared · function · L25-L44 — fn assert_metadata_shared(session: &Session)
- reset_and_clone_retain_one_unique_encrypted_payload_until_mutation · function · L47-L141 — fn reset_and_clone_retain_one_unique_encrypted_payload_until_mutation()
- redacted_export_cannot_mutate_original_or_persisted_snapshot · function · L144-L174 — fn redacted_export_cannot_mutate_original_or_persisted_snapshot()
- arc_compaction_json_is_byte_identical_and_reads_owned_snapshot · function · L177-L213 — fn arc_compaction_json_is_byte_identical_and_reads_owned_snapshot() -> anyhow::Result<()>
- journal_replay_distinguishes_unchanged_changed_and_explicit_clear · function · L216-L251 — fn journal_replay_distinguishes_unchanged_changed_and_explicit_clear() -> anyhow::Result<()>
- HomeGuard · struct · L253-L253 — struct HomeGuard(Option<std::ffi::OsString>);
- drop · function · L255-L261 — fn drop(&mut self)
- shared_compaction_retention_microprobe · function · L268-L311 — fn shared_compaction_retention_microprobe()
- save_and_load_share_metadata_and_persist_compaction_transitions · function · L314-L402 — fn save_and_load_share_metadata_and_persist_compaction_transitions() -> anyhow::Result<()>
