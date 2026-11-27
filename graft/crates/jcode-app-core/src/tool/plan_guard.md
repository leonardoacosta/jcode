# crates/jcode-app-core/src/tool/plan_guard.rs

- PlanModeDenied · struct · L5-L5 — pub struct PlanModeDenied(pub String);
- fmt · function · L7-L9 — fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
- PLAN_TURNS · constant · L13-L14 — static PLAN_TURNS: LazyLock<StdRwLock<HashMap<String, usize>>> =
- PlanTurnGuard · struct · L17-L17 — pub struct PlanTurnGuard(String);
- new · function · L19-L26 — pub fn new(session: &str) -> Self
- drop · function · L29-L37 — fn drop(&mut self)
- plan_turn_active · function · L39-L44 — pub fn plan_turn_active(session: &str) -> bool
- admit_plan_tool · function · L46-L84 — pub(crate) fn admit_plan_tool(name: &str, input: &Value, custom: bool) -> Result<()>
- plan_bash_argv · function · L88-L238 — pub(crate) fn plan_bash_argv(command: &str) -> Result<Vec<String>>
- execute_plan_bash · function · L240-L385 — pub(crate) async fn execute_plan_bash(input: &Value, ctx: &ToolContext) -> Result<ToolOutput>
