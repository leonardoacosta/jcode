# crates/jcode-provider-core/src/catalog_snapshot_tests.rs

- CatalogProvider · struct · L4-L4 — struct CatalogProvider;
- complete · function · L8-L16 — async fn complete(
- name · function · L17-L19 — fn name(&self) -> &str
- model · function · L20-L22 — fn model(&self) -> String
- available_models_display · function · L23-L25 — fn available_models_display(&self) -> Vec<String>
- fork · function · L26-L28 — fn fork(&self) -> Arc<dyn Provider>
- CombinedCatalogProvider · struct · L32-L32 — struct CombinedCatalogProvider;
- complete · function · L36-L44 — async fn complete(
- name · function · L45-L47 — fn name(&self) -> &str
- available_models_display · function · L48-L50 — fn available_models_display(&self) -> Vec<String>
- model_routes · function · L51-L53 — fn model_routes(&self) -> Vec<ModelRoute>
- model_catalog_snapshot · function · L54-L61 — fn model_catalog_snapshot(&self) -> ModelCatalogSnapshot
- fork · function · L62-L64 — fn fork(&self) -> Arc<dyn Provider>
- default_catalog_snapshot_preserves_legacy_fields · function · L68-L77 — fn default_catalog_snapshot_preserves_legacy_fields()
- catalog_snapshot_dispatches_combined_hook_through_trait_object · function · L80-L93 — fn catalog_snapshot_dispatches_combined_hook_through_trait_object()
