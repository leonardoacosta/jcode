# crates/jcode-tui-render/src/layout.rs

- rect_contains · function · L3-L8 — pub fn rect_contains(outer: Rect, inner: Rect) -> bool
- point_in_rect · function · L10-L15 — pub fn point_in_rect(col: u16, row: u16, rect: Rect) -> bool
- parse_area_spec · function · L17-L32 — pub fn parse_area_spec(spec: &str) -> Option<Rect>
- tests · module · L35-L64 — mod tests
- rect_contains_requires_full_containment · function · L39-L46 — fn rect_contains_requires_full_containment()
- point_in_rect_uses_half_open_bounds · function · L49-L56 — fn point_in_rect_uses_half_open_bounds()
- parse_area_spec_parses_geometry · function · L59-L63 — fn parse_area_spec_parses_geometry()
