# crates/jcode-harness-api-server/src/background_progress.rs

- BackgroundProgress · struct · L17-L24 — pub struct BackgroundProgress
- into_event · function · L27-L36 — pub fn into_event(self) -> ApiEvent
- parse_background_notification · function · L42-L101 — pub fn parse_background_notification(message: &str) -> Option<BackgroundProgress>
- split_backticked · function · L104-L114 — fn split_backticked(rest: &str) -> Option<(String, &str)>
- header_label · function · L118-L126 — fn header_label(rest: &str) -> String
- inline_detail · function · L129-L133 — fn inline_detail(rest: &str) -> Option<&str>
- clean_summary · function · L137-L152 — fn clean_summary(detail: &str) -> String
- parse_percent · function · L155-L167 — fn parse_percent(summary: &str) -> Option<f32>
- tests · module · L171-L171 — mod tests;
