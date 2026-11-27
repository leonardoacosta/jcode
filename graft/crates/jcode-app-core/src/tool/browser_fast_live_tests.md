# crates/jcode-app-core/src/tool/browser_fast_live_tests.rs

- fixture_context · function · L13-L27 — fn fixture_context() -> ToolContext
- local_fixture_tab · function · L29-L31 — async fn local_fixture_tab(variable: &str, ctx: &ToolContext) -> i64
- local_fixture_tab_at_path · function · L33-L80 — async fn local_fixture_tab_at_path(
- handoff · function · L82-L84 — async fn handoff(tab_id: i64, goal: &str, ctx: ToolContext) -> Value
- handoff_with_candidates · function · L86-L116 — async fn handoff_with_candidates(
- live_browser_handoff_completes_local_navigation · function · L120-L156 — async fn live_browser_handoff_completes_local_navigation()
- live_browser_handoff_completes_search_and_nested_navigation · function · L160-L203 — async fn live_browser_handoff_completes_search_and_nested_navigation()
- live_browser_handoff_sensitive_fixture_hands_back_without_actions · function · L207-L224 — async fn live_browser_handoff_sensitive_fixture_hands_back_without_actions()
- live_browser_handoff_requests_script_and_resumes · function · L228-L297 — async fn live_browser_handoff_requests_script_and_resumes()
- TITLE · constant · L229-L229 — const TITLE: &str = "Jev hybrid verified";
- GOAL · constant · L230-L230 — const GOAL: &str = "Set this page's document.title to exactly Jev hybrid verified, then finish only when the fresh page title matches. Stay on this page.";
