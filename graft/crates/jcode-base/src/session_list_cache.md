# crates/jcode-base/src/session_list_cache.rs

- Invalidator · type · L11-L11 — type Invalidator = fn();
- INVALIDATORS · constant · L13-L13 — static INVALIDATORS: LazyLock<RwLock<Vec<Invalidator>>> = LazyLock::new(|| RwLock::new(Vec::new()));
- register_invalidator · function · L20-L25 — pub fn register_invalidator(invalidator: Invalidator)
- invalidate · function · L29-L36 — pub fn invalidate()
