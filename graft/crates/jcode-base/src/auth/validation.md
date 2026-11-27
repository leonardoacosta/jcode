# crates/jcode-base/src/auth/validation.rs

- VALIDATION_STATUS_FILE · constant · L5-L5 — const VALIDATION_STATUS_FILE: &str = "auth-validation.json";
- status_path · function · L9-L11 — pub fn status_path() -> Result<PathBuf>
- load_all · function · L13-L18 — pub fn load_all() -> BTreeMap<String, ProviderValidationRecord>
- get · function · L20-L22 — pub fn get(provider_id: &str) -> Option<ProviderValidationRecord>
- save · function · L24-L28 — pub fn save(provider_id: &str, record: ProviderValidationRecord) -> Result<()>
- status_label · function · L30-L32 — pub fn status_label(provider_id: &str) -> Option<String>
- format_record_label · function · L34-L59 — pub fn format_record_label(record: &ProviderValidationRecord) -> String
- age_label · function · L61-L71 — fn age_label(checked_at_ms: i64) -> String
- tests · module · L74-L137 — mod tests
- format_record_label_prefers_tool_validated_wording · function · L78-L87 — fn format_record_label_prefers_tool_validated_wording()
- format_record_label_reports_failures · function · L90-L99 — fn format_record_label_reports_failures()
- format_record_label_marks_stale_records · function · L102-L136 — fn format_record_label_marks_stale_records()
