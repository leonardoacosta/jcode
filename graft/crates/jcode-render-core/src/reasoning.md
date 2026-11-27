# crates/jcode-render-core/src/reasoning.rs

- REASONING_SENTINEL · constant · L12-L12 — pub const REASONING_SENTINEL: &str = "\u{2063}";
- REASONING_ESCAPES · constant · L14-L14 — const REASONING_ESCAPES: &str = "\\*_`[]<>&~|$";
- reasoning_line_content · function · L20-L39 — pub fn reasoning_line_content(line: &str) -> Option<String>
- escape_reasoning_inline_markdown · function · L44-L53 — fn escape_reasoning_inline_markdown(line: &str) -> String
- reasoning_line_markup · function · L72-L82 — pub fn reasoning_line_markup(line: &str) -> String
- reasoning_partial_markup · function · L90-L100 — pub fn reasoning_partial_markup(line: &str) -> String
- reasoning_summary_line_markup · function · L112-L118 — pub fn reasoning_summary_line_markup(line_count: usize) -> String
- tests · module · L121-L151 — mod tests
- reasoning_markup_round_trips_original_markdown · function · L125-L137 — fn reasoning_markup_round_trips_original_markdown()
- reasoning_decoder_requires_both_sentinels_and_wrapper · function · L140-L150 — fn reasoning_decoder_requires_both_sentinels_and_wrapper()
