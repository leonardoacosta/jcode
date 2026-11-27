# crates/jcode-app-core/src/running_tool_registry.rs

- RunningTool · struct · L14-L17 — pub struct RunningTool
- NEXT_TOKEN · constant · L19-L19 — static NEXT_TOKEN: AtomicU64 = AtomicU64::new(1);
- RUNNING · constant · L20-L21 — static RUNNING: LazyLock<Mutex<HashMap<String, (u64, RunningTool)>>> =
- RunningToolGuard · struct · L25-L28 — pub struct RunningToolGuard
- register · function · L30-L48 — pub fn register(session_id: &str, call_id: &str, name: &str) -> RunningToolGuard
- current · function · L51-L56 — pub fn current(session_id: &str) -> Option<RunningTool>
- drop · function · L59-L67 — fn drop(&mut self)
- tests · module · L71-L96 — mod tests
- guard_scopes_the_entry_and_newer_registration_wins · function · L75-L95 — fn guard_scopes_the_entry_and_newer_registration_wins()
