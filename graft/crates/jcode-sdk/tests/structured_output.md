# crates/jcode-sdk/tests/structured_output.rs

- PairTransport · struct · L14-L14 — struct PairTransport(UnixStream);
- split · function · L17-L22 — fn split(
- fake_harness · function · L25-L55 — fn fake_harness(handle: impl Fn(&ClientFrame, &mut dyn Write) + Send + 'static) -> JcodeClient
- reply · function · L57-L67 — fn reply(frame: &ClientFrame, event: ApiEvent, writer: &mut dyn Write)
- push · function · L69-L79 — fn push(event: ApiEvent, writer: &mut dyn Write)
- send_turn · function · L81-L103 — fn send_turn(writer: &mut dyn Write, session_id: &str, text: &str)
- schema · function · L105-L115 — fn schema() -> serde_json::Value
- Summary · struct · L118-L121 — struct Summary
- validates_fenced_json_and_returns_parsed_data · function · L124-L167 — fn validates_fenced_json_and_returns_parsed_data()
- retries_with_validation_details_then_returns_the_correction · function · L170-L212 — fn retries_with_validation_details_then_returns_the_correction()
- exhaustion_reports_every_bounded_attempt_and_last_parse_error · function · L215-L245 — fn exhaustion_reports_every_bounded_attempt_and_last_parse_error()
- invalid_schema_fails_before_any_model_turn · function · L248-L264 — fn invalid_schema_fails_before_any_model_turn()
- structured_output_uses_final_message_not_process_narration · function · L267-L305 — fn structured_output_uses_final_message_not_process_narration()
