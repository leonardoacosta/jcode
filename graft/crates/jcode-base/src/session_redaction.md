# crates/jcode-base/src/session_redaction.rs

- RedactedField · enum · L6-L10 — enum RedactedField
- redact_shared_content · function · L12-L47 — pub(super) fn redact_shared_content(content: &mut SharedContent)
- redacted_text · function · L49-L52 — fn redacted_text(text: &str) -> Option<String>
- redacted_field · function · L54-L80 — fn redacted_field(block: &ContentBlock) -> Option<RedactedField>
- redacted_json_value · function · L82-L114 — fn redacted_json_value(value: &serde_json::Value) -> Option<serde_json::Value>
- redact_json_value · function · L118-L139 — pub(super) fn redact_json_value(value: &mut serde_json::Value)
- is_sensitive_json_key · function · L141-L155 — pub(super) fn is_sensitive_json_key(key: &str) -> bool
