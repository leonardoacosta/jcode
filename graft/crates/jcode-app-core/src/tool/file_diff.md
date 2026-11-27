# crates/jcode-app-core/src/tool/file_diff.rs

- snapshot · function · L7-L13 — pub(super) async fn snapshot(path: &std::path::Path) -> Option<(bool, String)>
- unified · function · L15-L57 — pub(super) fn unified(old_path: &str, new_path: &str, old: &str, new: &str) -> String
- header · function · L18-L46 — fn header(path: &str, prefix: &str) -> String
- attach · function · L59-L74 — pub(super) fn attach(mut output: ToolOutput, diff: String) -> ToolOutput
- tests · module · L77-L270 — mod tests
- whole_file_positions_and_disjoint_hunks · function · L81-L90 — fn whole_file_positions_and_disjoint_hunks()
- creation_deletion_eof_and_noop · function · L93-L104 — fn creation_deletion_eof_and_noop()
- text_and_metadata_share_exact_diff · function · L107-L114 — fn text_and_metadata_share_exact_diff()
- authoritative_diffs_cover_successful_tool_mutations · function · L117-L269 — async fn authoritative_diffs_cover_successful_tool_mutations()
- Home · struct · L122-L122 — struct Home(Option<std::ffi::OsString>);
- drop · function · L124-L130 — fn drop(&mut self)
