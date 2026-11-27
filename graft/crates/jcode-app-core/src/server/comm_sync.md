# crates/jcode-app-core/src/server/comm_sync.rs

- CommResyncPlanContext · struct · L14-L23 — pub(super) struct CommResyncPlanContext<'a>
- live_activity_snapshot · function · L25-L60 — fn live_activity_snapshot(
- SWARM_LIST_TOKEN_WINDOW_SECS · constant · L64-L64 — pub(super) const SWARM_LIST_TOKEN_WINDOW_SECS: u64 = 10;
- MemberRuntimeExtras · struct · L70-L83 — pub(super) struct MemberRuntimeExtras
- member_runtime_extras · function · L89-L147 — pub(super) async fn member_runtime_extras(
- ensure_same_swarm_access · function · L149-L181 — async fn ensure_same_swarm_access(
- can_read_full_context · function · L183-L197 — async fn can_read_full_context(
- handle_comm_summary · function · L199-L248 — pub(super) async fn handle_comm_summary(
- control_snapshot · function · L252-L280 — pub(super) fn control_snapshot(session_id: &str) -> Option<crate::protocol::ControlStatusSnapshot>
- handle_comm_status · function · L286-L398 — pub(super) async fn handle_comm_status(
- handle_comm_read_context · function · L400-L456 — pub(super) async fn handle_comm_read_context(
- handle_comm_plan_status · function · L458-L492 — pub(super) async fn handle_comm_plan_status(
- handle_comm_resync_plan · function · L494-L573 — pub(super) async fn handle_comm_resync_plan(
