# crates/jcode-tui-markdown/src/markdown_render_support.rs

- line_plain_text · function · L3-L8 — pub(super) fn line_plain_text(line: &Line<'_>) -> String
- extract_copy_targets_from_rendered_lines · function · L10-L136 — pub fn extract_copy_targets_from_rendered_lines(lines: &[Line<'static>]) -> Vec<RawCopyTarget>
- is_blockquote_gutter_line · function · L142-L144 — fn is_blockquote_gutter_line(text: &str) -> bool
- strip_blockquote_gutter · function · L148-L159 — fn strip_blockquote_gutter(text: &str) -> &str
- render_table_aligned · function · L168-L305 — pub(super) fn render_table_aligned(
- wrap_table_cell · function · L307-L352 — fn wrap_table_cell(cell: &str, width: usize) -> Vec<String>
- wrap_long_table_word · function · L354-L374 — fn wrap_long_table_word(word: &str, width: usize) -> Vec<String>
- render_table_with_width · function · L377-L379 — pub fn render_table_with_width(rows: &[Vec<String>], max_width: usize) -> Vec<Line<'static>>
- highlight_code_cached · function · L384-L409 — pub(super) fn highlight_code_cached(code: &str, lang: Option<&str>) -> Vec<Line<'static>>
- highlight_code · function · L412-L441 — pub(super) fn highlight_code(code: &str, lang: Option<&str>) -> Vec<Line<'static>>
- syntect_ranges_to_spans · function · L447-L472 — fn syntect_ranges_to_spans(ranges: Vec<(SynStyle, &str)>) -> Vec<Span<'static>>
- syntect_to_ratatui_style · function · L475-L478 — fn syntect_to_ratatui_style(style: SynStyle) -> Style
- highlight_line · function · L483-L498 — pub fn highlight_line(code: &str, ext: Option<&str>) -> Vec<Span<'static>>
- highlight_file_lines · function · L502-L528 — pub fn highlight_file_lines(
- placeholder_code_block · function · L532-L541 — pub(super) fn placeholder_code_block(code: &str, lang: Option<&str>) -> Vec<Line<'static>>
- ranges_overlap · function · L544-L546 — pub(super) fn ranges_overlap(a: std::ops::Range<usize>, b: std::ops::Range<usize>) -> bool
