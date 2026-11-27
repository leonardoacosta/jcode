# crates/jcode-app-core/src/tool/side_panel.rs

- SidePanelTool · struct · L11-L11 — pub struct SidePanelTool;
- new · function · L14-L16 — pub fn new() -> Self
- SidePanelInput · struct · L20-L32 — struct SidePanelInput
- name · function · L36-L38 — fn name(&self) -> &str
- model_visible · function · L40-L42 — fn model_visible(&self) -> bool
- description · function · L44-L46 — fn description(&self) -> &str
- parameters_schema · function · L48-L81 — fn parameters_schema(&self) -> Value
- execute · function · L83-L187 — async fn execute(&self, input: Value, ctx: ToolContext) -> Result<ToolOutput>
- derive_page_id · function · L190-L216 — fn derive_page_id(path: &Path) -> String
- side_panel_tests · module · L220-L220 — mod side_panel_tests;
