# crates/jcode-app-core/src/tool/panel.rs

- PanelTool · struct · L8-L8 — pub struct PanelTool;
- new · function · L10-L12 — pub fn new() -> Self
- PanelInput · struct · L17-L30 — struct PanelInput
- name · function · L34-L36 — fn name(&self) -> &str
- description · function · L37-L39 — fn description(&self) -> &str
- parameters_schema · function · L40-L54 — fn parameters_schema(&self) -> Value
- execute · function · L55-L134 — async fn execute(&self, input: Value, ctx: ToolContext) -> Result<ToolOutput>
- tests · module · L139-L139 — mod tests;
