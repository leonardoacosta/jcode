# crates/jcode-base/src/auth/doctor.rs

- VALIDATION_STALE_AFTER_MS · constant · L4-L4 — pub const VALIDATION_STALE_AFTER_MS: i64 = 7 * 24 * 60 * 60 * 1000;
- native_doctor_supports_provider · function · L18-L33 — pub fn native_doctor_supports_provider(provider_id: &str) -> bool
- validation_is_stale · function · L35-L38 — pub fn validation_is_stale(checked_at_ms: i64) -> bool
- needs_attention · function · L40-L55 — pub fn needs_attention(
- diagnostics · function · L57-L109 — pub fn diagnostics(
- recommended_actions · function · L111-L227 — pub fn recommended_actions(
- tests · module · L230-L345 — mod tests
- base_assessment · function · L236-L255 — fn base_assessment() -> ProviderAuthAssessment
- refresh_failure_marks_available_provider_as_needing_attention · function · L258-L287 — fn refresh_failure_marks_available_provider_as_needing_attention()
- billing_validation_failure_does_not_recommend_reauthentication · function · L290-L321 — fn billing_validation_failure_does_not_recommend_reauthentication()
- stale_validation_marks_provider_as_needing_attention · function · L324-L344 — fn stale_validation_marks_provider_as_needing_attention()
