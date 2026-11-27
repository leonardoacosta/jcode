# crates/jcode-base/src/provider/tests/catalog_discovery.rs

- catalog_refresh_reaches_every_runtime_slot_and_compatible_profile · function · L5-L117 — fn catalog_refresh_reaches_every_runtime_slot_and_compatible_profile()
- CatalogProbe · struct · L7-L11 — struct CatalogProbe
- complete · function · L14-L22 — async fn complete(
- name · function · L23-L25 — fn name(&self) -> &str
- model · function · L26-L28 — fn model(&self) -> String
- fork · function · L29-L31 — fn fork(&self) -> Arc<dyn Provider>
- available_models_display · function · L32-L34 — fn available_models_display(&self) -> Vec<String>
- prefetch_models · function · L35-L39 — async fn prefetch_models(&self) -> Result<()>
- refresh_model_catalog · function · L40-L47 — async fn refresh_model_catalog(&self) -> Result<ModelCatalogRefreshSummary>
- building_direct_profile_routes_does_not_schedule_catalog_refreshes · function · L127-L161 — fn building_direct_profile_routes_does_not_schedule_catalog_refreshes()
- profile_catalog_cache_needs_refresh_for_missing_cache · function · L166-L177 — fn profile_catalog_cache_needs_refresh_for_missing_cache()
