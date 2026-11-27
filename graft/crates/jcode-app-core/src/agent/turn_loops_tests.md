# crates/jcode-app-core/src/agent/turn_loops_tests.rs

- user_text · function · L3-L14 — fn user_text(text: &str) -> Message
- tool_result · function · L16-L28 — fn tool_result(id: &str, content: &str) -> Message
- messages_end_with_tool_result_detects_tool_continuation_context · function · L31-L39 — fn messages_end_with_tool_result_detects_tool_continuation_context()
- messages_end_with_tool_result_allows_memory_after_tool_results · function · L42-L50 — fn messages_end_with_tool_result_allows_memory_after_tool_results()
- recovery_reminder_alone_is_not_a_tool_result · function · L53-L68 — fn recovery_reminder_alone_is_not_a_tool_result()
- messages_end_with_tool_result_ignores_plain_user_prompt · function · L71-L75 — fn messages_end_with_tool_result_ignores_plain_user_prompt()
- sequential_tool_rounds_trigger_after_three_single_calls · function · L78-L85 — fn sequential_tool_rounds_trigger_after_three_single_calls()
- parallel_or_batch_calls_reset_sequential_tool_rounds · function · L88-L92 — fn parallel_or_batch_calls_reset_sequential_tool_rounds()
- pending_nudge_is_injected_only_when_batch_is_available · function · L95-L101 — fn pending_nudge_is_injected_only_when_batch_is_available()
- plan_limit_reminder_relays_upgrade_link_without_purchasing · function · L104-L117 — fn plan_limit_reminder_relays_upgrade_link_without_purchasing()
