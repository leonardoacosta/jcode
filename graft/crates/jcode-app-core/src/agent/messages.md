# crates/jcode-app-core/src/agent/messages.rs

- persist_turn_error · function · L6-L10 — pub(crate) fn persist_turn_error(&mut self, message: &str) -> anyhow::Result<()>
- add_message · function · L12-L27 — pub(crate) fn add_message(
- add_message_with_display_role · function · L29-L47 — pub(crate) fn add_message_with_display_role(
- add_message_with_duration · function · L49-L67 — pub(crate) fn add_message_with_duration(
- add_message_ext · function · L69-L88 — pub(crate) fn add_message_ext(
- export_conversation_markdown · function · L93-L160 — pub fn export_conversation_markdown(&self) -> String
- messages_end_with_tool_result · function · L164-L193 — pub(super) fn messages_end_with_tool_result(messages: &[Message]) -> bool
- durable_error_tests · module · L197-L292 — mod durable_error_tests
- NoNetworkProvider · struct · L200-L200 — struct NoNetworkProvider;
- complete · function · L204-L212 — async fn complete(
- name · function · L214-L216 — fn name(&self) -> &str
- fork · function · L218-L220 — fn fork(&self) -> Arc<dyn crate::provider::Provider>
- message_insertion_retains_shared_payload_storage · function · L224-L242 — fn message_insertion_retains_shared_payload_storage()
- failed_wake_error_is_saved_without_an_assistant_message · function · L245-L291 — fn failed_wake_error_is_saved_without_an_assistant_message()
- RestoreHome · struct · L248-L248 — struct RestoreHome(Option<std::ffi::OsString>);
- drop · function · L250-L255 — fn drop(&mut self)
