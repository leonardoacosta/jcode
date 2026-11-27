# crates/jcode-tui/src/tui/ui_overlays.rs

- draw_changelog_overlay · function · L15-L136 — pub(super) fn draw_changelog_overlay(
- draw_commit_log_overlay · function · L138-L221 — pub(super) fn draw_commit_log_overlay(
- draw_help_overlay · function · L223-L755 — pub(super) fn draw_help_overlay(frame: &mut Frame, area: Rect, scroll: usize, app: &dyn TuiState)
- draw_model_status_overlay · function · L757-L808 — pub(super) fn draw_model_status_overlay(
- model_status_line_style · function · L810-L824 — fn model_status_line_style(raw: &str, default: Style) -> Style
- draw_debug_overlay · function · L826-L847 — pub(super) fn draw_debug_overlay(
- render_overlay_box · function · L849-L858 — fn render_overlay_box(frame: &mut Frame, area: Rect, title: &str, color: Color)
- debug_palette_json · function · L860-L877 — pub(super) fn debug_palette_json() -> Option<serde_json::Value>
- color_to_rgb · function · L879-L888 — fn color_to_rgb(color: Color) -> Option<[u8; 3]>
