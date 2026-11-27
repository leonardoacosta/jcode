# crates/jcode-provider-core/src/reasoning.rs

- OPENAI_SELECTABLE_EFFORTS · constant · L8-L10 — pub const OPENAI_SELECTABLE_EFFORTS: &[&str] = &[
- OPENROUTER_SELECTABLE_EFFORTS · constant · L16-L17 — pub const OPENROUTER_SELECTABLE_EFFORTS: &[&str] =
- DEEPSEEK_SELECTABLE_EFFORTS · constant · L20-L20 — pub const DEEPSEEK_SELECTABLE_EFFORTS: &[&str] = &["none", "low", "medium", "high", "max", "swarm"];
- canonical_reasoning_effort · function · L24-L35 — pub fn canonical_reasoning_effort(value: &str) -> Option<&'static str>
- inferred_reasoning_efforts · function · L39-L96 — pub fn inferred_reasoning_efforts(
- tests · module · L99-L143 — mod tests
- provider_ladders_preserve_distinct_max_semantics · function · L103-L118 — fn provider_ladders_preserve_distinct_max_semantics()
- zai_and_glm_identities_expose_openai_efforts · function · L121-L130 — fn zai_and_glm_identities_expose_openai_efforts()
- anthropic_ladder_comes_from_model_capabilities · function · L133-L142 — fn anthropic_ladder_comes_from_model_capabilities()
