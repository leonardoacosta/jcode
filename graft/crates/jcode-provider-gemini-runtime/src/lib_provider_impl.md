# crates/jcode-provider-gemini-runtime/src/lib_provider_impl.rs

- complete · function · L5-L345 — async fn complete(
- name · function · L347-L349 — fn name(&self) -> &'static str
- model · function · L351-L356 — fn model(&self) -> String
- supports_image_input · function · L358-L360 — fn supports_image_input(&self) -> bool
- set_model · function · L362-L374 — fn set_model(&self, model: &str) -> Result<()>
- available_models · function · L376-L378 — fn available_models(&self) -> Vec<&'static str>
- available_models_display · function · L380-L396 — fn available_models_display(&self) -> Vec<String>
- available_models_for_switching · function · L398-L400 — fn available_models_for_switching(&self) -> Vec<String>
- model_routes · function · L402-L415 — fn model_routes(&self) -> Vec<jcode_provider_core::ModelRoute>
- prefetch_models · function · L417-L420 — async fn prefetch_models(&self) -> Result<()>
- supports_compaction · function · L422-L430 — fn supports_compaction(&self) -> bool
- fork · function · L432-L439 — fn fork(&self) -> Arc<dyn Provider>
- invalidate_credentials · function · L441-L444 — async fn invalidate_credentials(&self)
