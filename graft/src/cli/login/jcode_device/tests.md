# src/cli/login/jcode_device/tests.rs

- ScriptedHttpResponse · type · L5-L5 — type ScriptedHttpResponse = (u16, Vec<(&'static str, &'static str)>, String);
- accept_token_request · function · L7-L44 — fn accept_token_request(listener: &std::net::TcpListener) -> std::net::TcpStream
- spawn_scripted_http_server · function · L46-L64 — fn spawn_scripted_http_server(responses: Vec<ScriptedHttpResponse>) -> String
- token_fixture_rejects_oversized_headers_and_bodies · function · L67-L89 — fn token_fixture_rejects_oversized_headers_and_bodies()
- scripted_responses_are_not_consumed_by_unrelated_http_probes · function · L92-L111 — async fn scripted_responses_are_not_consumed_by_unrelated_http_probes()
- test_client · function · L113-L118 — fn test_client() -> reqwest::Client
- polling_pending_slow_down_then_approval · function · L121-L151 — async fn polling_pending_slow_down_then_approval()
- polling_denied_has_clear_redacted_error · function · L154-L173 — async fn polling_denied_has_clear_redacted_error()
- polling_timeout_is_deterministic_before_first_request · function · L176-L188 — async fn polling_timeout_is_deterministic_before_first_request()
- cancellation_during_consumed_exchange_finishes_and_returns_the_key · function · L191-L215 — async fn cancellation_during_consumed_exchange_finishes_and_returns_the_key()
- approved_key_persistence_is_owner_only_and_clear_is_deterministic · function · L218-L262 — fn approved_key_persistence_is_owner_only_and_clear_is_deterministic()
