# crates/jcode-schema-dialect/src/registry.rs

- OPENAI · constant · L14-L72 — pub const OPENAI: DialectSpec = DialectSpec
- GEMINI · constant · L79-L109 — pub const GEMINI: DialectSpec = DialectSpec
- ANTHROPIC · constant · L113-L166 — pub const ANTHROPIC: DialectSpec = DialectSpec
- OPENROUTER · constant · L171-L180 — pub const OPENROUTER: DialectSpec = DialectSpec
- ANTIGRAVITY_CLAUDE · constant · L191-L203 — pub const ANTIGRAVITY_CLAUDE: DialectSpec = DialectSpec
- ANTIGRAVITY_BRIDGE · constant · L212-L234 — pub const ANTIGRAVITY_BRIDGE: DialectSpec = DialectSpec
- DEFAULT_TRANSFORMS · constant · L236-L245 — const DEFAULT_TRANSFORMS: DialectTransforms = DialectTransforms
- ALL · constant · L248-L255 — pub const ALL: &[&DialectSpec] = &[
- by_id · function · L258-L260 — pub fn by_id(id: &str) -> Option<&'static DialectSpec>
- tests · module · L263-L289 — mod tests
- every_registered_dialect_is_coherent · function · L267-L271 — fn every_registered_dialect_is_coherent()
- dialect_ids_are_unique · function · L274-L280 — fn dialect_ids_are_unique()
- lookup_round_trips · function · L283-L288 — fn lookup_round_trips()
