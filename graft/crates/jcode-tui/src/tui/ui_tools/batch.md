# crates/jcode-tui/src/tui/ui_tools/batch.rs

- BatchSubResult · struct · L11-L14 — pub(crate) struct BatchSubResult
- BatchCompletionCounts · struct · L17-L20 — pub(crate) struct BatchCompletionCounts
- total · function · L23-L25 — pub(crate) fn total(self) -> usize
- batch_section_index · function · L28-L32 — pub(crate) fn batch_section_index(line: &str) -> Option<usize>
- batch_header_fragment · function · L34-L42 — fn batch_header_fragment(line: &str) -> Option<&str>
- is_batch_footer_line · function · L44-L59 — pub(crate) fn is_batch_footer_line(line: &str) -> bool
- parse_batch_completion_counts · function · L61-L82 — pub(crate) fn parse_batch_completion_counts(content: &str) -> Option<BatchCompletionCounts>
- finalize_batch_section · function · L84-L97 — pub(crate) fn finalize_batch_section(raw: &str) -> BatchSubResult
- parse_batch_sub_outputs · function · L100-L104 — pub(crate) fn parse_batch_sub_outputs(content: &str) -> Vec<BatchSubResult>
- parse_batch_sub_outputs_by_index · function · L106-L143 — pub(crate) fn parse_batch_sub_outputs_by_index(
- batch_subcall_params · function · L148-L164 — pub(crate) fn batch_subcall_params(call: &serde_json::Value) -> serde_json::Value
- batch_subcall_intent · function · L168-L178 — pub(crate) fn batch_subcall_intent(
- tests · module · L181-L208 — mod tests
- parses_first_batch_section_with_tool_timing_prefix · function · L185-L199 — fn parses_first_batch_section_with_tool_timing_prefix()
- rejects_non_header_lines_containing_batch_marker · function · L202-L207 — fn rejects_non_header_lines_containing_batch_marker()
