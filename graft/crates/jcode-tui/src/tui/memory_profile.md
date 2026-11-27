# crates/jcode-tui/src/tui/memory_profile.rs

- LARGE_DISPLAY_BLOB_THRESHOLD_BYTES · constant · L7-L7 — const LARGE_DISPLAY_BLOB_THRESHOLD_BYTES: usize = 16 * 1024;
- build_transcript_memory_profile · function · L9-L132 — pub fn build_transcript_memory_profile(
- estimate_display_message_bytes · function · L134-L152 — pub fn estimate_display_message_bytes(message: &DisplayMessage) -> usize
- nested_usize · function · L154-L166 — fn nested_usize(value: &serde_json::Value, path: &[&str]) -> usize
- ProviderMessageMemoryStats · struct · L169-L180 — struct ProviderMessageMemoryStats
- record_bytes · function · L183-L189 — fn record_bytes(&mut self, bytes: usize)
- record_message · function · L191-L251 — fn record_message(&mut self, message: &Message)
- payload_text_bytes · function · L253-L259 — fn payload_text_bytes(&self) -> usize
- DisplayMessageMemoryStats · struct · L263-L276 — struct DisplayMessageMemoryStats
- record_message · function · L279-L306 — fn record_message(&mut self, message: &DisplayMessage)
- chrome_text_bytes · function · L308-L310 — fn chrome_text_bytes(&self) -> usize
- SidePanelMemoryStats · struct · L314-L322 — struct SidePanelMemoryStats
- estimate_side_panel_memory · function · L324-L358 — fn estimate_side_panel_memory(snapshot: &SidePanelSnapshot) -> SidePanelMemoryStats
- tests · module · L361-L471 — mod tests
- pdf_payload_counts_toward_side_panel_memory · function · L367-L379 — fn pdf_payload_counts_toward_side_panel_memory()
- transcript_memory_profile_breaks_out_provider_display_and_side_panel · function · L382-L470 — fn transcript_memory_profile_breaks_out_provider_display_and_side_panel()
