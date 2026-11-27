# crates/jcode-tui/src/tui/ui_panel_image_preview.rs

- clear_regions · function · L11-L13 — pub(crate) fn clear_regions()
- record_image · function · L15-L22 — pub(super) fn record_image(area: Rect, hash: u64)
- image_at · function · L24-L32 — pub(crate) fn image_at(column: u16, row: u16) -> Option<u64>
- image_position_on_row · function · L34-L42 — pub(super) fn image_position_on_row(pane: Rect, row: u16) -> Option<(u16, u16)>
- draw_preview · function · L44-L64 — pub(super) fn draw_preview(frame: &mut Frame, area: Rect, hash: u64)
- preview_image_area · function · L66-L82 — fn preview_image_area(area: Rect, width: u32, height: u32, font_size: Option<(u16, u16)>) -> Rect
- tests · module · L85-L118 — mod tests
- panel_image_preview_targets_only_visible_image_cells · function · L89-L99 — fn panel_image_preview_targets_only_visible_image_cells()
- panel_image_preview_fits_and_centers_wide_and_tall_images · function · L102-L117 — fn panel_image_preview_fits_and_centers_wide_and_tall_images()
