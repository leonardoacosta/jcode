# crates/jcode-base/src/auth/refresh_coordinator.rs

- REFRESH_LOCKS · constant · L19-L20 — static REFRESH_LOCKS: LazyLock<Mutex<HashMap<String, Arc<tokio::sync::Mutex<()>>>>> =
- cancellation · function · L26-L28 — pub(crate) fn cancellation() -> Option<tokio::sync::watch::Receiver<()>>
- inherit_cancellation · function · L30-L39 — pub(crate) async fn inherit_cancellation<T>(
- with_cancellation · function · L43-L63 — pub async fn with_cancellation<T>(
- ensure_not_cancelled · function · L65-L70 — pub(crate) fn ensure_not_cancelled() -> anyhow::Result<()>
- lock_for · function · L72-L77 — fn lock_for(key: &str) -> Arc<tokio::sync::Mutex<()>>
- FRESHNESS_MARGIN_MS · constant · L81-L81 — pub const FRESHNESS_MARGIN_MS: i64 = 60_000;
- expiry_is_fresh · function · L84-L86 — pub fn expiry_is_fresh(expires_at_ms: i64) -> bool
- single_flight · function · L95-L128 — pub async fn single_flight<T, Reload, Fresh, Refresh, Fut>(
- tests · module · L131-L348 — mod tests
- cancelled_lock_wait_never_dispatches_refresh · function · L136-L174 — async fn cancelled_lock_wait_never_dispatches_refresh()
- cancelled_after_dispatch_finishes_refresh_transaction · function · L177-L212 — async fn cancelled_after_dispatch_finishes_refresh_transaction()
- cancelled_ready_lock_never_polls_exchange · function · L215-L229 — async fn cancelled_ready_lock_never_polls_exchange()
- aborted_nested_transaction_waiter_never_dispatches · function · L232-L266 — async fn aborted_nested_transaction_waiter_never_dispatches()
- concurrent_refreshes_for_same_key_run_once · function · L269-L303 — async fn concurrent_refreshes_for_same_key_run_once()
- refresh_uses_newest_stored_state · function · L306-L322 — async fn refresh_uses_newest_stored_state()
- distinct_keys_do_not_serialize · function · L325-L347 — async fn distinct_keys_do_not_serialize()
