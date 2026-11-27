# crates/jcode-provider-core/src/transport.rs

- send_with_initial_response_timeout · function · L18-L29 — pub async fn send_with_initial_response_timeout(
- is_transient_transport_error · function · L34-L89 — pub fn is_transient_transport_error(error_str: &str) -> bool
- tests · module · L92-L205 — mod tests
- accepted_request_without_response_headers_times_out · function · L101-L148 — async fn accepted_request_without_response_headers_times_out()
- http2_stream_protocol_error_is_transient · function · L151-L157 — fn http2_stream_protocol_error_is_transient()
- http2_goaway_and_refused_stream_are_transient · function · L160-L163 — fn http2_goaway_and_refused_stream_are_transient()
- auth_errors_are_not_transient · function · L166-L169 — fn auth_errors_are_not_transient()
- real_world_transport_errors_are_transient · function · L173-L191 — fn real_world_transport_errors_are_transient()
- openai_compatible_stream_read_error_is_transient · function · L196-L204 — fn openai_compatible_stream_read_error_is_transient()
