# crates/jcode-voice/src/global/session.rs

- CHECK_TIMEOUT · constant · L15-L15 — const CHECK_TIMEOUT: Duration = Duration::from_millis(200);
- permitted · function · L19-L22 — pub async fn permitted() -> bool
- permitted · function · L25-L27 — pub async fn permitted() -> bool
- within · function · L30-L42 — async fn within(check: impl Future<Output = bool>, deadline: impl Future) -> bool
- check_session · function · L45-L89 — async fn check_session() -> zbus::Result<bool>
- allowed · function · L94-L96 — fn allowed(active: bool, locked: bool, remote: bool, kind: &str) -> bool
- tests · module · L99-L127 — mod tests
- only_active_unlocked_local_graphical_session_is_allowed · function · L103-L116 — fn only_active_unlocked_local_graphical_session_is_allowed()
- timeout_and_unknown_results_fail_closed · function · L119-L126 — fn timeout_and_unknown_results_fail_closed()
