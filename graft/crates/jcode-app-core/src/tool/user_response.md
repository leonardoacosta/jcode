# crates/jcode-app-core/src/tool/user_response.rs

- UserResponseTool · struct · L8-L8 — pub(super) struct UserResponseTool;
- Input · struct · L12-L19 — struct Input
- name · function · L23-L25 — fn name(&self) -> &str
- description · function · L27-L29 — fn description(&self) -> &str
- parameters_schema · function · L31-L42 — fn parameters_schema(&self) -> Value
- execute · function · L44-L49 — async fn execute(&self, input: Value, _ctx: ToolContext) -> Result<ToolOutput>
- tests · module · L53-L144 — mod tests
- context · function · L56-L66 — fn context() -> ToolContext
- user_response_preserves_markdown_and_optional_title · function · L69-L93 — async fn user_response_preserves_markdown_and_optional_title()
- user_response_rejects_invalid_input · function · L96-L106 — async fn user_response_rejects_invalid_input()
- user_response_desktop_registration_is_reversible_and_isolated · function · L109-L143 — async fn user_response_desktop_registration_is_reversible_and_isolated()
