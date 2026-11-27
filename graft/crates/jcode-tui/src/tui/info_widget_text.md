# crates/jcode-tui/src/tui/info_widget_text.rs

- truncate_smart · function · L3-L31 — pub(super) fn truncate_smart(s: &str, max_len: usize) -> String
- clip · function · L37-L46 — pub(super) fn clip(s: &str, max_cells: usize) -> String
- tests · module · L49-L66 — mod tests
- clip_fills_the_width_without_an_ellipsis · function · L53-L59 — fn clip_fills_the_width_without_an_ellipsis()
- clip_counts_display_cells_not_chars · function · L62-L65 — fn clip_counts_display_cells_not_chars()
- cell_width_tests · module · L69-L92 — mod cell_width_tests
- truncate_smart_keeps_fitting_text · function · L74-L77 — fn truncate_smart_keeps_fitting_text()
- truncate_smart_counts_cjk_cells · function · L80-L84 — fn truncate_smart_counts_cjk_cells()
- truncate_smart_counts_emoji_cells · function · L87-L91 — fn truncate_smart_counts_emoji_cells()
