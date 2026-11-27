# crates/jcode-base/src/provider/direct_profile_routes.rs

- OPENAI_COMPATIBLE_PROFILE_CATALOG_SOFT_REFRESH_SECS · constant · L6-L6 — pub(crate) const OPENAI_COMPATIBLE_PROFILE_CATALOG_SOFT_REFRESH_SECS: u64 = 15 * 60;
- openai_compatible_profile_catalog_cache_is_stale · function · L8-L10 — pub(super) fn openai_compatible_profile_catalog_cache_is_stale(cached_at: u64, now: u64) -> bool
- cached_live_models_for_openai_compatible_profile · function · L12-L39 — pub(crate) fn cached_live_models_for_openai_compatible_profile(
- direct_openai_compatible_profile_routes · function · L41-L100 — pub(super) fn direct_openai_compatible_profile_routes(
- standard_openrouter_profile_configured · function · L102-L108 — pub(super) fn standard_openrouter_profile_configured() -> bool
- configured_standard_openrouter_profile_routes · function · L110-L134 — pub(super) fn configured_standard_openrouter_profile_routes() -> Vec<ModelRoute>
