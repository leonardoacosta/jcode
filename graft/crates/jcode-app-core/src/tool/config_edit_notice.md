# crates/jcode-app-core/src/tool/config_edit_notice.rs

- comparable · function · L17-L22 — fn comparable(path: &Path) -> std::path::PathBuf
- is_active_config_file · function · L28-L33 — fn is_active_config_file(path: &Path) -> bool
- is_config_source_file · function · L40-L42 — pub(super) fn is_config_source_file(path: &Path) -> bool
- is_active_fragment · function · L45-L47 — fn is_active_fragment(path: &Path) -> bool
- fragment_position · function · L50-L57 — fn fragment_position(path: &Path) -> Option<usize>
- winning_keys · function · L65-L81 — fn winning_keys(index: usize) -> std::collections::HashSet<String>
- config_edit_notice · function · L87-L129 — pub fn config_edit_notice(path: &Path, before: &str, after: &str) -> Option<String>
- append_config_edit_notice · function · L132-L136 — pub fn append_config_edit_notice(body: &mut String, path: &Path, before: &str, after: &str)
- read_config_text · function · L143-L145 — fn read_config_text(path: &Path) -> String
- ConfigEditWatch · struct · L154-L157 — pub struct ConfigEditWatch
- begin · function · L161-L173 — pub fn begin() -> Self
- finish · function · L180-L195 — pub fn finish(self, body: &mut String)
- tests · module · L200-L200 — mod tests;
