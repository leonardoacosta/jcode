# crates/jcode-app-core/src/server/lane_dispatch.rs

- LaneDispatchContext · struct · L31-L45 — pub(super) struct LaneDispatchContext
- CONTEXT · constant · L47-L47 — static CONTEXT: OnceLock<LaneDispatchContext> = OnceLock::new();
- register_context · function · L49-L55 — pub(super) fn register_context(context: LaneDispatchContext)
- spawn_startup_scan · function · L61-L81 — pub(super) fn spawn_startup_scan(delay: Duration)
- in_flight · function · L86-L89 — fn in_flight() -> &'static StdMutex<HashSet<(String, String)>>
- IN_FLIGHT · constant · L87-L87 — static IN_FLIGHT: OnceLock<StdMutex<HashSet<(String, String)>>> = OnceLock::new();
- queued_sessions · function · L93-L96 — fn queued_sessions() -> &'static StdMutex<HashSet<String>>
- QUEUED · constant · L94-L94 — static QUEUED: OnceLock<StdMutex<HashSet<String>>> = OnceLock::new();
- QUEUED_RETRY_INTERVAL · constant · L98-L98 — const QUEUED_RETRY_INTERVAL: Duration = Duration::from_secs(10);
- dispatchable_lanes · function · L104-L123 — pub(crate) fn dispatchable_lanes(todos: &[TodoItem]) -> Vec<String>
- on_todos_saved · function · L126-L140 — pub(super) fn on_todos_saved(session_id: &str, todos: &[TodoItem])
- dispatch · function · L142-L201 — async fn dispatch(session_id: String, lanes: Vec<String>)
- is_capacity_error · function · L203-L205 — fn is_capacity_error(message: &str) -> bool
- schedule_queued_retry · function · L207-L236 — fn schedule_queued_retry(session_id: &str)
- spawn_lane_worker · function · L238-L302 — async fn spawn_lane_worker(
- tests · module · L306-L306 — mod tests;
