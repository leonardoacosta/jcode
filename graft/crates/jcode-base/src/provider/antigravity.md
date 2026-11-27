# crates/jcode-base/src/provider/antigravity.rs

- persisted_catalog_path · function · L30-L32 — pub fn persisted_catalog_path() -> Result<std::path::PathBuf>
- load_persisted_catalog · function · L40-L48 — pub fn load_persisted_catalog() -> Option<PersistedCatalog>
- PersistedCatalogRef · struct · L52-L59 — struct PersistedCatalogRef<'a>
- FetchAvailableModelsRequest · struct · L62-L65 — struct FetchAvailableModelsRequest<'a>
- new · function · L68-L72 — fn new(project_id: Option<&'a str>) -> Self
- persist_catalog · function · L76-L96 — pub fn persist_catalog(snapshot: &CatalogSnapshot)
- fetch_available_models_with_project · function · L98-L141 — async fn fetch_available_models_with_project(
- fetch_catalog_snapshot · function · L149-L178 — pub async fn fetch_catalog_snapshot(client: &reqwest::Client) -> Result<CatalogSnapshot>
- tests · module · L181-L250 — mod tests
- borrowed_catalog_matches_legacy_bytes · function · L185-L220 — fn borrowed_catalog_matches_legacy_bytes()
- borrowed_project_matches_legacy_bytes_and_builder_owns_body · function · L223-L249 — fn borrowed_project_matches_legacy_bytes_and_builder_owns_body()
