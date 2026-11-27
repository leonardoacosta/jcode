# crates/jcode-app-core/src/agent/tools.rs

- MAX_TOOL_OUTPUT_CHARS_FOR_HISTORY · constant · L5-L5 — pub(super) const MAX_TOOL_OUTPUT_CHARS_FOR_HISTORY: usize = 512 * 1024;
- cap_tool_output_for_history · function · L7-L19 — pub(super) fn cap_tool_output_for_history(tool_name: &str, mut output: ToolOutput) -> ToolOutput
- cap_sdk_tool_content_for_history · function · L21-L31 — pub(super) fn cap_sdk_tool_content_for_history(tool_name: &str, content: String) -> String
- tool_output_side_pane_images · function · L40-L76 — pub(super) fn tool_output_side_pane_images(
- TOOL_REFERENCES_METADATA_KEY · constant · L80-L80 — pub(crate) const TOOL_REFERENCES_METADATA_KEY: &str = "tool_references";
- tool_output_references · function · L83-L97 — pub(crate) fn tool_output_references(output: &ToolOutput) -> Vec<String>
- tool_output_to_content_blocks · function · L99-L136 — pub(super) fn tool_output_to_content_blocks(
- print_tool_summary · function · L138-L162 — pub(super) fn print_tool_summary(tool: &ToolCall)
- tests · module · L165-L210 — mod tests
- authoritative_diff_text_survives_history_conversion_and_serialization · function · L169-L182 — fn authoritative_diff_text_survives_history_conversion_and_serialization()
- cap_tool_output_leaves_small_output_unchanged · function · L185-L189 — fn cap_tool_output_leaves_small_output_unchanged()
- cap_tool_output_adds_visible_truncation_notice · function · L192-L199 — fn cap_tool_output_adds_visible_truncation_notice()
- cap_sdk_tool_content_adds_same_notice · function · L202-L209 — fn cap_sdk_tool_content_adds_same_notice()
- image_anchor_tests · module · L213-L237 — mod image_anchor_tests
- live_batch_images_anchor_to_parent_and_have_no_history_boundary · function · L217-L236 — fn live_batch_images_anchor_to_parent_and_have_no_history_boundary()
