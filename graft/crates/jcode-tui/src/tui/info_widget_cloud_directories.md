# crates/jcode-tui/src/tui/info_widget_cloud_directories.rs

- MAX_ENTRIES · constant · L10-L10 — const MAX_ENTRIES: usize = 6;
- fit · function · L13-L27 — fn fit(text: &str, width: usize) -> String
- render_cloud_directories · function · L29-L124 — pub(super) fn render_cloud_directories(data: &InfoWidgetData, inner: Rect) -> Framed
- tests · module · L127-L326 — mod tests
- fixture · function · L134-L177 — fn fixture() -> InfoWidgetData
- render · function · L178-L180 — fn render(data: &InfoWidgetData, width: u16) -> Framed
- cloud_directories_local_absent_and_loading · function · L182-L194 — fn cloud_directories_local_absent_and_loading()
- cloud_directories_order_counts_and_staleness · function · L196-L243 — fn cloud_directories_order_counts_and_staleness()
- cloud_directories_unicode_widths_and_bounded_overflow · function · L245-L299 — fn cloud_directories_unicode_widths_and_bounded_overflow()
- cloud_directories_empty_and_layout · function · L301-L325 — fn cloud_directories_empty_and_layout()
