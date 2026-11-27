# crates/jcode-app-core/src/tool/feedback.rs

- MAX_SUMMARY_CHARS · constant · L7-L7 — const MAX_SUMMARY_CHARS: usize = 240;
- MAX_DETAILS_CHARS · constant · L8-L8 — const MAX_DETAILS_CHARS: usize = 1500;
- MaintainerFeedbackTool · struct · L10-L10 — pub struct MaintainerFeedbackTool;
- new · function · L13-L15 — pub fn new() -> Self
- FeedbackInput · struct · L20-L27 — struct FeedbackInput
- FeedbackCategory · enum · L31-L37 — enum FeedbackCategory
- FeedbackOrigin · enum · L41-L45 — enum FeedbackOrigin
- limited · function · L47-L56 — fn limited(value: &str, label: &str, max: usize) -> Result<String>
- payload · function · L58-L77 — fn payload(input: FeedbackInput) -> Result<String>
- name · function · L81-L83 — fn name(&self) -> &str
- description · function · L85-L87 — fn description(&self) -> &str
- parameters_schema · function · L89-L123 — fn parameters_schema(&self) -> Value
- execute · function · L125-L136 — async fn execute(&self, input: Value, _ctx: ToolContext) -> Result<ToolOutput>
- tests · module · L140-L230 — mod tests
- payload_labels_and_formats_feedback · function · L144-L157 — fn payload_labels_and_formats_feedback()
- payload_rejects_empty_and_oversized_fields · function · L160-L178 — fn payload_rejects_empty_and_oversized_fields()
- schema_requires_provenance_and_carries_privacy_guidance · function · L181-L205 — fn schema_requires_provenance_and_carries_privacy_guidance()
- user_origin_requires_explicit_confirmation · function · L208-L229 — fn user_origin_requires_explicit_confirmation()
