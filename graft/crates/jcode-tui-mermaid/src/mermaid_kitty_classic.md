# crates/jcode-tui-mermaid/src/mermaid_kitty_classic.rs

- PixelCrop · type · L8-L8 — type PixelCrop = (u32, u32, u32, u32);
- Placement · struct · L10-L17 — struct Placement
- ImageFrame · struct · L20-L25 — struct ImageFrame
- FRAME · constant · L27-L27 — static FRAME: LazyLock<Mutex<ImageFrame>> = LazyLock::new(|| Mutex::new(ImageFrame::default()));
- FrameComposition · struct · L30-L33 — struct FrameComposition
- with_image_frame_offset · function · L47-L67 — pub fn with_image_frame_offset<T>(x: u16, y: u16, render: impl FnOnce() -> T) -> T
- Restore · struct · L48-L48 — struct Restore(FrameComposition);
- drop · function · L50-L52 — fn drop(&mut self)
- state_hash · function · L69-L78 — pub(super) fn state_hash(hash: u64, fit: Option<(u16, u16)>, font: (u16, u16)) -> u64
- delete_placement · function · L80-L83 — fn delete_placement(payload: &mut String, image: u32, placement: u32)
- begin_terminal_image_frame · function · L88-L104 — pub fn begin_terminal_image_frame()
- clear_placements · function · L106-L113 — pub(super) fn clear_placements()
- take_placement_cleanup · function · L115-L120 — pub(super) fn take_placement_cleanup() -> String
- render_crop · function · L122-L186 — pub(super) fn render_crop(hash: u64, area: Rect, buf: &mut Buffer, crop: PixelCrop) -> bool
- render_cell_viewport · function · L189-L220 — pub(super) fn render_cell_viewport(
- finish_terminal_image_frame · function · L226-L342 — pub fn finish_terminal_image_frame(buf: &mut Buffer) -> bool
- render_widget · function · L346-L423 — pub(super) fn render_widget(
- tests · module · L427-L427 — mod tests;
