# crates/jcode-app-core/src/tool/browser_user_browser_tests.rs

- FAKE_CLI · constant · L10-L60 — const FAKE_CLI: &str = r#"#!/usr/bin/env python3
- Fixture · struct · L62-L67 — struct Fixture
- new · function · L70-L87 — fn new() -> Self
- lines · function · L89-L95 — fn lines(&self, file: &str) -> Vec<Value>
- context · function · L97-L107 — fn context(&self, name: &str) -> ToolContext
- drop · function · L111-L134 — fn drop(&mut self)
- closed_agent_window_restarts_the_session_and_retries_once · function · L138-L155 — async fn closed_agent_window_restarts_the_session_and_retries_once()
- explicit_tab_targets_are_not_retried_on_a_new_session · function · L158-L172 — async fn explicit_tab_targets_are_not_retried_on_a_new_session()
- list_tabs_shows_every_window_without_opening_an_agent_window · function · L175-L208 — async fn list_tabs_shows_every_window_without_opening_an_agent_window()
- upload_works_without_a_session_daemon · function · L211-L237 — async fn upload_works_without_a_session_daemon()
- bridge_errors_are_explained · function · L240-L254 — fn bridge_errors_are_explained()
