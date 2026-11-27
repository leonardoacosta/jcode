# crates/jcode-provider-openai-runtime/src/openai_tests/websocket_prewarm.rs

- prewarm_test_credentials · function · L1-L9 — fn prewarm_test_credentials() -> CodexCredentials
- prewarm_test_tool · function · L11-L22 — fn prewarm_test_tool() -> ToolDefinition
- prewarm_user_message · function · L24-L35 — fn prewarm_user_message(text: &str) -> ChatMessage
- wait_for_prewarm · function · L37-L45 — async fn wait_for_prewarm(slot: &openai_websocket_prewarm::PrewarmSlot)
- websocket_v2_prewarm_is_adopted_by_complete_without_losing_request_state · function · L50-L220 — async fn websocket_v2_prewarm_is_adopted_by_complete_without_losing_request_state()
- unfinished_or_incompatible_prewarm_is_cancelled_without_foreground_wait · function · L223-L261 — async fn unfinished_or_incompatible_prewarm_is_cancelled_without_foreground_wait()
- ready_prewarm_with_different_settings_is_invalidated · function · L264-L296 — async fn ready_prewarm_with_different_settings_is_invalidated()
- rejected_warmup_is_not_adopted · function · L299-L326 — async fn rejected_warmup_is_not_adopted()
