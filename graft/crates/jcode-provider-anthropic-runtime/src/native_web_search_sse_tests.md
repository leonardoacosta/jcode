# crates/jcode-provider-anthropic-runtime/src/native_web_search_sse_tests.rs

- sse · function · L6-L11 — fn sse(event_type: &str, data: serde_json::Value) -> SseEvent
- provider_native_items · function · L13-L24 — fn provider_native_items(events: &[StreamEvent]) -> Vec<serde_json::Value>
- server_tool_use_streams_input_and_emits_one_native_item · function · L27-L68 — fn server_tool_use_streams_input_and_emits_one_native_item()
- web_search_tool_result_is_kept_verbatim · function · L71-L107 — fn web_search_tool_result_is_kept_verbatim()
- client_tool_use_after_server_tool_still_works · function · L110-L140 — fn client_tool_use_after_server_tool_still_works()
- stored_search_turn · function · L142-L168 — fn stored_search_turn() -> Vec<Message>
- stored_server_blocks_replay_verbatim_when_tool_attached · function · L171-L187 — fn stored_server_blocks_replay_verbatim_when_tool_attached()
- stored_server_blocks_become_text_without_the_tool · function · L190-L204 — fn stored_server_blocks_become_text_without_the_tool()
- paused_server_tool_turn_is_not_given_a_continuation_user_turn · function · L207-L223 — fn paused_server_tool_turn_is_not_given_a_continuation_user_turn()
- interrupted_turn_after_search_text_still_gets_continuation_user_turn · function · L226-L237 — fn interrupted_turn_after_search_text_still_gets_continuation_user_turn()
- request_tools_place_server_tool_between_eager_and_deferred · function · L240-L258 — fn request_tools_place_server_tool_between_eager_and_deferred()
