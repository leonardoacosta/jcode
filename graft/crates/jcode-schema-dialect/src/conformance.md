# crates/jcode-schema-dialect/src/conformance.rs

- ConformanceError · struct · L20-L24 — pub struct ConformanceError
- untyped_properties · function · L35-L92 — pub fn untyped_properties(schema: &Value) -> Vec<ConformanceError>
- declares_a_type · function · L36-L53 — fn declares_a_type(schema: &Value) -> bool
- walk · function · L55-L87 — fn walk(schema: &Value, path: &str, errors: &mut Vec<ConformanceError>)
- fmt · function · L95-L97 — fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
- must_not_contain_unsupported_constructs · function · L104-L112 — pub fn must_not_contain_unsupported_constructs(
- check_root_shape · function · L120-L142 — fn check_root_shape(schema: &Value, errors: &mut Vec<ConformanceError>)
- walk · function · L144-L214 — fn walk(schema: &Value, spec: &DialectSpec, path: &str, errors: &mut Vec<ConformanceError>)
- check_required · function · L216-L234 — fn check_required(
- must_preserve_meaning · function · L242-L246 — pub fn must_preserve_meaning(original: &Value, normalized: &Value) -> Vec<ConformanceError>
- compare · function · L248-L289 — fn compare(original: &Value, normalized: &Value, path: &str, errors: &mut Vec<ConformanceError>)
- tests · module · L292-L381 — mod tests
- normalizing_a_hostile_schema_satisfies_the_dialect · function · L299-L330 — fn normalizing_a_hostile_schema_satisfies_the_dialect()
- normalization_never_destroys_tool_meaning · function · L333-L358 — fn normalization_never_destroys_tool_meaning()
- the_checker_rejects_an_unnormalized_schema · function · L363-L380 — fn the_checker_rejects_an_unnormalized_schema()
