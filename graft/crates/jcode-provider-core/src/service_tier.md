# crates/jcode-provider-core/src/service_tier.rs

- SPEED_TIER_STANDARD · constant · L13-L13 — pub const SPEED_TIER_STANDARD: &str = "off";
- SPEED_TIER_FAST · constant · L15-L15 — pub const SPEED_TIER_FAST: &str = "priority";
- SPEED_TIER_ULTRAFAST · constant · L17-L17 — pub const SPEED_TIER_ULTRAFAST: &str = "ultrafast";
- STANDARD_FAST · constant · L19-L19 — const STANDARD_FAST: &[&str] = &[SPEED_TIER_STANDARD, SPEED_TIER_FAST];
- STANDARD_FAST_ULTRAFAST · constant · L20-L21 — const STANDARD_FAST_ULTRAFAST: &[&str] =
- base_model · function · L23-L28 — fn base_model(model: &str) -> String
- openai_model_supports_ultrafast · function · L35-L43 — pub fn openai_model_supports_ultrafast(model: &str) -> bool
- canonical_speed_tier · function · L46-L56 — pub fn canonical_speed_tier(tier: Option<&str>) -> &'static str
- speed_tier_ladder · function · L61-L112 — pub fn speed_tier_ladder(
- speed_tier_ladder_with_capabilities · function · L117-L132 — pub fn speed_tier_ladder_with_capabilities(
- step_speed_tier · function · L138-L166 — pub fn step_speed_tier(
- tests · module · L169-L295 — mod tests
- authoritative_capabilities_filter_without_adding_flex · function · L173-L199 — fn authoritative_capabilities_filter_without_adding_flex()
- openai_ladder_includes_ultrafast_only_for_supported_models · function · L202-L219 — fn openai_ladder_includes_ultrafast_only_for_supported_models()
- sol_ultrafast_accepts_qualified_snapshots_and_context_suffixes · function · L222-L241 — fn sol_ultrafast_accepts_qualified_snapshots_and_context_suffixes()
- other_providers_get_fast_toggle_or_nothing · function · L244-L256 — fn other_providers_get_fast_toggle_or_nothing()
- stepping_clamps_at_both_ends · function · L259-L279 — fn stepping_clamps_at_both_ends()
- unsupported_current_tier_steps_down_from_the_top_rung · function · L282-L294 — fn unsupported_current_tier_steps_down_from_the_top_rung()
