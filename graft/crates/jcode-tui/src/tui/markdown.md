# crates/jcode-tui/src/tui/markdown.rs

- to_markdown_diagram_mode · function · L13-L21 — fn to_markdown_diagram_mode(
- from_markdown_diagram_mode · function · L23-L31 — fn from_markdown_diagram_mode(
- to_markdown_spacing_mode · function · L33-L44 — fn to_markdown_spacing_mode(
- to_markdown_latex_mode · function · L46-L56 — fn to_markdown_latex_mode(
- install_jcode_markdown_hooks · function · L58-L85 — pub fn install_jcode_markdown_hooks()
- set_diagram_mode_override · function · L87-L89 — pub fn set_diagram_mode_override(mode: Option<crate::config::DiagramDisplayMode>)
- get_diagram_mode_override · function · L91-L93 — pub fn get_diagram_mode_override() -> Option<crate::config::DiagramDisplayMode>
- with_diagram_mode_scope · function · L98-L103 — pub fn with_diagram_mode_scope<T>(
- with_deferred_mermaid_render_context · function · L105-L107 — pub fn with_deferred_mermaid_render_context<T>(f: impl FnOnce() -> T) -> T
