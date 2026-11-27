# crates/jcode-base/src/cache_invalidation.rs

- MAX_ENTRIES · constant · L20-L20 — const MAX_ENTRIES: usize = 16;
- DocumentedInvalidation · struct · L24-L31 — pub struct DocumentedInvalidation
- JOURNAL · constant · L33-L33 — static JOURNAL: Mutex<VecDeque<DocumentedInvalidation>> = Mutex::new(VecDeque::new());
- record · function · L40-L57 — pub fn record(source: &'static str, detail: impl Into<String>)
- most_recent_since · function · L64-L73 — pub fn most_recent_since(since: Instant) -> Option<DocumentedInvalidation>
- clear_for_tests · function · L77-L82 — pub fn clear_for_tests()
- tests · module · L85-L95 — mod tests
- most_recent_since_filters_older_entries · function · L89-L94 — fn most_recent_since_filters_older_entries()
