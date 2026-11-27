# crates/jcode-background-types/src/lib.rs

- BackgroundTaskStatus · enum · L7-L15 — pub enum BackgroundTaskStatus
- BackgroundTaskProgressKind · enum · L19-L22 — pub enum BackgroundTaskProgressKind
- BackgroundTaskProgressSource · enum · L26-L30 — pub enum BackgroundTaskProgressSource
- BackgroundTaskProgress · struct · L33-L43 — pub struct BackgroundTaskProgress
- normalize · function · L46-L69 — pub fn normalize(mut self) -> Self
- tests · module · L73-L93 — mod tests
- measurable_counts_override_a_conflicting_reported_percent · function · L77-L92 — fn measurable_counts_override_a_conflicting_reported_percent()
- BackgroundTaskProgressEvent · struct · L96-L102 — pub struct BackgroundTaskProgressEvent
- BackgroundTaskCompleted · struct · L106-L118 — pub struct BackgroundTaskCompleted
- BackgroundTaskStalled · struct · L127-L141 — pub struct BackgroundTaskStalled
- format_progress_display · function · L145-L152 — pub fn format_progress_display(progress: &BackgroundTaskProgress, width: usize) -> String
- format_progress_summary · function · L155-L183 — pub fn format_progress_summary(progress: &BackgroundTaskProgress) -> String
- progress_source_label · function · L185-L191 — fn progress_source_label(source: &BackgroundTaskProgressSource) -> &'static str
- render_progress_bar · function · L194-L204 — pub fn render_progress_bar(progress: &BackgroundTaskProgress, width: usize) -> Option<String>
