# crates/jcode-provider-openai-runtime/src/openai_rate_limit_format.rs

- format_rate_limit_error · function · L4-L70 — pub(crate) fn format_rate_limit_error(body: &str, retry_after: Option<Duration>) -> String
- format_unstructured_rate_limit_error · function · L72-L77 — fn format_unstructured_rate_limit_error(body: &str, retry_after: Option<Duration>) -> String
- ensure_sentence_ending · function · L79-L83 — fn ensure_sentence_ending(text: &mut String)
- json_nonnegative_seconds · function · L85-L89 — fn json_nonnegative_seconds(value: &Value) -> Option<u64>
- json_timestamp · function · L91-L95 — fn json_timestamp(value: &Value) -> Option<i64>
- format_compact_duration · function · L97-L128 — fn format_compact_duration(duration: Duration) -> String
- tests · module · L131-L194 — mod tests
- structured_usage_limit_is_human_readable · function · L135-L148 — fn structured_usage_limit_is_human_readable()
- structured_quota_preserves_kind_when_message_omits_error_code · function · L151-L158 — fn structured_quota_preserves_kind_when_message_omits_error_code()
- structured_usage_limit_handles_missing_optional_fields · function · L161-L168 — fn structured_usage_limit_handles_missing_optional_fields()
- structured_usage_limit_accepts_string_reset_values · function · L171-L178 — fn structured_usage_limit_accepts_string_reset_values()
- unknown_or_malformed_rate_limit_payload_keeps_original_body · function · L181-L193 — fn unknown_or_malformed_rate_limit_payload_keeps_original_body()
