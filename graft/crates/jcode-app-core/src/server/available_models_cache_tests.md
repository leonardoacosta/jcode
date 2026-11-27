# crates/jcode-app-core/src/server/available_models_cache_tests.rs

- GatedProvider · struct · L8-L16 — struct GatedProvider
- BuildGate · type · L18-L21 — type BuildGate = (
- complete · function · L25-L33 — async fn complete(
- name · function · L35-L37 — fn name(&self) -> &str
- model · function · L38-L40 — fn model(&self) -> String
- set_model · function · L41-L44 — fn set_model(&self, model: &str) -> anyhow::Result<()>
- fork · function · L45-L47 — fn fork(&self) -> Arc<dyn Provider>
- explicit_provider_pin_for_current_model · function · L48-L50 — fn explicit_provider_pin_for_current_model(&self) -> Option<String>
- service_tier · function · L51-L53 — fn service_tier(&self) -> Option<String>
- available_models_display · function · L54-L68 — fn available_models_display(&self) -> Vec<String>
- catalog_sharing_identity · function · L69-L80 — fn catalog_sharing_identity(&self) -> Option<String>
- Release · struct · L84-L84 — struct Release(Option<std::sync::mpsc::Sender<()>>);
- drop · function · L86-L90 — fn drop(&mut self)
- offloaded_catalog_releases_agent_and_validates_selection_before_metadata · function · L98-L191 — async fn offloaded_catalog_releases_agent_and_validates_selection_before_metadata()
- offloaded_catalog_join_failure_and_refresh_churn_are_not_empty_successes · function · L194-L220 — async fn offloaded_catalog_join_failure_and_refresh_churn_are_not_empty_successes()
