# crates/jcode-base/src/mobile_sync_incremental_cache.rs

- MAX_SESSIONS · constant · L7-L7 — const MAX_SESSIONS: usize = 32;
- CACHE · constant · L8-L8 — static CACHE: Mutex<VecDeque<Entry>> = Mutex::new(VecDeque::new());
- Entry · struct · L11-L16 — struct Entry
- SourceHasher · struct · L19-L19 — struct SourceHasher(DefaultHasher);
- write · function · L21-L23 — fn write(&mut self, bytes: &[u8])
- finish · function · L24-L26 — fn finish(&self) -> u64
- json · function · L29-L32 — fn json(&mut self, value: &serde_json::Value) -> bool
- bounded_json · function · L34-L89 — fn bounded_json(&mut self, value: &serde_json::Value, budget: &mut usize) -> bool
- digest · function · L91-L93 — fn digest(&self) -> u64
- extend · function · L95-L172 — fn extend(&mut self, messages: &[crate::session::StoredMessage], start: usize) -> bool
- PrefixCache · struct · L176-L176 — pub(super) struct PrefixCache(VecDeque<Entry>);
- remember · function · L178-L201 — fn remember(
- messages · function · L203-L252 — pub(super) fn messages(
- messages · function · L255-L263 — pub(super) fn messages(session: &Session, progress: &SessionProgress) -> (Vec<WireMessage>, u64)
- tests · module · L267-L267 — mod tests;
