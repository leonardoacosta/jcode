# crates/jcode-provider-openai-runtime/src/openai_tests/persistent_prefix.rs

- PrefixTestSocket · type · L3-L3 — type PrefixTestSocket = tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>;
- prefix_test_request · function · L5-L18 — async fn prefix_test_request(socket: &mut PrefixTestSocket) -> Value
- prefix_test_response · function · L20-L31 — async fn prefix_test_response(socket: &mut PrefixTestSocket, id: &str)
- prefix_test_complete · function · L33-L56 — async fn prefix_test_complete(provider: &OpenAIProvider, messages: &[ChatMessage], id: &str)
- prefix_test_provider · function · L58-L64 — async fn prefix_test_provider() -> OpenAIProvider
- persistent_prefix_changed_output_case · function · L66-L171 — async fn persistent_prefix_changed_output_case(had_real_output: bool)
- persistent_prefix_late_tool_output_before_cursor_replays_full_input · function · L174-L176 — async fn persistent_prefix_late_tool_output_before_cursor_replays_full_input()
- persistent_prefix_mutated_tool_output_with_growing_input_replays_full_input · function · L179-L181 — async fn persistent_prefix_mutated_tool_output_with_growing_input_replays_full_input()
- persistent_prefix_append_only_reuses_socket_and_refreshes_hashes_each_turn · function · L184-L266 — async fn persistent_prefix_append_only_reuses_socket_and_refreshes_hashes_each_turn()
