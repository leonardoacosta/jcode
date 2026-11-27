# crates/jcode-app-core/src/server/turn_stop_log.rs

- StopLog · struct · L5-L9 — pub(super) struct StopLog
- provider_reason · function · L12-L24 — fn provider_reason(reason: &str) -> &'static str
- outcome · function · L26-L77 — fn outcome(&mut self, event: &ServerEvent) -> Option<(u64, &'static str)>
- observe · function · L79-L83 — pub(super) fn observe(&mut self, session: &str, identity: &TurnIdentity, event: &ServerEvent)
- needs_investigation · function · L85-L87 — fn needs_investigation(reason: &str) -> bool
- log · function · L89-L114 — fn log(&self, session: &str, identity: &TurnIdentity, request: Option<u64>, reason: &str)
- finish · function · L116-L122 — pub(super) fn finish(&mut self, session: &str, identity: &TurnIdentity)
- tests · module · L126-L288 — mod tests
- provider_aliases_are_known_without_copying_untrusted_strings · function · L130-L139 — fn provider_aliases_are_known_without_copying_untrusted_strings()
- recovered_stop_reason_does_not_taint_later_success · function · L142-L157 — fn recovered_stop_reason_does_not_taint_later_success()
- missing_terminal_is_reported_once_and_is_not_assumed_normal · function · L160-L175 — fn missing_terminal_is_reported_once_and_is_not_assumed_normal()
- expected_intentional_stops_do_not_need_investigation · function · L178-L184 — fn expected_intentional_stops_do_not_need_investigation()
- structured_reasons_override_provider_completion_without_logging_messages · function · L187-L210 — fn structured_reasons_override_provider_completion_without_logging_messages()
- retry_does_not_reuse_previous_provider_completion · function · L213-L223 — fn retry_does_not_reuse_previous_provider_completion()
- provider_subturns_do_not_log_and_final_reason_wins · function · L226-L245 — fn provider_subturns_do_not_log_and_final_reason_wins()
- unknown_success_is_not_normal_and_errors_are_failures · function · L248-L261 — fn unknown_success_is_not_normal_and_errors_are_failures()
- interruption_and_output_limits_are_abnormal · function · L264-L279 — fn interruption_and_output_limits_are_abnormal()
- provider_metadata_is_allowlisted · function · L282-L287 — fn provider_metadata_is_allowlisted()
