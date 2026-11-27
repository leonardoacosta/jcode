# crates/jcode-render-core/src/markdown.rs

- InlineStyle · struct · L20-L31 — struct InlineStyle
- attrs · function · L34-L41 — fn attrs(self) -> TextAttrs
- role · function · L43-L51 — fn role(self) -> StyleRole
- ListFrame · struct · L54-L62 — struct ListFrame
- align_ordered_markers · function · L71-L104 — fn align_ordered_markers(doc: &mut Document, start_block: usize, depth: usize)
- ordered_marker_digits · function · L108-L123 — fn ordered_marker_digits(block: &Block, indent: &str) -> Option<usize>
- current_kind · function · L127-L138 — fn current_kind(blockquote_depth: usize, list_stack: &[ListFrame]) -> BlockKind
- MarkdownParseOptions · struct · L142-L146 — pub struct MarkdownParseOptions
- default · function · L149-L153 — fn default() -> Self
- parse_markdown · function · L157-L159 — pub fn parse_markdown(text: &str) -> Document
- link_urls_option_preserves_core_default_and_link_style · function · L163-L195 — fn link_urls_option_preserves_core_default_and_link_style()
- parse_markdown_with_options · function · L198-L787 — pub fn parse_markdown_with_options(text: &str, config: MarkdownParseOptions) -> Document
