# crates/jcode-tui/src/tui/app/commit_log.rs

- COMMIT_LOG_LIMIT · constant · L7-L7 — const COMMIT_LOG_LIMIT: usize = 500;
- CommitLogEntry · struct · L11-L17 — pub struct CommitLogEntry
- CommitLogOverlay · struct · L21-L28 — pub struct CommitLogOverlay
- parse_commit_log · function · L31-L45 — pub(crate) fn parse_commit_log(text: &str) -> Vec<CommitLogEntry>
- load_commit_log · function · L48-L114 — pub(crate) fn load_commit_log(dir: Option<&std::path::Path>) -> CommitLogOverlay
- toggle_commit_log_overlay · function · L118-L130 — pub(super) fn toggle_commit_log_overlay(&mut self)
- handle_commit_log_key · function · L132-L171 — pub(super) fn handle_commit_log_key(&mut self, code: KeyCode) -> Result<()>
- scroll_commit_log_overlay · function · L173-L184 — pub(super) fn scroll_commit_log_overlay(&mut self, direction: i16) -> bool
- tests · module · L188-L213 — mod tests
- parses_commit_log_records · function · L192-L202 — fn parses_commit_log_records()
- non_repo_reports_error · function · L205-L212 — fn non_repo_reports_error()
