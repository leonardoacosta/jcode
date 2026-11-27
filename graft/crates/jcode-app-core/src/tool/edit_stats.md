# crates/jcode-app-core/src/tool/edit_stats.rs

- changed_lines · function · L6-L19 — fn changed_lines(old: &str, new: &str, approximate: bool) -> SessionEditStats
- line_delta · function · L23-L26 — pub(super) fn line_delta(old: &str, new: &str) -> (u64, u64)
- record · function · L28-L42 — pub(super) async fn record(ctx: &ToolContext, old: &str, new: &str, approximate: bool)
- tests · module · L45-L233 — mod tests
- exact_full_file_counts_cover_repetition_truncation_and_noops · function · L48-L69 — fn exact_full_file_counts_cover_repetition_truncation_and_noops()
- successful_tools_persist_exact_counts_and_failed_edits_do_not · function · L72-L232 — async fn successful_tools_persist_exact_counts_and_failed_edits_do_not()
- Home · struct · L77-L77 — struct Home(Option<std::ffi::OsString>);
- drop · function · L79-L85 — fn drop(&mut self)
