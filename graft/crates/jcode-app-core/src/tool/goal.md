# crates/jcode-app-core/src/tool/goal.rs

- InitiativeTool · struct · L10-L10 — pub struct InitiativeTool;
- new · function · L13-L15 — pub fn new() -> Self
- default_display_for_action · function · L18-L26 — fn default_display_for_action(action: &str) -> crate::goal::GoalDisplayMode
- publish_side_panel_snapshot · function · L28-L33 — fn publish_side_panel_snapshot(session_id: &str, snapshot: &crate::side_panel::SidePanelSnapshot)
- maybe_publish_goals_overview_refresh · function · L35-L45 — fn maybe_publish_goals_overview_refresh(
- goal_page_is_open · function · L47-L51 — fn goal_page_is_open(session_id: &str, goal_id: &str) -> Result<bool>
- GoalInput · struct · L54-L84 — struct GoalInput
- goal_step_schema · function · L86-L91 — fn goal_step_schema() -> Value
- goal_milestone_schema · function · L93-L104 — fn goal_milestone_schema() -> Value
- name · function · L108-L110 — fn name(&self) -> &str
- description · function · L112-L114 — fn description(&self) -> &str
- parameters_schema · function · L116-L142 — fn parameters_schema(&self) -> Value
- execute · function · L144-L360 — async fn execute(&self, input: Value, ctx: ToolContext) -> Result<ToolOutput>
- goal_tests · module · L365-L365 — mod goal_tests;
