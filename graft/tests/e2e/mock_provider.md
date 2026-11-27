# tests/e2e/mock_provider.rs

- MockProvider · struct · L12-L22 — pub struct MockProvider
- new · function · L25-L34 — pub fn new() -> Self
- with_models · function · L36-L49 — pub fn with_models(models: Vec<&'static str>) -> Self
- queue_response · function · L52-L54 — pub fn queue_response(&self, events: Vec<StreamEvent>)
- complete · function · L59-L91 — async fn complete(
- name · function · L93-L95 — fn name(&self) -> &str
- model · function · L97-L99 — fn model(&self) -> String
- set_model · function · L101-L107 — fn set_model(&self, model: &str) -> Result<()>
- available_models · function · L109-L111 — fn available_models(&self) -> Vec<&'static str>
- fork · function · L113-L123 — fn fork(&self) -> Arc<dyn Provider>
