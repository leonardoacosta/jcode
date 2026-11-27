# crates/jcode-base/src/session/render/response_stats.rs

- is_user_prompt · function · L6-L22 — fn is_user_prompt(message: &StoredMessage) -> bool
- attach · function · L24-L38 — pub(super) fn attach(messages: &[StoredMessage], rendered: &mut [RenderedMessage])
- attach_turn · function · L40-L85 — fn attach_turn(
- tests · module · L88-L246 — mod tests
- text · function · L92-L106 — fn text(role: Role, text: &str, usage: Option<StoredTokenUsage>) -> StoredMessage
- usage · function · L107-L115 — fn usage(input: u64) -> Option<StoredTokenUsage>
- row · function · L116-L126 — fn row(index: usize) -> RenderedMessage
- tool_call · function · L127-L135 — fn tool_call(message: &mut StoredMessage)
- response_stats_aggregate_hidden_tool_rounds_and_separate_user_turns · function · L138-L171 — fn response_stats_aggregate_hidden_tool_rounds_and_separate_user_turns()
- response_stats_unknown_cache_and_old_usage_are_not_zero · function · L174-L199 — fn response_stats_unknown_cache_and_old_usage_are_not_zero()
- response_stats_pending_tool_call_has_no_footer · function · L202-L211 — fn response_stats_pending_tool_call_has_no_footer()
- response_stats_survive_stored_json_and_real_rendering · function · L213-L229 — fn response_stats_survive_stored_json_and_real_rendering()
- response_stats_internal_rows_do_not_split_and_overflow_is_unknown · function · L231-L245 — fn response_stats_internal_rows_do_not_split_and_overflow_is_unknown()
