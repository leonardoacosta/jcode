# crates/jcode-tui/src/tui/app/debug_profile.rs

- runtime_memory_profile · function · L5-L7 — pub(in crate::tui::app) fn runtime_memory_profile(&self) -> serde_json::Value
- debug_memory_profile · function · L9-L11 — pub(in crate::tui::app) fn debug_memory_profile(&self) -> serde_json::Value
- memory_profile_value · function · L13-L173 — fn memory_profile_value(&self, include_history: bool) -> serde_json::Value
- debug_app_owned_memory_profile · function · L175-L514 — fn debug_app_owned_memory_profile(&self) -> serde_json::Value
- build_debug_summary · function · L517-L753 — fn build_debug_summary(payload: &serde_json::Value) -> serde_json::Value
- ratio · function · L755-L761 — fn ratio(numerator: usize, denominator: usize) -> f64
- estimate_pending_remote_message_bytes · function · L763-L771 — fn estimate_pending_remote_message_bytes(value: &PendingRemoteMessage) -> usize
- estimate_pending_split_prompt_bytes · function · L773-L775 — fn estimate_pending_split_prompt_bytes(value: &PendingSplitPrompt) -> usize
- estimate_pending_catchup_resume_bytes · function · L777-L784 — fn estimate_pending_catchup_resume_bytes(value: &PendingCatchupResume) -> usize
- estimate_rendered_images_bytes · function · L786-L799 — fn estimate_rendered_images_bytes(images: &[crate::session::RenderedImage]) -> usize
- nested_usize · function · L801-L813 — fn nested_usize(value: &serde_json::Value, path: &[&str]) -> usize
- nested_opt_u64 · function · L817-L823 — fn nested_opt_u64(value: &serde_json::Value, path: &[&str]) -> Option<u64>
- debug_summary_tests · module · L826-L897 — mod debug_summary_tests
- summary_explains_pss_with_process_buckets · function · L830-L880 — fn summary_explains_pss_with_process_buckets()
- summary_coverage_never_exceeds_one · function · L883-L896 — fn summary_coverage_never_exceeds_one()
