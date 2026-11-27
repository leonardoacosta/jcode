# crates/jcode-app-core/src/server/runtime_commands.rs

- boolean_action · function · L7-L24 — pub(super) fn boolean_action(
- status_only · function · L26-L32 — fn status_only(args: &[String]) -> Result<()>
- execute · function · L34-L358 — pub(super) async fn execute(
- role_args · function · L360-L405 — fn role_args(args: &[String], default_label: &str) -> Result<(Option<String>, String, String)>
- skills_report · function · L407-L452 — fn skills_report(data: &Value) -> String
- tests · module · L455-L661 — mod tests
- NoInferenceProvider · struct · L460-L462 — struct NoInferenceProvider
- complete · function · L465-L473 — async fn complete(
- name · function · L474-L476 — fn name(&self) -> &str
- fork · function · L477-L479 — fn fork(&self) -> Arc<dyn crate::provider::Provider>
- service_tier · function · L480-L482 — fn service_tier(&self) -> Option<String>
- set_service_tier · function · L483-L490 — fn set_service_tier(&self, tier: &str) -> Result<()>
- runtime_commands_read_actual_state_without_inference · function · L495-L622 — async fn runtime_commands_read_actual_state_without_inference()
- Restore · struct · L500-L500 — struct Restore(Option<std::ffi::OsString>);
- drop · function · L502-L508 — fn drop(&mut self)
- toggle_and_status_are_distinct · function · L624-L633 — fn toggle_and_status_are_distinct()
- role_arguments_preserve_model_and_task_and_reject_unsupported_resume · function · L635-L660 — fn role_arguments_preserve_model_and_task_and_reject_unsupported_resume()
