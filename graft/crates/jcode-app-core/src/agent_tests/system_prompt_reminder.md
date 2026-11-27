# crates/jcode-app-core/src/agent_tests/system_prompt_reminder.rs

- CapturedSystemPromptRequests · type · L3-L3 — type CapturedSystemPromptRequests = Arc<std::sync::Mutex<Vec<(String, Vec<Message>)>>>;
- SystemPromptCaptureProvider · struct · L6-L8 — struct SystemPromptCaptureProvider
- complete · function · L12-L33 — async fn complete(
- name · function · L35-L37 — fn name(&self) -> &str
- fork · function · L39-L41 — fn fork(&self) -> Arc<dyn Provider>
- turn_system_reminder_does_not_change_the_system_prompt · function · L49-L111 — async fn turn_system_reminder_does_not_change_the_system_prompt()
- pending_alerts_and_reminder_only_turn_publish_context_entries · function · L114-L157 — async fn pending_alerts_and_reminder_only_turn_publish_context_entries()
- late_skill_announcement_publishes_one_new_context_entry · function · L160-L194 — async fn late_skill_announcement_publishes_one_new_context_entry()
