# crates/jcode-provider-gemini-runtime/src/catalog.rs

- PersistedCatalogRef · struct · L6-L9 — struct PersistedCatalogRef<'a>
- persisted_catalog_path · function · L12-L14 — pub(super) fn persisted_catalog_path() -> Result<std::path::PathBuf>
- load_persisted_catalog · function · L16-L40 — pub(super) fn load_persisted_catalog() -> Option<PersistedCatalog>
- persist_catalog · function · L42-L60 — pub(super) fn persist_catalog(models: &[String])
- seed_cached_catalog · function · L62-L68 — pub(super) fn seed_cached_catalog(&self)
- refresh_available_models · function · L70-L99 — pub(super) async fn refresh_available_models(&self) -> Result<Vec<String>>
- refresh_available_models_api_key · function · L104-L166 — pub(super) async fn refresh_available_models_api_key(
- tests · module · L170-L203 — mod tests
- persisted_catalog_ref_matches_legacy_json_and_borrows_models · function · L174-L202 — fn persisted_catalog_ref_matches_legacy_json_and_borrows_models()
