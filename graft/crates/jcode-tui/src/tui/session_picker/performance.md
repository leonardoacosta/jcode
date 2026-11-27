# crates/jcode-tui/src/tui/session_picker/performance.rs

- RETAINED_PREVIEW_BYTES · constant · L8-L8 — pub(super) const RETAINED_PREVIEW_BYTES: usize = 16 * 1024 * 1024;
- RETAINED_PREVIEW_ENTRIES · constant · L9-L9 — pub(super) const RETAINED_PREVIEW_ENTRIES: usize = 8;
- FINGERPRINT_ENTRIES · constant · L10-L10 — const FINGERPRINT_ENTRIES: usize = 16;
- PreviewReuse · struct · L13-L17 — pub(super) struct PreviewReuse
- park · function · L20-L34 — fn park(&mut self, cache: PreviewRenderCache, bytes: usize)
- retained_bytes · function · L36-L38 — pub(super) fn retained_bytes(&self) -> usize
- retained_entries · function · L40-L42 — pub(super) fn retained_entries(&self) -> usize
- estimated_bytes · function · L46-L72 — pub(super) fn estimated_bytes(&self) -> usize
- reuse_preview · function · L76-L102 — pub(super) fn reuse_preview(&mut self, key: &PreviewCacheKey) -> bool
- refresh_preview_query · function · L106-L124 — fn refresh_preview_query(&mut self, query: &str)
- install_preview · function · L126-L130 — pub(super) fn install_preview(&mut self, cache: PreviewRenderCache)
- install_sized_preview · function · L132-L137 — fn install_sized_preview(&mut self, cache: PreviewRenderCache, bytes: usize)
- selected_preview_content_hash · function · L142-L165 — pub(super) fn selected_preview_content_hash(
- transcript_hash · function · L168-L183 — pub(super) fn transcript_hash(session: &SessionInfo) -> u64
- visible_item_bounds · function · L188-L229 — pub(super) fn visible_item_bounds(
