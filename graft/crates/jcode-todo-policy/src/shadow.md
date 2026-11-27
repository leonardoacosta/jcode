# crates/jcode-todo-policy/src/shadow.rs

- MAX_SHADOW_REPLAY_BYTES · constant · L6-L6 — pub const MAX_SHADOW_REPLAY_BYTES: usize = 4 * 1024 * 1024;
- ShadowVerdict · enum · L10-L14 — pub enum ShadowVerdict
- ShadowRecord · struct · L19-L27 — pub struct ShadowRecord
- ShadowReplayReport · struct · L30-L39 — pub struct ShadowReplayReport
- replay_shadow_json · function · L42-L112 — pub fn replay_shadow_json(input: &[u8]) -> Result<ShadowReplayReport, String>
- tests · module · L115-L282 — mod tests
- row · function · L119-L134 — fn row() -> Value
- semantic_shadow_replay_reports_cost_and_redundancy_without_savings_claims · function · L137-L145 — fn semantic_shadow_replay_reports_cost_and_redundancy_without_savings_claims()
- semantic_shadow_replay_dropped_required_checks_are_unknown_never_redundant · function · L148-L215 — fn semantic_shadow_replay_dropped_required_checks_are_unknown_never_redundant()
- semantic_shadow_replay_rejects_missing_evidence_and_overflow · function · L218-L247 — fn semantic_shadow_replay_rejects_missing_evidence_and_overflow()
- semantic_shadow_replay_controls_failed_missing_waits_blocked_and_premature_stop · function · L250-L281 — fn semantic_shadow_replay_controls_failed_missing_waits_blocked_and_premature_stop()
