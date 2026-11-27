# crates/jcode-tui/src/tui/ui_pinned_layout.rs

- SIDE_PANEL_INLINE_IMAGE_TARGET_UTILIZATION_PERCENT · constant · L8-L8 — const SIDE_PANEL_INLINE_IMAGE_TARGET_UTILIZATION_PERCENT: u16 = 85;
- SIDE_PANEL_INLINE_IMAGE_MAX_AUTO_FILL_ZOOM_PERCENT · constant · L9-L9 — const SIDE_PANEL_INLINE_IMAGE_MAX_AUTO_FILL_ZOOM_PERCENT: u16 = 1000;
- estimate_side_panel_image_layout · function · L11-L38 — pub(super) fn estimate_side_panel_image_layout(
- estimate_side_panel_image_layout_with_font · function · L40-L117 — pub(super) fn estimate_side_panel_image_layout_with_font(
- axis_fill_zoom_percent · function · L119-L130 — fn axis_fill_zoom_percent(available_cells: u32, image_px: u32, cell_px: u32) -> u16
- rect_utilization_percent · function · L132-L137 — fn rect_utilization_percent(used: u16, total: u16) -> u16
- area_utilization_percent · function · L139-L146 — fn area_utilization_percent(used: Rect, total: Rect) -> u16
- scaled_image_rows · function · L148-L155 — pub(super) fn scaled_image_rows(image_h_cells: u32, zoom_percent: u16) -> u16
- estimate_side_panel_image_rows_with_font · function · L158-L187 — pub(super) fn estimate_side_panel_image_rows_with_font(
- side_panel_viewport_scroll_x · function · L189-L222 — pub(super) fn side_panel_viewport_scroll_x(
- fit_zoom_percent_for_area · function · L224-L240 — fn fit_zoom_percent_for_area(
- plan_fit_image_render · function · L242-L314 — pub(super) fn plan_fit_image_render(
- fit_side_panel_image_area · function · L316-L330 — pub(super) fn fit_side_panel_image_area(
- fit_image_area_with_font · function · L332-L382 — pub(super) fn fit_image_area_with_font(
- clamp_side_panel_image_rows · function · L384-L399 — pub(super) fn clamp_side_panel_image_rows(
