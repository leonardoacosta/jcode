# crates/jcode-app-core/src/server/provider_control/deferred_operations.rs

- log_provider_control_deferred · function · L3-L15 — pub(super) fn log_provider_control_deferred(operation: &'static str, id: u64) -> Instant
- log_provider_control_lock_acquired · function · L17-L31 — pub(super) fn log_provider_control_lock_acquired(
- log_provider_control_completed · function · L33-L43 — pub(super) fn log_provider_control_completed(operation: &'static str, id: u64, queued_at: Instant)
- spawn_deferred_agent_mutation · function · L45-L61 — pub(super) fn spawn_deferred_agent_mutation<F>(
- spawn_deferred_provider_operation · function · L63-L82 — pub(super) fn spawn_deferred_provider_operation<F>(
- auth_refresh_targets · function · L84-L138 — pub(super) async fn auth_refresh_targets(
- push_unique · function · L90-L97 — fn push_unique(handles: &mut Vec<Arc<dyn Provider>>, provider: Arc<dyn Provider>)
- spawn_deferred_auth_refreshes · function · L140-L151 — pub(super) fn spawn_deferred_auth_refreshes(agents: Vec<Arc<Mutex<Agent>>>)
- apply_auth_runtime_model_to_agent · function · L153-L204 — pub(super) async fn apply_auth_runtime_model_to_agent(
- apply_auth_route_to_agent · function · L206-L256 — pub(super) async fn apply_auth_route_to_agent(
