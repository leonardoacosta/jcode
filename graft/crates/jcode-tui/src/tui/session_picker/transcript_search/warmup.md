# crates/jcode-tui/src/tui/session_picker/transcript_search/warmup.rs

- QUEUE_CAPACITY · constant · L8-L8 — const QUEUE_CAPACITY: usize = 8;
- MAX_HINTS · constant · L9-L9 — const MAX_HINTS: usize = 32;
- Queue · struct · L11-L14 — struct Queue
- start · function · L16-L38 — fn start() -> Option<Self>
- schedule · function · L40-L57 — fn schedule<'a>(&self, paths: impl Iterator<Item = &'a PathBuf>)
- schedule · function · L60-L65 — pub(super) fn schedule<'a>(paths: impl Iterator<Item = &'a PathBuf>)
- QUEUE · constant · L61-L61 — static QUEUE: OnceLock<Option<Queue>> = OnceLock::new();
- prefetch · function · L71-L95 — pub(super) fn prefetch(paths: Vec<PathBuf>)
- GENERATION · constant · L72-L72 — static GENERATION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
- tests · module · L98-L138 — mod tests
- preparation_survives_query_cancellation_and_is_reusable · function · L104-L120 — fn preparation_survives_query_cancellation_and_is_reusable()
- scheduling_is_bounded_and_deduplicated · function · L123-L137 — fn scheduling_is_bounded_and_deduplicated()
