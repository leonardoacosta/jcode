# crates/jcode-app-core/src/session_effort.rs

- SESSION_EFFORTS · constant · L13-L14 — static SESSION_EFFORTS: LazyLock<RwLock<HashMap<String, String>>> =
- record_session_effort · function · L18-L30 — pub fn record_session_effort(session_id: &str, effort: Option<&str>)
- session_effort · function · L33-L35 — pub fn session_effort(session_id: &str) -> Option<String>
- forget_session_effort · function · L38-L42 — pub fn forget_session_effort(session_id: &str)
- tests · module · L45-L65 — mod tests
- records_and_reads_back_effort · function · L49-L64 — fn records_and_reads_back_effort()
