# crates/jcode-provider-anthropic/src/trailing_assistant_repair_tests.rs

- text_msg · function · L12-L23 — fn text_msg(role: Role, text: &str) -> Message
- roles · function · L25-L27 — fn roles(messages: &[ApiMessage]) -> Vec<&str>
- transcript_ending_on_assistant_gets_a_continuation_user_turn · function · L30-L45 — fn transcript_ending_on_assistant_gets_a_continuation_user_turn()
- transcript_already_ending_on_user_is_left_alone · function · L48-L63 — fn transcript_already_ending_on_user_is_left_alone()
- dangling_tool_use_repair_already_ends_on_user_and_is_not_double_patched · function · L66-L91 — fn dangling_tool_use_repair_already_ends_on_user_and_is_not_double_patched()
