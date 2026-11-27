# crates/jcode-provider-anthropic/src/deferred_tools_tests.rs

- def · function · L12-L18 — fn def(name: &str) -> ToolDefinition
- msg · function · L20-L27 — fn msg(role: Role, content: Vec<ContentBlock>) -> Message
- tools_json · function · L29-L34 — fn tools_json(tools: &[ApiTool]) -> Vec<serde_json::Value>
- deferred_tools_follow_eager_and_breakpoint_stays_on_last_eager · function · L37-L67 — fn deferred_tools_follow_eager_and_breakpoint_stays_on_last_eager()
- adding_deferred_tools_does_not_change_eager_prefix · function · L70-L82 — fn adding_deferred_tools_does_not_change_eager_prefix()
- all_deferred_list_is_sent_eagerly · function · L85-L91 — fn all_deferred_list_is_sent_eagerly()
- eager_only_lists_serialize_without_defer_loading · function · L94-L98 — fn eager_only_lists_serialize_without_defer_loading()
- reference_conversation · function · L100-L137 — fn reference_conversation() -> Vec<Message>
- tool_reference_renders_as_reference_only_tool_result · function · L140-L169 — fn tool_reference_renders_as_reference_only_tool_result()
- tool_reference_without_available_definition_keeps_plain_result · function · L172-L185 — fn tool_reference_without_available_definition_keeps_plain_result()
- parallel_tool_references_keep_tool_results_contiguous · function · L188-L241 — fn parallel_tool_references_keep_tool_results_contiguous()
