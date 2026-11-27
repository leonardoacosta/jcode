# crates/jcode-app-core/src/ambient/runner_mcp_wake.rs

- McpWakeRouter · struct · L18-L27 — pub(super) struct McpWakeRouter
- session_is_runnable · function · L29-L33 — fn session_is_runnable(session_id: &str) -> bool
- install · function · L36-L42 — pub(super) fn install()
- item_id · function · L44-L51 — fn item_id(request: &WakeRequest) -> String
- build_item · function · L53-L84 — pub(crate) fn build_item(request: &WakeRequest) -> ScheduledItem
- route · function · L87-L118 — fn route(&self, request: &WakeRequest) -> WakeOutcome
- push_unique · function · L122-L134 — fn push_unique(mut queue: ScheduledQueue, item: ScheduledItem) -> anyhow::Result<bool>
- wake · function · L137-L139 — fn wake(&self, request: WakeRequest) -> WakeOutcome
- tests · module · L144-L144 — mod tests;
