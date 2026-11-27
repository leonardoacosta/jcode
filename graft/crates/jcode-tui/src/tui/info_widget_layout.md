# crates/jcode-tui/src/tui/info_widget_layout.rs

- MIN_WIDGET_WIDTH · constant · L9-L9 — const MIN_WIDGET_WIDTH: u16 = 24;
- MAX_WIDGET_WIDTH · constant · L11-L11 — const MAX_WIDGET_WIDTH: u16 = 40;
- MIN_WIDGET_HEIGHT · constant · L13-L13 — const MIN_WIDGET_HEIGHT: u16 = 5;
- MAX_HIDDEN_FRAMES · constant · L23-L23 — const MAX_HIDDEN_FRAMES: u16 = 120;
- WidgetAnchor · struct · L28-L37 — pub(crate) struct WidgetAnchor
- PlacementOutcome · struct · L41-L44 — pub(crate) struct PlacementOutcome
- Margins · struct · L48-L72 — pub struct Margins
- right_dock_widths · function · L76-L82 — fn right_dock_widths(&self) -> &[u16]
- left_dock_widths · function · L84-L90 — fn left_dock_widths(&self) -> &[u16]
- MarginSpace · struct · L95-L101 — struct MarginSpace
- dock_widths_with_fallback · function · L107-L118 — fn dock_widths_with_fallback(reliable: &[u16], instant: &[u16]) -> Vec<u16>
- calculate_placements_anchored · function · L125-L536 — pub(crate) fn calculate_placements_anchored(
- reserve_rows · function · L542-L571 — fn reserve_rows(
- find_all_empty_rects · function · L575-L609 — fn find_all_empty_rects(
- add_region_rects · function · L611-L633 — fn add_region_rects(
- tests · module · L637-L637 — mod tests;
