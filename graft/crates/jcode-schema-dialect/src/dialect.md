# crates/jcode-schema-dialect/src/dialect.rs

- DialectTransforms · struct · L39-L67 — pub struct DialectTransforms
- DialectSpec · struct · L72-L81 — pub struct DialectSpec
- UNIVERSAL_KEYWORDS · constant · L85-L85 — const UNIVERSAL_KEYWORDS: &[&str] = &["type", "properties", "items", "required", "enum", "title"];
- supports · function · L89-L91 — pub fn supports(&self, key: &str) -> bool
- supports_format · function · L94-L96 — pub fn supports_format(&self, format: &str) -> bool
- validate · function · L102-L120 — pub fn validate(&self) -> Result<(), String>
- LearnedQuirks · struct · L126-L129 — pub struct LearnedQuirks
- is_empty · function · L132-L134 — pub fn is_empty(&self) -> bool
- apply · function · L138-L140 — pub fn apply(schema: &Value, spec: &DialectSpec) -> Value
- apply_with_quirks · function · L143-L168 — pub fn apply_with_quirks(schema: &Value, spec: &DialectSpec, quirks: &LearnedQuirks) -> Value
- walk · function · L170-L265 — fn walk(schema: &Value, spec: &DialectSpec, quirks: &LearnedQuirks) -> Value
- normalize_type_union · function · L268-L300 — fn normalize_type_union(out: &mut Map<String, Value>)
- is_object_typed · function · L302-L308 — fn is_object_typed(map: &Map<String, Value>) -> bool
- prune_dangling_required · function · L311-L326 — fn prune_dangling_required(out: &mut Map<String, Value>)
- flatten_top_level_combiners · function · L331-L399 — fn flatten_top_level_combiners(schema: &mut Value)
- merge_property · function · L404-L413 — fn merge_property(parent: &Value, branch: &Value) -> Value
- flatten_all_combiners · function · L424-L485 — fn flatten_all_combiners(schema: &Value) -> Value
- merge_all_of_branches · function · L498-L565 — fn merge_all_of_branches(out: &mut Map<String, Value>)
