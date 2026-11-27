# crates/jcode-tui/src/tui/session_picker/render.rs

- todo_dot_spans · function · L5-L22 — pub(super) fn todo_dot_spans(session: &SessionInfo, width: usize) -> Vec<Span<'static>>
- running_spinner_frame · function · L24-L32 — fn running_spinner_frame() -> usize
- crash_reason_line · function · L34-L55 — pub(super) fn crash_reason_line(session: &SessionInfo) -> Option<Line<'static>>
- format_estimated_tokens · function · L58-L100 — pub(super) fn format_estimated_tokens(tokens: usize) -> String
- UNITS · constant · L63-L71 — const UNITS: &[(f64, &str)] = &[
- active_highlight_tokens · function · L105-L107 — pub(super) fn active_highlight_tokens(&self) -> Vec<String>
- highlight_spans · function · L114-L174 — pub(super) fn highlight_spans(
- primary_title_display · function · L176-L185 — fn primary_title_display(session: &SessionInfo) -> String
- render_session_item_lines · function · L188-L194 — pub(super) fn render_session_item_lines(
- render_session_item_lines_at_frame · function · L196-L341 — pub(super) fn render_session_item_lines_at_frame(
- render_session_item · function · L343-L403 — fn render_session_item(
- render_session_list · function · L405-L407 — pub(super) fn render_session_list(&mut self, frame: &mut Frame, area: Rect)
- list_session_at · function · L409-L416 — fn list_session_at(&self, index: usize) -> Option<&SessionInfo>
- render_session_list_impl · function · L418-L804 — pub(super) fn render_session_list_impl(
- render_crash_banner · function · L806-L843 — pub(super) fn render_crash_banner(&self, frame: &mut Frame, area: Rect)
- compact_dir · function · L848-L881 — pub(super) fn compact_dir(dir: &str) -> String
- MAX · constant · L849-L849 — const MAX: usize = 28;
