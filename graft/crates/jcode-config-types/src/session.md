# crates/jcode-config-types/src/session.rs

- SessionConfig · struct · L7-L13 — pub struct SessionConfig
- HibernationMode · enum · L18-L28 — pub enum HibernationMode
- parse · function · L31-L38 — pub fn parse(value: &str) -> Option<Self>
- default · function · L42-L47 — fn default() -> Self
- tests · module · L51-L86 — mod tests
- missing_setting_defaults_to_thirty_minutes · function · L55-L59 — fn missing_setting_defaults_to_thirty_minutes()
- zero_disables_hibernation_and_roundtrips · function · L62-L72 — fn zero_disables_hibernation_and_roundtrips()
- mode_parses_aliases · function · L75-L85 — fn mode_parses_aliases()
- SessionsConfig · struct · L91-L93 — pub struct SessionsConfig
- SessionRetentionConfig · struct · L98-L106 — pub struct SessionRetentionConfig
- default · function · L109-L117 — fn default() -> Self
