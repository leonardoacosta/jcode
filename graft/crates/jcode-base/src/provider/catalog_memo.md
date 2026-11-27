# crates/jcode-base/src/provider/catalog_memo.rs

- RoutesMemoEntry · struct · L9-L19 — pub(super) struct RoutesMemoEntry
- GLOBAL_ROUTES_MEMO · constant · L31-L32 — static GLOBAL_ROUTES_MEMO: LazyLock<Mutex<HashMap<String, Arc<RoutesMemoEntry>>>> =
- GLOBAL_ROUTES_BUILD_LOCK · constant · L40-L40 — static GLOBAL_ROUTES_BUILD_LOCK: Mutex<()> = Mutex::new(());
- CATALOG_GENERATION · constant · L44-L44 — static CATALOG_GENERATION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
- catalog_generation · function · L46-L48 — pub(super) fn catalog_generation() -> u64
- bump_catalog_generation · function · L56-L58 — pub fn bump_catalog_generation()
- next_catalog_scope · function · L60-L63 — pub(super) fn next_catalog_scope() -> u64
- NEXT · constant · L61-L61 — static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
- invalidate_routes_memo · function · L69-L73 — pub(super) fn invalidate_routes_memo(&self)
- invalidate_routes_memo_globally · function · L80-L83 — pub(super) fn invalidate_routes_memo_globally(&self)
- routes_memo_key · function · L92-L94 — pub(super) fn routes_memo_key(&self) -> String
- routes_memo_key_unscoped · function · L99-L101 — pub(super) fn routes_memo_key_unscoped(&self) -> String
- routes_memo_key_with_scope · function · L103-L177 — fn routes_memo_key_with_scope(&self, catalog_scope: Option<u64>) -> String
- CATALOG_CLOCK · constant · L135-L135 — static CATALOG_CLOCK: LazyLock<std::time::Instant> = LazyLock::new(std::time::Instant::now);
- fresh_routes_memo_entry · function · L186-L297 — pub(super) fn fresh_routes_memo_entry(&self) -> Arc<RoutesMemoEntry>
- ROUTES_MEMO_TTL · constant · L189-L189 — const ROUTES_MEMO_TTL: std::time::Duration = std::time::Duration::from_secs(60);
