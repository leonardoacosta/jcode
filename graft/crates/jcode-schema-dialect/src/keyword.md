# crates/jcode-schema-dialect/src/keyword.rs

- KeywordRole · enum · L14-L26 — pub enum KeywordRole
- SUBSCHEMA_MAP_KEYWORDS · constant · L29-L35 — const SUBSCHEMA_MAP_KEYWORDS: &[&str] = &[
- SUBSCHEMA_KEYWORDS · constant · L38-L51 — const SUBSCHEMA_KEYWORDS: &[&str] = &[
- SUBSCHEMA_ARRAY_KEYWORDS · constant · L54-L54 — const SUBSCHEMA_ARRAY_KEYWORDS: &[&str] = &["allOf", "anyOf", "oneOf", "prefixItems"];
- DATA_KEYWORDS · constant · L58-L58 — const DATA_KEYWORDS: &[&str] = &["enum", "const", "default", "examples", "required"];
- keyword_role · function · L61-L76 — pub fn keyword_role(key: &str) -> KeywordRole
- LOAD_BEARING_KEYWORDS · constant · L86-L93 — pub const LOAD_BEARING_KEYWORDS: &[&str] = &[
- is_droppable · function · L101-L103 — pub fn is_droppable(key: &str) -> bool
- tests · module · L106-L131 — mod tests
- schema_bearing_keywords_are_classified · function · L110-L115 — fn schema_bearing_keywords_are_classified()
- unknown_keywords_are_data_so_recursion_cannot_corrupt_them · function · L118-L121 — fn unknown_keywords_are_data_so_recursion_cannot_corrupt_them()
- load_bearing_keywords_are_not_droppable · function · L124-L130 — fn load_bearing_keywords_are_not_droppable()
