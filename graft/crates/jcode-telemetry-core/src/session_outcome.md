# crates/jcode-telemetry-core/src/session_outcome.rs

- min_optional_ms · function · L5-L7 — pub(super) fn min_optional_ms(values: impl IntoIterator<Item = Option<u64>>) -> Option<u64>
- time_to_first_agent_action_ms · function · L9-L17 — pub(super) fn time_to_first_agent_action_ms(state: &SessionTelemetry) -> Option<u64>
- time_to_first_useful_action_ms · function · L19-L26 — pub(super) fn time_to_first_useful_action_ms(state: &SessionTelemetry) -> Option<u64>
- infer_agent_role · function · L28-L38 — pub(super) fn infer_agent_role(state: &SessionTelemetry) -> &'static str
- infer_session_stop_reason · function · L41-L98 — pub(super) fn infer_session_stop_reason(
