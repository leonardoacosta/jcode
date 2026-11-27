# crates/jcode-base/src/auth/integration.rs

- AuthProviderIntegration · struct · L14-L17 — pub struct AuthProviderIntegration
- id · function · L20-L22 — pub fn id(self) -> &'static str
- supports_surface · function · L24-L26 — pub fn supports_surface(self, surface: LoginProviderSurface) -> bool
- runtime_key · function · L28-L30 — pub fn runtime_key(self) -> Option<&'static str>
- auth_provider_integrations · function · L33-L42 — pub fn auth_provider_integrations() -> Vec<AuthProviderIntegration>
- auth_provider_integration · function · L44-L50 — pub fn auth_provider_integration(provider_id_or_alias: &str) -> Option<AuthProviderIntegration>
- runtime_id_for_login_provider · function · L52-L74 — pub fn runtime_id_for_login_provider(
- tests · module · L77-L157 — mod tests
- integration_registry_covers_login_catalog_once · function · L85-L102 — fn integration_registry_covers_login_catalog_once()
- integration_registry_preserves_runtime_identity · function · L105-L115 — fn integration_registry_preserves_runtime_identity()
- integration_surface_support_matches_catalog_order · function · L118-L138 — fn integration_surface_support_matches_catalog_order()
- model_auth_status_providers_have_runtime_identity · function · L141-L156 — fn model_auth_status_providers_have_runtime_identity()
