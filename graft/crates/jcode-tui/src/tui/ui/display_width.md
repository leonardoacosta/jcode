# crates/jcode-tui/src/tui/ui/display_width.rs

- line_display_width · function · L1-L6 — pub(super) fn line_display_width(text: &str) -> usize
- display_col_to_byte_offset · function · L8-L22 — pub(super) fn display_col_to_byte_offset(text: &str, display_col: usize) -> usize
- display_col_to_byte_offset_inclusive · function · L27-L41 — pub(super) fn display_col_to_byte_offset_inclusive(text: &str, display_col: usize) -> usize
- clamp_display_col · function · L43-L45 — pub(super) fn clamp_display_col(text: &str, display_col: usize) -> usize
- display_col_slice · function · L47-L51 — pub(super) fn display_col_slice(text: &str, start_col: usize, end_col: usize) -> &str
- display_col_slice_copy · function · L55-L62 — pub(super) fn display_col_slice_copy(text: &str, start_col: usize, end_col: usize) -> &str
- tests · module · L65-L124 — mod tests
- inclusive_offset_includes_partially_covered_wide_char · function · L72-L79 — fn inclusive_offset_includes_partially_covered_wide_char()
- copy_slice_keeps_last_cjk_char_when_end_is_inside_it · function · L82-L90 — fn copy_slice_keeps_last_cjk_char_when_end_is_inside_it()
- line_display_width_counts_wide_chars · function · L93-L96 — fn line_display_width_counts_wide_chars()
- display_col_to_byte_offset_stops_before_partial_wide_char · function · L99-L107 — fn display_col_to_byte_offset_stops_before_partial_wide_char()
- clamp_display_col_caps_at_line_display_width · function · L110-L113 — fn clamp_display_col_caps_at_line_display_width()
- display_col_slice_respects_wide_char_boundaries · function · L116-L123 — fn display_col_slice_respects_wide_char_boundaries()
