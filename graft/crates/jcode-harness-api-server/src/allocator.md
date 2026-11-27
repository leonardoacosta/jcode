# crates/jcode-harness-api-server/src/allocator.rs

- DEFAULT_ARENA_MAX · constant · L17-L17 — const DEFAULT_ARENA_MAX: i32 = 2;
- MIN_TRIM_INTERVAL_MS · constant · L19-L19 — const MIN_TRIM_INTERVAL_MS: u64 = 2_000;
- LAST_TRIM_MS · constant · L21-L21 — static LAST_TRIM_MS: AtomicU64 = AtomicU64::new(0);
- configure · function · L24-L34 — pub fn configure()
- trim · function · L37-L54 — pub fn trim()
- request_is_heavy · function · L57-L71 — pub fn request_is_heavy(request: &serde_json::Value) -> bool
- tests · module · L74-L110 — mod tests
- heavy_requests_are_classified · function · L79-L84 — fn heavy_requests_are_classified()
- streaming_events_do_not_enter_block_in_place · function · L87-L101 — fn streaming_events_do_not_enter_block_in_place()
- trim_is_rate_limited_and_safe · function · L104-L109 — fn trim_is_rate_limited_and_safe()
