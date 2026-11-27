# crates/jcode-base/src/auth/oauth_callback_io.rs

- bad_request_response · function · L3-L13 — pub(super) fn bad_request_response(message: &str) -> String
- is_socket_read_timeout · function · L15-L20 — pub(super) fn is_socket_read_timeout(err: &std::io::Error) -> bool
- read_http_request_line_blocking · function · L22-L32 — pub(super) fn read_http_request_line_blocking<R: BufRead>(
- drain_http_headers_blocking · function · L34-L46 — pub(super) fn drain_http_headers_blocking<R: BufRead>(reader: &mut R) -> Result<bool>
- read_http_request_line_async · function · L48-L66 — pub(super) async fn read_http_request_line_async<R>(
- drain_http_headers_async · function · L68-L90 — pub(super) async fn drain_http_headers_async<R>(
- wait_for_callback · function · L93-L183 — pub fn wait_for_callback(port: u16, expected_state: &str) -> Result<String>
- wait_for_callback_async · function · L186-L189 — pub async fn wait_for_callback_async(port: u16, expected_state: &str) -> Result<String>
- bind_callback_listener · function · L191-L195 — pub fn bind_callback_listener(port: u16) -> Result<tokio::net::TcpListener>
- wait_for_callback_async_on_listener · function · L197-L306 — pub async fn wait_for_callback_async_on_listener(
