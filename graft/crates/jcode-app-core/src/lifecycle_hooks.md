# crates/jcode-app-core/src/lifecycle_hooks.rs

- TOOL_RESPONSE_LIMIT · constant · L17-L17 — const TOOL_RESPONSE_LIMIT: usize = 64 * 1024;
- GenerationOwner · struct · L20-L20 — struct GenerationOwner(GenerationToken);
- OwnedHookChild · struct · L22-L22 — struct OwnedHookChild(OwnedChild);
- set_pid · function · L25-L27 — fn set_pid(&mut self, pid: u32)
- own · function · L31-L33 — fn own(&self) -> anyhow::Result<Box<dyn ChildLease>>
- call · function · L37-L49 — pub(crate) fn call(event: LifecycleEvent, session_id: &str, cwd: Option<&str>) -> LifecycleCall
- fire_session_start · function · L52-L61 — pub(crate) fn fire_session_start(session_id: &str, cwd: Option<&str>, source: &str)
- fire_pre_compact · function · L64-L73 — pub(crate) fn fire_pre_compact(session_id: &str, cwd: Option<&str>, trigger: &str)
- fire_post_tool_use · function · L76-L108 — pub(crate) fn fire_post_tool_use(
- run_pre_tool_use · function · L111-L133 — pub(crate) async fn run_pre_tool_use(
- with_report_parts · function · L143-L159 — pub(crate) async fn with_report_parts<F: std::future::Future>(
- fire_subagent_stop · function · L163-L186 — pub(crate) fn fire_subagent_stop(
- STOP_CONTINUATIONS_PER_TURN · constant · L189-L189 — pub(crate) const STOP_CONTINUATIONS_PER_TURN: u32 = 3;
- run_stop · function · L193-L210 — pub(crate) async fn run_stop(
