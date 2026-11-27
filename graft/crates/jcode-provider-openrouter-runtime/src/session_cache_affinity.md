# crates/jcode-provider-openrouter-runtime/src/session_cache_affinity.rs

- SessionCacheAffinity · struct · L3-L7 — pub(super) struct SessionCacheAffinity
- new · function · L10-L31 — pub(super) fn new(
- bind · function · L33-L50 — pub(super) fn bind(&self, session: &str)
- key · function · L52-L54 — pub(super) fn key(&self) -> String
- fork · function · L56-L62 — pub(super) fn fork(&self) -> Self
- insert_default · function · L64-L69 — pub(super) fn insert_default(&self, request: &mut Value)
- final_key · function · L71-L101 — pub(super) fn final_key(&self, request: &Value) -> Result<String>
