# crates/jcode-provider-anthropic/src/duplicate_tool_result_tests.rs

- text_msg · function · L16-L27 — fn text_msg(role: Role, text: &str) -> Message
- tool_use · function · L29-L42 — fn tool_use(id: &str) -> Message
- tool_result · function · L44-L56 — fn tool_result(id: &str, content: &str, is_error: Option<bool>) -> Message
- assert_unique_tool_results · function · L59-L71 — fn assert_unique_tool_results(messages: &[ApiMessage])
- placeholder_then_real_output_keeps_only_the_real_output · function · L74-L98 — fn placeholder_then_real_output_keeps_only_the_real_output()
- real_output_then_placeholder_keeps_the_real_output · function · L101-L117 — fn real_output_then_placeholder_keeps_the_real_output()
- duplicate_real_outputs_keep_the_first · function · L120-L134 — fn duplicate_real_outputs_keep_the_first()
- distinct_tool_ids_are_untouched · function · L137-L154 — fn distinct_tool_ids_are_untouched()
- message_left_empty_by_dedupe_is_dropped_and_roles_stay_valid · function · L157-L177 — fn message_left_empty_by_dedupe_is_dropped_and_roles_stay_valid()
- synthetic_interrupt_placeholder_text_is_also_treated_as_a_placeholder · function · L180-L200 — fn synthetic_interrupt_placeholder_text_is_also_treated_as_a_placeholder()
- duplicated_tool_history_run_formats_without_orphaned_tool_use · function · L206-L239 — fn duplicated_tool_history_run_formats_without_orphaned_tool_use()
