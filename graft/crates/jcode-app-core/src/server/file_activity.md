# crates/jcode-app-core/src/server/file_activity.rs

- parse_file_activity_line_range · function · L3-L53 — pub(crate) fn parse_file_activity_line_range(summary: Option<&str>) -> Option<(u64, u64)>
- line_range_tests · module · L56-L79 — mod line_range_tests
- rejects_missing_or_overflowing_range_endpoints · function · L60-L78 — fn rejects_missing_or_overflowing_range_endpoints()
- file_activity_scope_label · function · L81-L97 — pub(crate) fn file_activity_scope_label(
- FILE_ACTIVITY_NOTICE_MAX_ENTRIES · constant · L100-L100 — pub(crate) const FILE_ACTIVITY_NOTICE_MAX_ENTRIES: usize = 12;
- FILE_ACTIVITY_INTENT_MAX_CHARS · constant · L101-L101 — const FILE_ACTIVITY_INTENT_MAX_CHARS: usize = 60;
- FileActivityEntry · struct · L105-L120 — pub(crate) struct FileActivityEntry
- merge_file_activity_entry · function · L124-L148 — pub(crate) fn merge_file_activity_entry(
- format_file_activity_entry · function · L150-L191 — fn format_file_activity_entry(entry: &FileActivityEntry) -> String
- format_file_activity_notice · function · L199-L202 — pub(crate) fn format_file_activity_notice(entries: &[FileActivityEntry]) -> String
- file_activity_display_path · function · L205-L215 — pub(crate) fn file_activity_display_path(
- PendingFileActivityNotice · struct · L218-L221 — pub(crate) struct PendingFileActivityNotice
- deliver_file_activity_notice · function · L226-L289 — pub(crate) async fn deliver_file_activity_notice(
