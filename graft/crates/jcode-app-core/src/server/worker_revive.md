# crates/jcode-app-core/src/server/worker_revive.rs

- WorkerReviveContext · struct · L25-L31 — pub(super) struct WorkerReviveContext
- CONTEXT · constant · L33-L33 — static CONTEXT: OnceLock<WorkerReviveContext> = OnceLock::new();
- register_context · function · L35-L37 — pub(super) fn register_context(context: WorkerReviveContext)
- live_or_revived_agent · function · L43-L148 — pub(super) async fn live_or_revived_agent(
