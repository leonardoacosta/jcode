# crates/jcode-tui/src/tui/ui_layout.rs

- SIDE_PANEL_DISMISSED_HINT · constant · L3-L3 — pub(crate) const SIDE_PANEL_DISMISSED_HINT: &str = "Side panel hidden. Alt+M brings it back";
- side_panel_restore_area · function · L9-L11 — pub(crate) fn side_panel_restore_area() -> Option<Rect>
- draw_side_panel_restore · function · L16-L38 — pub(crate) fn draw_side_panel_restore(frame: &mut ratatui::Frame, app: &dyn crate::tui::TuiState)
- right_rail_border_style · function · L46-L52 — pub(super) fn right_rail_border_style(focused: bool, focus_color: Color) -> Style
- side_panel_dismiss_area · function · L56-L59 — pub(crate) fn side_panel_dismiss_area(area: Rect) -> Option<Rect>
- draw_side_panel_markdown · function · L62-L81 — pub(crate) fn draw_side_panel_markdown(
