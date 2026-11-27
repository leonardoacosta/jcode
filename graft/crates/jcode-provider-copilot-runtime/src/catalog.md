# crates/jcode-provider-copilot-runtime/src/catalog.rs

- PersistedCatalogRef · struct · L6-L9 — struct PersistedCatalogRef<'a>
- persisted_catalog_path · function · L12-L14 — pub(super) fn persisted_catalog_path() -> Result<std::path::PathBuf>
- load_persisted_catalog · function · L16-L40 — pub(super) fn load_persisted_catalog() -> Option<PersistedCatalog>
- persist_catalog · function · L42-L60 — pub(super) fn persist_catalog(models: &[String])
- seed_cached_catalog · function · L62-L71 — pub(super) fn seed_cached_catalog(&self)
- model_catalog_detail_impl · function · L73-L84 — pub(super) fn model_catalog_detail_impl(&self) -> String
- tests · module · L88-L121 — mod tests
- persisted_catalog_ref_matches_legacy_json_and_borrows_models · function · L92-L120 — fn persisted_catalog_ref_matches_legacy_json_and_borrows_models()
