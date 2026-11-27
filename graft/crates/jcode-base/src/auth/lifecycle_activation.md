# crates/jcode-base/src/auth/lifecycle_activation.rs

- activate_auth_change · function · L3-L23 — pub fn activate_auth_change(request: &AuthActivationRequest) -> AuthActivationResult
- api_key_env_bindings_for_provider · function · L28-L89 — pub(super) fn api_key_env_bindings_for_provider(provider_id: &str) -> Vec<(String, String)>
- sync_process_env_from_saved_credentials · function · L102-L171 — pub(super) fn sync_process_env_from_saved_credentials(
- apply_auth_provider_runtime · function · L173-L237 — pub(super) fn apply_auth_provider_runtime(provider_id: Option<&str>) -> Option<String>
- direct_provider_activation · function · L239-L255 — pub(super) fn direct_provider_activation(provider_id: &str) -> Option<ProviderActivation>
- config_has_explicit_default_provider · function · L257-L265 — pub(super) fn config_has_explicit_default_provider() -> bool
- direct_provider_runtime_ids · function · L267-L285 — pub(super) fn direct_provider_runtime_ids(
