# crates/jcode-provider-bedrock/src/catalog.rs

- BorrowedCatalog · struct · L6-L16 — struct BorrowedCatalog<'a>
- serialize_model_set · function · L18-L25 — fn serialize_model_set<S: serde::Serializer>(
- persisted_catalog_path · function · L28-L30 — pub(super) fn persisted_catalog_path() -> Result<std::path::PathBuf>
- load_persisted_catalog · function · L32-L52 — pub(super) fn load_persisted_catalog() -> Option<PersistedCatalog>
- persist_catalog · function · L57-L83 — pub(super) fn persist_catalog(
- seed_cached_catalog · function · L85-L124 — pub(super) fn seed_cached_catalog(&self)
- tests · module · L128-L205 — mod tests
- borrowed_catalog_preserves_exact_legacy_json · function · L132-L204 — fn borrowed_catalog_preserves_exact_legacy_json()
