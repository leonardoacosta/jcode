# crates/jcode-provider-anthropic/src/history_repair.rs

- repair_efficiency_tests · module · L10-L10 — mod repair_efficiency_tests;
- repair_cow_tests · module · L14-L14 — mod repair_cow_tests;
- filter_message_content · function · L18-L48 — fn filter_message_content<'a>(
- hoist_late_tool_results · function · L54-L147 — pub(crate) fn hoist_late_tool_results(messages: &[Message]) -> Vec<Message>
- attached_to_result · function · L155-L186 — pub(crate) fn attached_to_result(blocks: &[ContentBlock], result_idx: usize) -> Vec<usize>
- IMAGE_LABEL_PREFIX · constant · L156-L156 — const IMAGE_LABEL_PREFIX: &str = "[Attached image associated with the preceding tool result:";
- apply_tool_references · function · L195-L273 — pub(crate) fn apply_tool_references(
- rewrite_orphaned_tool_results · function · L284-L375 — pub(crate) fn rewrite_orphaned_tool_results(messages: &mut [ApiMessage])
- DISPLACED_TOOL_RESULT_TEXT · constant · L379-L380 — const DISPLACED_TOOL_RESULT_TEXT: &str =
- answer_unpaired_tool_uses · function · L391-L465 — pub(crate) fn answer_unpaired_tool_uses(messages: &mut Vec<ApiMessage>)
- is_placeholder_tool_result · function · L469-L473 — pub(crate) fn is_placeholder_tool_result(content: &str, is_error: Option<bool>) -> bool
- repair_dangling_anthropic_server_tools · function · L483-L584 — pub(crate) fn repair_dangling_anthropic_server_tools(messages: &[Message]) -> Vec<Message>
- dedupe_tool_uses · function · L590-L628 — pub(crate) fn dedupe_tool_uses(messages: &[Message]) -> Vec<Message>
- dedupe_tool_results · function · L634-L703 — pub(crate) fn dedupe_tool_results(messages: &[Message]) -> Vec<Message>
