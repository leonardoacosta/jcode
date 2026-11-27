# crates/jcode-app-core/src/message_notifications.rs

- InputShellResult · struct · L4-L16 — pub struct InputShellResult
- sanitize_fenced_block · function · L18-L20 — fn sanitize_fenced_block(text: &str) -> String
- format_input_shell_result_markdown · function · L22-L55 — pub fn format_input_shell_result_markdown(shell: &InputShellResult) -> String
- indent_block · function · L57-L62 — fn indent_block(text: &str) -> String
- input_shell_status_notice · function · L64-L74 — pub fn input_shell_status_notice(shell: &InputShellResult) -> String
- format_background_task_status · function · L76-L84 — fn format_background_task_status(status: &BackgroundTaskStatus) -> &'static str
- normalize_background_task_preview · function · L86-L94 — fn normalize_background_task_preview(preview: &str) -> Option<String>
- format_background_task_notification_markdown · function · L96-L123 — pub fn format_background_task_notification_markdown(task: &BackgroundTaskCompleted) -> String
- ParsedBackgroundTaskNotification · struct · L126-L134 — pub struct ParsedBackgroundTaskNotification
- parse_background_task_notification_markdown · function · L136-L194 — pub fn parse_background_task_notification_markdown(
- HEADER_RE · constant · L139-L139 — static HEADER_RE: OnceLock<Option<Regex>> = OnceLock::new();
- FULL_OUTPUT_RE · constant · L140-L140 — static FULL_OUTPUT_RE: OnceLock<Option<Regex>> = OnceLock::new();
- background_task_status_notice · function · L196-L217 — pub fn background_task_status_notice(task: &BackgroundTaskCompleted) -> String
