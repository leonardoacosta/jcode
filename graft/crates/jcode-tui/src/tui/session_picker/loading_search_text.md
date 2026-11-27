# crates/jcode-tui/src/tui/session_picker/loading_search_text.rs

- push_with_byte_budget · function · L4-L19 — pub(crate) fn push_with_byte_budget(dst: &mut String, src: &str, budget: &mut usize)
- suffix_at_most · function · L21-L27 — pub(crate) fn suffix_at_most(value: &str, max_bytes: usize) -> &str
- push_sampled_search_text · function · L33-L79 — pub(crate) fn push_sampled_search_text(dst: &mut String, src: &str, limit: usize)
- build_search_index · function · L81-L112 — pub(crate) fn build_search_index(
- search_index_header · function · L117-L139 — pub(crate) fn search_index_header(
- classify_json_into · function · L144-L167 — pub(crate) fn classify_json_into(
- write_tags_first · function · L169-L201 — fn write_tags_first(value: &serde_json::Value, out: &mut String)
- raw_value_search_excerpt · function · L204-L213 — pub(crate) fn raw_value_search_excerpt(raw: &RawValue, budget: usize) -> Option<String>
- raw_value_display_text · function · L215-L252 — pub(crate) fn raw_value_display_text(raw: &RawValue) -> Option<String>
- collect_text · function · L216-L239 — fn collect_text(value: &serde_json::Value, out: &mut String)
- classified_search_excerpt · function · L257-L284 — pub(crate) fn classified_search_excerpt(
