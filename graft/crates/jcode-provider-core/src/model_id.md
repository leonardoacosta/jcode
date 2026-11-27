# crates/jcode-provider-core/src/model_id.rs

- LONG_CONTEXT_SUFFIX · constant · L15-L15 — pub const LONG_CONTEXT_SUFFIX: &str = "[1m]";
- strip_long_context_suffix · function · L18-L20 — pub fn strip_long_context_suffix(model: &str) -> &str
- split_long_context · function · L24-L29 — pub fn split_long_context(model: &str) -> (&str, bool)
- canonical · function · L35-L38 — pub fn canonical(model: &str) -> String
- strip_date_suffix · function · L43-L50 — pub fn strip_date_suffix(model: &str) -> &str
- slash_base · function · L55-L57 — pub fn slash_base(model: &str) -> &str
- matches_known_model · function · L62-L74 — pub fn matches_known_model(model: &str, known: &[&str]) -> bool
- tests · module · L77-L126 — mod tests
- canonical_lowercases_trims_and_strips_long_context · function · L81-L84 — fn canonical_lowercases_trims_and_strips_long_context()
- split_long_context_reports_suffix · function · L87-L96 — fn split_long_context_reports_suffix()
- strip_date_suffix_only_strips_8_digit_dates · function · L99-L106 — fn strip_date_suffix_only_strips_8_digit_dates()
- slash_base_takes_last_segment · function · L109-L115 — fn slash_base_takes_last_segment()
- matches_known_model_is_case_insensitive_and_1m_tolerant · function · L118-L125 — fn matches_known_model_is_case_insensitive_and_1m_tolerant()
