# crates/jcode-tui/src/tui/ui_debug_capture.rs

- capture_widget_placements · function · L5-L16 — pub(super) fn capture_widget_placements(
- build_info_widget_summary · function · L18-L78 — pub(super) fn build_info_widget_summary(data: &info_widget::InfoWidgetData) -> InfoWidgetSummary
- rects_overlap · function · L80-L89 — pub(super) fn rects_overlap(a: Rect, b: Rect) -> bool
- rect_within_bounds · function · L91-L97 — pub(super) fn rect_within_bounds(rect: Rect, bounds: Rect) -> bool
- widget_overlaps_content · function · L108-L136 — pub(super) fn widget_overlaps_content(
- tests · module · L139-L193 — mod tests
- placement · function · L143-L149 — fn placement(side: Side, x: u16, y: u16, w: u16, h: u16) -> WidgetPlacement
- widget_fitting_in_reported_margin_is_not_flagged · function · L152-L163 — fn widget_fitting_in_reported_margin_is_not_flagged()
- widget_wider_than_free_margin_intrudes · function · L166-L178 — fn widget_wider_than_free_margin_intrudes()
- widget_on_row_without_recorded_margin_intrudes · function · L181-L192 — fn widget_on_row_without_recorded_margin_intrudes()
