# crates/jcode-app-core/src/mobile_push_summary.rs

- cached_regex · function · L3-L16 — pub(super) fn cached_regex(
- plain_inline · function · L19-L36 — pub(super) fn plain_inline(text: &str) -> String
- LINK · constant · L20-L20 — static LINK: std::sync::OnceLock<Option<regex::Regex>> = std::sync::OnceLock::new();
- EMPH · constant · L21-L21 — static EMPH: std::sync::OnceLock<Option<regex::Regex>> = std::sync::OnceLock::new();
- plain_lines · function · L40-L78 — pub(super) fn plain_lines(text: &str) -> Vec<String>
- strip_list_marker · function · L80-L94 — pub(super) fn strip_list_marker(line: &str) -> &str
- summarize_answer · function · L98-L111 — pub(super) fn summarize_answer(text: &str) -> String
- join_sentences · function · L113-L125 — pub(super) fn join_sentences<'a>(parts: impl Iterator<Item = &'a str>) -> String
- trailing_question · function · L128-L146 — pub(super) fn trailing_question(text: &str) -> Option<String>
- summarize_error · function · L150-L199 — pub(super) fn summarize_error(text: &str) -> String
- JSON · constant · L158-L158 — static JSON: std::sync::OnceLock<Option<regex::Regex>> = std::sync::OnceLock::new();
- summarize_wake · function · L203-L254 — pub(super) fn summarize_wake(text: &str) -> String
- last_meaningful_line · function · L256-L262 — pub(super) fn last_meaningful_line(text: &str) -> Option<String>
- humanize_secs · function · L264-L277 — pub(super) fn humanize_secs(value: &str) -> String
- truncate_words · function · L280-L294 — pub(super) fn truncate_words(text: &str, max: usize) -> String
- truncate_chars · function · L298-L305 — pub(super) fn truncate_chars(text: &str, max: usize) -> String
