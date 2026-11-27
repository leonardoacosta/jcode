# crates/jcode-app-core/src/wait_registry.rs

- NEXT_TOKEN · constant · L7-L7 — static NEXT_TOKEN: AtomicU64 = AtomicU64::new(1);
- WAITS · constant · L8-L9 — static WAITS: LazyLock<Mutex<HashMap<String, Vec<Wait>>>> =
- CHANGES · constant · L10-L11 — static CHANGES: LazyLock<tokio::sync::watch::Sender<u64>> =
- subscribe_changes · function · L14-L16 — pub fn subscribe_changes() -> tokio::sync::watch::Receiver<u64>
- session_ids · function · L18-L25 — pub fn session_ids() -> Vec<String>
- notify_change · function · L27-L29 — fn notify_change()
- Wait · struct · L31-L35 — struct Wait
- WaitGuard · struct · L39-L42 — pub struct WaitGuard
- register · function · L44-L61 — pub fn register(session_id: &str, reason: &str) -> WaitGuard
- current · function · L64-L71 — pub fn current(session_id: &str) -> Option<(String, u64)>
- drop · function · L74-L84 — fn drop(&mut self)
- tests · module · L88-L161 — mod tests
- entry_and_cleanup_notify_subscribers · function · L92-L104 — async fn entry_and_cleanup_notify_subscribers()
- stale_guard_cannot_erase_new_wait · function · L107-L115 — fn stale_guard_cannot_erase_new_wait()
- overlapping_wait_completion_restores_older_active_wait · function · L118-L127 — fn overlapping_wait_completion_restores_older_active_wait()
- reports_monotonic_age · function · L130-L144 — fn reports_monotonic_age()
- cancellation_clears_wait · function · L147-L160 — async fn cancellation_clears_wait()
