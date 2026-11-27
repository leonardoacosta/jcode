# crates/jcode-app-core/src/agent/model_usage_tests.rs

- UsageProvider · struct · L6-L10 — struct UsageProvider
- routes · function · L13-L26 — fn routes() -> Vec<crate::provider::ModelRoute>
- complete · function · L31-L49 — async fn complete(
- name · function · L50-L52 — fn name(&self) -> &str
- model · function · L53-L55 — fn model(&self) -> String
- model_routes · function · L56-L58 — fn model_routes(&self) -> Vec<crate::provider::ModelRoute>
- active_resolved_credential · function · L59-L61 — fn active_resolved_credential(&self) -> Option<jcode_provider_core::ResolvedCredential>
- fork · function · L62-L64 — fn fork(&self) -> Arc<dyn Provider>
- usage_agent · function · L67-L84 — async fn usage_agent(fail: bool) -> Agent
- both_turn_paths_record_once_across_continuations_and_attribute_serving_model · function · L87-L138 — async fn both_turn_paths_record_once_across_continuations_and_attribute_serving_model()
- failed_requests_and_debug_sessions_do_not_record_turns · function · L141-L156 — async fn failed_requests_and_debug_sessions_do_not_record_turns()
- resumed_session_keeps_turn_identity_until_new_input · function · L159-L222 — async fn resumed_session_keeps_turn_identity_until_new_input()
