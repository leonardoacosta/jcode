# crates/jcode-tui/src/tui/app/oauth_telemetry.rs

- begin_frame · function · L9-L11 — pub(crate) fn begin_frame()
- note_status_render · function · L13-L15 — pub(crate) fn note_status_render(area: ratatui::layout::Rect)
- poll_oauth_account_telemetry · function · L20-L46 — pub(super) fn poll_oauth_account_telemetry(&mut self) -> bool
- acknowledge_rendered_oauth_notice · function · L50-L60 — pub(super) fn acknowledge_rendered_oauth_notice(&mut self, buffer: &ratatui::buffer::Buffer)
- readable_notice_active · function · L63-L65 — pub(super) fn readable_notice_active(at: std::time::Instant) -> bool
- notice_visible_in_area · function · L67-L75 — fn notice_visible_in_area(buffer: &ratatui::buffer::Buffer, area: ratatui::layout::Rect) -> bool
- tests · module · L78-L113 — mod tests
- oauth_notice_readable_interval_does_not_require_pending_upload · function · L83-L89 — fn oauth_notice_readable_interval_does_not_require_pending_upload()
- oauth_notice_frame_marker_is_reset_for_hidden_status · function · L92-L100 — fn oauth_notice_frame_marker_is_reset_for_hidden_status()
- oauth_notice_ack_requires_complete_visible_text · function · L103-L112 — fn oauth_notice_ack_requires_complete_visible_text()
