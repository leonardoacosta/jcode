# crates/jcode-provider-gemini-runtime/tests/every_provider_sends_clean_schemas.rs

- hostile_schema · function · L18-L41 — fn hostile_schema() -> Value
- hostile_tool · function · L43-L50 — fn hostile_tool() -> Vec<ToolDefinition>
- contains_key · function · L52-L58 — fn contains_key(value: &Value, key: &str) -> bool
- assert_descriptions_survive · function · L63-L73 — fn assert_descriptions_survive(wire: &Value, provider: &str)
- todo_add_missing_properties_is_repaired_on_gemini_wire_paths · function · L76-L118 — fn todo_add_missing_properties_is_repaired_on_gemini_wire_paths()
- gemini_sends_a_clean_schema_for_the_hostile_tool · function · L121-L143 — fn gemini_sends_a_clean_schema_for_the_hostile_tool()
- every_antigravity_route_sends_a_clean_schema_for_the_hostile_tool · function · L146-L158 — fn every_antigravity_route_sends_a_clean_schema_for_the_hostile_tool()
- openai_sends_a_clean_schema_and_does_not_overclaim_strict · function · L164-L191 — fn openai_sends_a_clean_schema_and_does_not_overclaim_strict()
- openrouter_sends_a_schema_its_strictest_upstream_accepts · function · L197-L225 — fn openrouter_sends_a_schema_its_strictest_upstream_accepts()
- provider_request_builders_that_reach_the_dialect_engine_are_pinned · function · L236-L276 — fn provider_request_builders_that_reach_the_dialect_engine_are_pinned()
- BUILDERS · constant · L238-L249 — const BUILDERS: &[(&str, bool)] = &[
- anthropic_sends_a_schema_without_a_top_level_combiner · function · L282-L341 — fn anthropic_sends_a_schema_without_a_top_level_combiner()
- a_keyword_no_deny_list_has_ever_heard_of_reaches_no_provider · function · L353-L419 — fn a_keyword_no_deny_list_has_ever_heard_of_reaches_no_provider()
- NOVEL · constant · L370-L370 — const NOVEL: &str = "someKeywordFromADraftThatDoesNotExistYet";
