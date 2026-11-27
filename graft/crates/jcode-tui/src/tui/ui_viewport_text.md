# crates/jcode-tui/src/tui/ui_viewport_text.rs

- truncate_line_in_place_to_width · function · L4-L40 — pub(crate) fn truncate_line_in_place_to_width(line: &mut Line<'static>, max_width: usize)
- trim_line_trailing_spaces · function · L45-L58 — pub(crate) fn trim_line_trailing_spaces(line: &mut Line<'static>)
- truncate_line_for_copy_badge · function · L65-L78 — pub(crate) fn truncate_line_for_copy_badge(line: &mut Line<'static>, max_content_width: usize)
- copy_badge_alt_label_from_config · function · L80-L87 — pub(super) fn copy_badge_alt_label_from_config(configured: &str) -> String
- allocation_tests · module · L90-L179 — mod allocation_tests
- legacy_truncate · function · L94-L123 — fn legacy_truncate(line: &mut Line<'static>, max_width: usize)
- viewport_truncation_matches_legacy_for_unicode_and_styles · function · L126-L152 — fn viewport_truncation_matches_legacy_for_unicode_and_styles()
- viewport_truncation_reuses_span_storage · function · L155-L178 — fn viewport_truncation_reuses_span_storage()
