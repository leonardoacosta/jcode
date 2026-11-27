# crates/jcode-app-core/src/agent/compaction.rs

- checkpoint_idle_runtime_upgrade · function · L6-L23 — pub(crate) fn checkpoint_idle_runtime_upgrade(&mut self) -> Result<()>
- note_compaction_applied · function · L25-L31 — pub(super) fn note_compaction_applied(&mut self)
- poll_compaction_completion_event · function · L33-L53 — pub fn poll_compaction_completion_event(&mut self) -> Option<CompactionEvent>
- request_manual_compaction · function · L55-L115 — pub fn request_manual_compaction(&mut self) -> (String, bool)
- is_context_limit_error · function · L117-L131 — fn is_context_limit_error(error: &str) -> bool
- try_auto_compact_after_context_limit · function · L137-L220 — pub(super) fn try_auto_compact_after_context_limit(&mut self, error: &str) -> bool
- take_recovery_compaction_event · function · L224-L237 — pub(super) fn take_recovery_compaction_event(&mut self) -> crate::protocol::ServerEvent
- try_recover_after_payload_too_large · function · L246-L277 — fn try_recover_after_payload_too_large(&mut self, error: &str) -> bool
- try_recover_after_image_rejection · function · L284-L311 — fn try_recover_after_image_rejection(&mut self, error: &str) -> bool
- reset_after_transcript_image_rewrite · function · L315-L335 — fn reset_after_transcript_image_rewrite(&mut self)
- try_recover_oversized_openai_native_compaction · function · L337-L378 — fn try_recover_oversized_openai_native_compaction(&mut self) -> bool
- effective_context_tokens_from_usage · function · L380-L395 — pub(super) fn effective_context_tokens_from_usage(
- update_compaction_usage_from_stream · function · L397-L416 — pub(super) fn update_compaction_usage_from_stream(
- push_embedding_snapshot_if_semantic · function · L421-L437 — pub(super) fn push_embedding_snapshot_if_semantic(&mut self, text: &str)
