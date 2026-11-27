# crates/jcode-app-core/src/agent/tool_handoff.rs

- foreground_budget · function · L3-L5 — pub(super) fn foreground_budget() -> Duration
- automatic_handoff_reason · function · L7-L9 — pub(super) fn automatic_handoff_reason(budget: Duration) -> String
- auto_background_allowed · function · L11-L15 — pub(super) fn auto_background_allowed(name: &str, input: &serde_json::Value) -> bool
- handoff_display_name · function · L19-L23 — pub(super) fn handoff_display_name(input: &serde_json::Value) -> Option<String>
- execute_with_background · function · L26-L33 — pub(in crate::agent) async fn execute_with_background(
- execute_with_background_budget · function · L35-L75 — async fn execute_with_background_budget(
- wait_for_tool_handoff · function · L78-L90 — pub(super) async fn wait_for_tool_handoff(
- handoff_tests · module · L94-L94 — mod handoff_tests;
- reload_interrupted_tool_result · function · L96-L127 — pub(super) fn reload_interrupted_tool_result(tc: &ToolCall, elapsed_secs: f64) -> (String, bool)
- incomplete_turn_stop · function · L130-L145 — pub(super) fn incomplete_turn_stop(stop_reason: Option<&str>) -> Option<ServerEvent>
