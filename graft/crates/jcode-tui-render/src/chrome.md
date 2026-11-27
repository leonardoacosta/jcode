# crates/jcode-tui-render/src/chrome.rs

- clear_area · function · L4-L21 — pub fn clear_area(frame: &mut Frame, area: Rect)
- left_aligned_content_inset · function · L23-L25 — pub fn left_aligned_content_inset(width: u16, centered: bool) -> u16
- centered_content_block_width · function · L27-L29 — pub fn centered_content_block_width(width: u16, max_width: usize) -> usize
- left_pad_lines_to_block_width · function · L31-L40 — pub fn left_pad_lines_to_block_width(lines: &mut [Line<'static>], width: u16, block_width: usize)
- RIGHT_RAIL_HEADER_HEIGHT · constant · L42-L42 — const RIGHT_RAIL_HEADER_HEIGHT: u16 = 1;
- right_rail_border_style · function · L44-L47 — pub fn right_rail_border_style(focused: bool, focus_color: Color, dim_color: Color) -> Style
- right_rail_inner · function · L49-L51 — fn right_rail_inner(area: Rect) -> Rect
- right_rail_content_area · function · L53-L65 — fn right_rail_content_area(area: Rect) -> Option<Rect>
- draw_right_rail_chrome · function · L67-L95 — pub fn draw_right_rail_chrome(
- align_if_unset · function · L100-L106 — pub fn align_if_unset(line: Line<'static>, align: Alignment) -> Line<'static>
- clear_area_tests · module · L109-L138 — mod clear_area_tests
- clear_area_clips_to_a_one_row_buffer · function · L116-L137 — fn clear_area_clips_to_a_one_row_buffer()
