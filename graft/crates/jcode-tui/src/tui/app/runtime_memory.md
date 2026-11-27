# crates/jcode-tui/src/tui/app/runtime_memory.rs

- note_runtime_memory_event · function · L4-L6 — pub(super) fn note_runtime_memory_event(&mut self, category: &str, reason: &str)
- note_runtime_memory_event_force · function · L8-L10 — pub(super) fn note_runtime_memory_event_force(&mut self, category: &str, reason: &str)
- note_runtime_memory_event_impl · function · L12-L99 — fn note_runtime_memory_event_impl(&mut self, category: &str, reason: &str, force: bool)
- maybe_capture_runtime_memory_heartbeat · function · L101-L184 — pub(super) fn maybe_capture_runtime_memory_heartbeat(&mut self)
- capture_runtime_memory_process_sample · function · L186-L215 — fn capture_runtime_memory_process_sample(
- capture_runtime_memory_attribution_sample · function · L217-L256 — fn capture_runtime_memory_attribution_sample(
- runtime_memory_client_info · function · L258-L269 — fn runtime_memory_client_info(&self) -> crate::runtime_memory_log::ClientRuntimeMemoryClient
- append_runtime_memory_sample · function · L271-L283 — fn append_runtime_memory_sample(
- client_runtime_totals_from_profile · function · L286-L505 — fn client_runtime_totals_from_profile(
- nested_u64 · function · L507-L516 — fn nested_u64(value: &serde_json::Value, path: &[&str]) -> u64
- tests · module · L519-L580 — mod tests
- client_runtime_totals_include_ui_render_side_panel_render_and_remote_images · function · L523-L579 — fn client_runtime_totals_include_ui_render_side_panel_render_and_remote_images()
