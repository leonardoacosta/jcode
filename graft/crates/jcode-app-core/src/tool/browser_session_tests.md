# crates/jcode-app-core/src/tool/browser_session_tests.rs

- BrowserFixture · struct · L6-L11 — struct BrowserFixture
- new · function · L14-L17 — fn new() -> Self
- with_temp · function · L19-L78 — fn with_temp(guard: std::sync::MutexGuard<'static, ()>, temp: tempfile::TempDir) -> Self
- starts · function · L80-L90 — fn starts(&self) -> Vec<(String, i32)>
- context · function · L92-L102 — fn context(&self, name: &str) -> ToolContext
- drop · function · L106-L128 — fn drop(&mut self)
- browser_session_bash_uses_canonical_fixture_home · function · L132-L156 — async fn browser_session_bash_uses_canonical_fixture_home()
- browser_session_reuses_daemon_across_five_tool_calls · function · L159-L177 — async fn browser_session_reuses_daemon_across_five_tool_calls()
- browser_session_concurrent_calls_start_one_daemon · function · L180-L204 — async fn browser_session_concurrent_calls_start_one_daemon()
- browser_session_stalled_startup_keeps_healthy_calls_and_workers_responsive · function · L207-L274 — async fn browser_session_stalled_startup_keeps_healthy_calls_and_workers_responsive()
