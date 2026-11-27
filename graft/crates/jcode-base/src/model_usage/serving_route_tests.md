# crates/jcode-base/src/model_usage/serving_route_tests.rs

- Probe · struct · L11-L19 — struct Probe
- complete · function · L22-L30 — async fn complete(
- name · function · L31-L33 — fn name(&self) -> &str
- display_name · function · L34-L36 — fn display_name(&self) -> String
- model · function · L37-L39 — fn model(&self) -> String
- active_resolved_credential · function · L40-L42 — fn active_resolved_credential(&self) -> Option<ResolvedCredential>
- explicit_provider_pin_for_current_model · function · L43-L45 — fn explicit_provider_pin_for_current_model(&self) -> Option<String>
- model_routes · function · L46-L49 — fn model_routes(&self) -> Vec<ModelRoute>
- fork · function · L50-L52 — fn fork(&self) -> Arc<dyn Provider>
- old · function · L56-L102 — fn old(provider: &dyn Provider, selected_method: Option<&str>) -> Option<ModelRoute>
- row · function · L103-L113 — fn row(model: &str, provider: &str, method: &str) -> ModelRoute
- probe · function · L114-L124 — fn probe() -> Probe
- parity · function · L125-L135 — fn parity(p: &Probe, selected: Option<&str>)
- exact_parity_aliases_pins_models_credentials_and_ambiguity · function · L137-L256 — fn exact_parity_aliases_pins_models_credentials_and_ambiguity()
- paired_serving_route_benchmark · function · L260-L299 — fn paired_serving_route_benchmark()
