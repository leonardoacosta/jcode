# crates/jcode-tui/src/tui/app/jarvis_status.rs

- CaptureStatus · struct · L11-L20 — pub(super) struct CaptureStatus
- new · function · L23-L40 — pub(super) fn new() -> Result<Self>
- in_directory · function · L42-L86 — fn in_directory(directory: &Path) -> Result<Self>
- paused · function · L88-L90 — pub(super) fn paused(&self) -> bool
- set_paused · function · L92-L113 — pub(super) fn set_paused(&self, paused: bool) -> Result<()>
- publish · function · L115-L123 — pub(super) fn publish(
- publish_at · function · L125-L180 — fn publish_at(
- drop · function · L184-L188 — fn drop(&mut self)
- tests · module · L192-L280 — mod tests
- native_history_is_bounded_throttled_and_cleared_on_pause · function · L195-L259 — fn native_history_is_bounded_throttled_and_cleared_on_pause()
- status_is_private_exclusive_and_cleared_on_drop · function · L262-L279 — fn status_is_private_exclusive_and_cleared_on_drop()
