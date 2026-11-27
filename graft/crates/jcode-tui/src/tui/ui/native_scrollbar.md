# crates/jcode-tui/src/tui/ui/native_scrollbar.rs

- split_native_scrollbar_area · function · L4-L20 — pub(crate) fn split_native_scrollbar_area(area: Rect, enabled: bool) -> (Rect, Option<Rect>)
- native_scrollbar_visible · function · L22-L28 — pub(crate) fn native_scrollbar_visible(
- NativeScrollbar · struct · L33-L39 — pub(crate) struct NativeScrollbar
- render · function · L42-L51 — pub(crate) fn render(&self, frame: &mut Frame)
- render_thumb_overlay · function · L55-L75 — pub(crate) fn render_thumb_overlay(&self, frame: &mut Frame)
- NativeScrollbarThumb · struct · L79-L82 — pub(crate) struct NativeScrollbarThumb
- contains · function · L85-L87 — pub(crate) fn contains(&self, row: usize) -> bool
- glyph · function · L89-L99 — pub(crate) fn glyph(&self, row: usize) -> &'static str
- native_scrollbar_thumb · function · L102-L125 — pub(crate) fn native_scrollbar_thumb(
- native_scrollbar_thumb_color · function · L127-L133 — pub(crate) fn native_scrollbar_thumb_color(focused: bool) -> Color
- render_native_scrollbar · function · L135-L159 — pub(crate) fn render_native_scrollbar(
