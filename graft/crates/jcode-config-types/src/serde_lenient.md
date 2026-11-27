# crates/jcode-config-types/src/serde_lenient.rs

- lenient_enum · function · L14-L22 — pub(crate) fn lenient_enum<'de, D, T>(deserializer: D) -> Result<T, D::Error>
- lenient_optional_enum · function · L26-L36 — pub(crate) fn lenient_optional_enum<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
- tests · module · L41-L115 — mod tests
- parse · function · L47-L49 — fn parse(json: &str) -> DisplayConfig
- unknown_diagram_mode_keeps_the_rest_of_the_display_config · function · L52-L68 — fn unknown_diagram_mode_keeps_the_rest_of_the_display_config()
- diagram_mode_inline_is_accepted_as_the_default_mode · function · L73-L77 — fn diagram_mode_inline_is_accepted_as_the_default_mode()
- unknown_reasoning_display_falls_back_without_losing_other_fields · function · L80-L87 — fn unknown_reasoning_display_falls_back_without_losing_other_fields()
- unknown_diff_and_latex_modes_fall_back_to_defaults · function · L90-L99 — fn unknown_diff_and_latex_modes_fall_back_to_defaults()
- legacy_overscroll_status_key_is_ignored · function · L103-L106 — fn legacy_overscroll_status_key_is_ignored()
- valid_values_are_unaffected · function · L110-L114 — fn valid_values_are_unaffected()
