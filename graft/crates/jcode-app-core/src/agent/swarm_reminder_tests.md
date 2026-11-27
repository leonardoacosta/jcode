# crates/jcode-app-core/src/agent/swarm_reminder_tests.rs

- gateway_defaults_are_not_overridden_by_agent_roles · function · L5-L102 — async fn gateway_defaults_are_not_overridden_by_agent_roles()
- PolicyProvider · struct · L8-L8 — struct PolicyProvider(Arc<std::sync::Mutex<Vec<InputCompressionProfile>>>);
- set_input_compression_profile · function · L11-L14 — fn set_input_compression_profile(&self, profile: InputCompressionProfile) -> bool
- complete · function · L15-L23 — async fn complete(
- name · function · L24-L26 — fn name(&self) -> &str
- model · function · L27-L29 — fn model(&self) -> String
- fork · function · L30-L32 — fn fork(&self) -> Arc<dyn Provider>
- EffortProvider · struct · L105-L107 — struct EffortProvider
- complete · function · L111-L119 — async fn complete(
- name · function · L120-L122 — fn name(&self) -> &str
- model · function · L123-L125 — fn model(&self) -> String
- reasoning_effort · function · L126-L128 — fn reasoning_effort(&self) -> Option<String>
- fork · function · L129-L131 — fn fork(&self) -> Arc<dyn Provider>
- last_user_text · function · L134-L155 — async fn last_user_text(effort: Option<&str>, message: &str) -> String
- swarm_effort_appends_reminder_to_user_message · function · L158-L165 — async fn swarm_effort_appends_reminder_to_user_message()
- non_swarm_effort_leaves_user_message_unchanged · function · L168-L176 — async fn non_swarm_effort_leaves_user_message_unchanged()
- swarm_effort_does_not_touch_system_prompt · function · L179-L191 — async fn swarm_effort_does_not_touch_system_prompt()
- run_once_path_also_appends_reminder · function · L194-L217 — async fn run_once_path_also_appends_reminder()
