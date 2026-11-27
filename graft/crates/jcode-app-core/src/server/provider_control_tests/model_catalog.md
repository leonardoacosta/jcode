# crates/jcode-app-core/src/server/provider_control_tests/model_catalog.rs

- failed_refresh_still_publishes_successful_partial_catalog · function · L4-L77 — async fn failed_refresh_still_publishes_successful_partial_catalog()
- PartialCatalogProvider · struct · L5-L5 — struct PartialCatalogProvider(std::sync::atomic::AtomicBool);
- complete · function · L8-L16 — async fn complete(
- name · function · L17-L19 — fn name(&self) -> &str
- model · function · L20-L22 — fn model(&self) -> String
- fork · function · L23-L27 — fn fork(&self) -> Arc<dyn Provider>
- available_models_display · function · L28-L34 — fn available_models_display(&self) -> Vec<String>
- refresh_model_catalog · function · L35-L38 — async fn refresh_model_catalog(&self) -> anyhow::Result<ModelCatalogRefreshSummary>
- refresh_models_emits_available_models_updated_after_prefetch · function · L80-L131 — async fn refresh_models_emits_available_models_updated_after_prefetch()
