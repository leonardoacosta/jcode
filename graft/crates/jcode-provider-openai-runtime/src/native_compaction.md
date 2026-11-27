# crates/jcode-provider-openai-runtime/src/native_compaction.rs

- compaction_usage · function · L5-L26 — fn compaction_usage(body: &Value) -> Option<jcode_provider_core::SimpleCompletionUsage>
- native_compact_request · function · L29-L120 — pub(super) async fn native_compact_request(
- parse_compaction_response · function · L123-L151 — fn parse_compaction_response(body: &Value) -> Result<jcode_provider_core::NativeCompactionResult>
- tests · module · L154-L239 — mod tests
- native_compaction_missing_artifact_preserves_observed_usage · function · L158-L168 — fn native_compaction_missing_artifact_preserves_observed_usage()
- native_compaction_returns_usage_even_for_oversized_artifacts · function · L171-L186 — fn native_compaction_returns_usage_even_for_oversized_artifacts()
- native_compaction_uses_only_explicit_response_model · function · L189-L210 — fn native_compaction_uses_only_explicit_response_model()
- native_compaction_preserves_inclusive_input_and_sparse_usage · function · L213-L238 — fn native_compaction_preserves_inclusive_input_and_sparse_usage()
