# crates/jcode-tui/src/tui/remote_diff.rs

- PendingFileDiff · struct · L7-L10 — pub(crate) struct PendingFileDiff
- RemoteDiffTracker · struct · L13-L18 — pub(crate) struct RemoteDiffTracker
- handle_tool_start · function · L21-L24 — pub(crate) fn handle_tool_start(&mut self, id: &str, _name: &str)
- handle_tool_input · function · L26-L37 — pub(crate) fn handle_tool_input(&mut self, id: Option<&str>, delta: &str)
- tool_input_json · function · L39-L44 — pub(crate) fn tool_input_json(&self, id: &str) -> Value
- handle_tool_exec · function · L46-L73 — pub(crate) fn handle_tool_exec(&mut self, id: &str, name: &str)
- take_tool_duration · function · L75-L79 — pub(crate) fn take_tool_duration(&mut self, id: &str) -> Option<f32>
- finish_tool · function · L81-L94 — pub(crate) fn finish_tool(&mut self, id: &str, name: &str, output: &str) -> String
- clear · function · L96-L101 — pub(crate) fn clear(&mut self)
- show_diffs_enabled · function · L105-L109 — pub(crate) fn show_diffs_enabled() -> bool
- resolve_diff_path · function · L113-L129 — pub(crate) fn resolve_diff_path(raw: &str) -> PathBuf
- generate_unified_diff · function · L132-L144 — pub(crate) fn generate_unified_diff(old: &str, new: &str, file_path: &str) -> String
- keyed_tool_tests · module · L147-L230 — mod keyed_tool_tests
- keyed_tool_inputs_survive_sibling_exec_and_preserve_legacy_fallback · function · L151-L167 — fn keyed_tool_inputs_survive_sibling_exec_and_preserve_legacy_fallback()
- keyed_tool_inputs_snapshot_the_matching_file_for_each_diff · function · L170-L193 — fn keyed_tool_inputs_snapshot_the_matching_file_for_each_diff()
- tool_duration_tracks_execution_not_input_and_clears_state · function · L196-L215 — fn tool_duration_tracks_execution_not_input_and_clears_state()
- keyed_tool_inputs_clear_discards_all_partial_calls · function · L218-L229 — fn keyed_tool_inputs_clear_discards_all_partial_calls()
