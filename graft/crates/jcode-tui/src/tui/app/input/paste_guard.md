# crates/jcode-tui/src/tui/app/input/paste_guard.rs

- PASTE_ENTER_SUPPRESS_WINDOW · constant · L11-L11 — const PASTE_ENTER_SUPPRESS_WINDOW: Duration = Duration::from_millis(150);
- note_paste · function · L18-L20 — pub(super) fn note_paste()
- consume_paste_trailing_enter · function · L25-L30 — pub(super) fn consume_paste_trailing_enter() -> bool
- expire_for_test · function · L34-L36 — pub(in crate::tui::app) fn expire_for_test()
- tiny_png_bytes_for_test · function · L41-L47 — pub(crate) fn tiny_png_bytes_for_test() -> Vec<u8>
- has_image_extension · function · L50-L59 — fn has_image_extension(path: &std::path::Path) -> bool
- load_dropped_image · function · L68-L92 — pub(super) fn load_dropped_image(path: &std::path::Path) -> Option<(String, Vec<u8>)>
