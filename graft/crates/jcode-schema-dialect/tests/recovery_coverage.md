# crates/jcode-schema-dialect/tests/recovery_coverage.rs

- DIALECTS_WITH_RUNTIME_RECOVERY · constant · L19-L24 — const DIALECTS_WITH_RUNTIME_RECOVERY: &[&str] = &[
- DIALECTS_WITHOUT_RUNTIME_RECOVERY · constant · L34-L34 — const DIALECTS_WITHOUT_RUNTIME_RECOVERY: &[&str] = &["openrouter", "anthropic"];
- every_dialect_is_accounted_for_as_having_recovery_or_not · function · L37-L53 — fn every_dialect_is_accounted_for_as_having_recovery_or_not()
- each_recovering_dialect_learns_retries_once_then_refuses_to_loop · function · L62-L105 — fn each_recovering_dialect_learns_retries_once_then_refuses_to_loop()
- no_dialect_absorbs_a_load_bearing_rejection · function · L113-L129 — fn no_dialect_absorbs_a_load_bearing_rejection()
- no_dialect_treats_an_operational_failure_as_a_schema_rejection · function · L135-L153 — fn no_dialect_treats_an_operational_failure_as_a_schema_rejection()
- RUNTIME_RECOVERY_EXPECTATIONS · constant · L163-L181 — const RUNTIME_RECOVERY_EXPECTATIONS: &[(&str, bool)] = &[
- each_runtime_recovery_wiring_matches_what_is_claimed · function · L190-L228 — fn each_runtime_recovery_wiring_matches_what_is_claimed()
