# crates/jcode-provider-doctor/src/live_provider_probes_tool_replay.rs

- parse_tool_arguments · function · L5-L11 — pub(super) fn parse_tool_arguments(input_json: &str) -> serde_json::Value
- assistant_tool_use · function · L16-L32 — pub(super) fn assistant_tool_use(
- tool_result_then_text · function · L36-L48 — pub(super) fn tool_result_then_text(tool_use_id: &str, result: &str) -> Message
- assistant_parallel_tool_uses · function · L55-L71 — pub(super) fn assistant_parallel_tool_uses(calls: &[NativeClaudeToolCall]) -> Message
- parallel_tool_results · function · L76-L92 — pub(super) fn parallel_tool_results(calls: &[NativeClaudeToolCall]) -> Message
