# crates/jcode-tui-markdown/src/render_core_adapter.rs

- document_to_lines · function · L27-L29 — pub fn document_to_lines(doc: &Document) -> Vec<Line<'static>>
- push_code_block · function · L33-L50 — fn push_code_block(
- push_math_display · function · L54-L68 — fn push_math_display(lines: &mut Vec<Line<'static>>, block: &jcode_render_core::Block)
- styled_line_to_line · function · L72-L85 — pub fn styled_line_to_line(sl: &StyledLine, kind: &BlockKind) -> Line<'static>
- styled_span_to_span · function · L87-L112 — fn styled_span_to_span(span: &StyledSpan, kind: &BlockKind) -> Span<'static>
- role_color · function · L114-L135 — fn role_color(role: StyleRole, kind: &BlockKind) -> ratatui::style::Color
- render_markdown_via_core · function · L138-L140 — pub fn render_markdown_via_core(text: &str) -> Vec<Line<'static>>
- parse_markdown_for_tui · function · L142-L149 — fn parse_markdown_for_tui(text: &str) -> Document
- document_to_lines_with_width · function · L155-L196 — pub fn document_to_lines_with_width(doc: &Document, width: Option<usize>) -> Vec<Line<'static>>
- rule_width · function · L200-L206 — fn rule_width(width: Option<usize>) -> usize
- render_markdown_via_core_wrapped · function · L212-L216 — pub fn render_markdown_via_core_wrapped(text: &str, width: usize) -> Vec<Line<'static>>
- tests · module · L220-L220 — mod tests;
