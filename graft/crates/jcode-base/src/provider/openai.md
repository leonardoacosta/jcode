# crates/jcode-base/src/provider/openai.rs

- OPENAI_API_BASE · constant · L16-L16 — const OPENAI_API_BASE: &str = "https://api.openai.com/v1";
- resolve_api_base · function · L29-L78 — pub fn resolve_api_base() -> String
- OVERRIDE_VARS · constant · L30-L34 — const OVERRIDE_VARS: [&str; 3] = [
- codex_config_responses_base · function · L92-L132 — fn codex_config_responses_base() -> Option<String>
- supports_extended_prompt_cache_retention · function · L139-L175 — pub fn supports_extended_prompt_cache_retention(model_id: &str) -> bool
- uses_prompt_cache_options · function · L179-L189 — pub fn uses_prompt_cache_options(model_id: &str) -> bool
- normalize_prompt_cache_retention · function · L193-L199 — pub fn normalize_prompt_cache_retention(value: &str) -> Option<&str>
- prompt_cache_retention_from_env · function · L201-L204 — pub fn prompt_cache_retention_from_env() -> Option<String>
- effective_prompt_cache_retention · function · L207-L218 — pub fn effective_prompt_cache_retention<'a>(
- prompt_cache_ttl_for_model · function · L224-L233 — pub fn prompt_cache_ttl_for_model(model_id: Option<&str>, configured: Option<&str>) -> u64
- cache_tests · module · L236-L359 — mod cache_tests
- extended_cache_retention_matches_documented_models_and_snapshots · function · L240-L274 — fn extended_cache_retention_matches_documented_models_and_snapshots()
- cache_retention_policy_and_ttl_share_override_semantics · function · L277-L303 — fn cache_retention_policy_and_ttl_share_override_semantics()
- cache_ttl_provider_routes_and_environment_override · function · L306-L358 — fn cache_ttl_provider_routes_and_environment_override()
- RestoreEnv · struct · L311-L311 — struct RestoreEnv(&'static str, Option<std::ffi::OsString>);
- drop · function · L313-L318 — fn drop(&mut self)
