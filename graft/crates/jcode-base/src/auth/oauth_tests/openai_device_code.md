# crates/jcode-base/src/auth/oauth_tests/openai_device_code.rs

- read_request · function · L7-L29 — async fn read_request(stream: &mut tokio::net::TcpStream) -> (String, String)
- http_response · function · L31-L36 — fn http_response(status: u16, body: &str) -> String
- device_code_flow_polls_until_approved_then_exchanges · function · L39-L95 — async fn device_code_flow_polls_until_approved_then_exchanges()
- device_code_disabled_account_gets_actionable_error · function · L98-L114 — async fn device_code_disabled_account_gets_actionable_error()
