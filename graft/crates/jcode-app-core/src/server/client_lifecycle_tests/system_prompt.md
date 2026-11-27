# crates/jcode-app-core/src/server/client_lifecycle_tests/system_prompt.rs

- subscribe_system_prompt_is_creation_only_and_preserves_empty · function · L4-L17 — fn subscribe_system_prompt_is_creation_only_and_preserves_empty()
- SystemPromptCaptureProvider · struct · L20-L20 — struct SystemPromptCaptureProvider(Arc<std::sync::Mutex<Vec<String>>>);
- complete · function · L24-L36 — async fn complete(
- name · function · L37-L39 — fn name(&self) -> &str
- fork · function · L40-L42 — fn fork(&self) -> Arc<dyn Provider>
- system_prompt_socket_creation_attach_resume_fork_and_no_leaking · function · L46-L339 — async fn system_prompt_socket_creation_attach_resume_fork_and_no_leaking()
- send · function · L116-L121 — async fn send(writer: &mut crate::transport::WriteHalf, value: serde_json::Value)
- until · function · L122-L142 — async fn until(
