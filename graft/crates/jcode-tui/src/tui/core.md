# crates/jcode-tui/src/tui/core.rs

- prev_char_boundary · function · L9-L19 — pub fn prev_char_boundary(s: &str, pos: usize) -> usize
- next_char_boundary · function · L23-L29 — pub fn next_char_boundary(s: &str, pos: usize) -> usize
- byte_offset_to_char_index · function · L33-L35 — pub fn byte_offset_to_char_index(s: &str, byte_offset: usize) -> usize
- char_index_to_byte_offset · function · L39-L48 — pub fn char_index_to_byte_offset(s: &str, char_index: usize) -> usize
- DisplayMessageRoleExt · interface · L52-L60 — pub(crate) trait DisplayMessageRoleExt
- effective_role · function · L59-L59 — fn effective_role(&self) -> &str;
- effective_role · function · L63-L72 — fn effective_role(&self) -> &str
- tests · module · L76-L161 — mod tests
- swarm_await_wake_reports_use_notification_role_without_changing_content · function · L81-L96 — fn swarm_await_wake_reports_use_notification_role_without_changing_content()
- background_task_wakes_share_semantic_prompt_classification · function · L99-L116 — fn background_task_wakes_share_semantic_prompt_classification()
- test_display_message_helpers · function · L119-L127 — fn test_display_message_helpers()
- test_byte_offset_to_char_index · function · L130-L145 — fn test_byte_offset_to_char_index()
- test_char_boundary_helpers · function · L148-L160 — fn test_char_boundary_helpers()
