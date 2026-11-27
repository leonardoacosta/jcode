# crates/jcode-base/src/session/message_helpers.rs

- stored_messages_to_messages · function · L3-L5 — pub(super) fn stored_messages_to_messages(messages: &[StoredMessage]) -> Vec<Message>
- message_has_tool_image · function · L7-L14 — pub(super) fn message_has_tool_image(content: &[ContentBlock]) -> bool
- is_internal_system_reminder_message · function · L16-L25 — pub(super) fn is_internal_system_reminder_message(message: &StoredMessage) -> bool
- is_visible_conversation_message · function · L27-L31 — pub(super) fn is_visible_conversation_message(message: &StoredMessage) -> bool
- is_scheduled_task_message · function · L36-L41 — pub fn is_scheduled_task_message(message: &StoredMessage) -> bool
- derive_session_provider_key · function · L43-L83 — pub fn derive_session_provider_key(provider_name: &str) -> Option<String>
