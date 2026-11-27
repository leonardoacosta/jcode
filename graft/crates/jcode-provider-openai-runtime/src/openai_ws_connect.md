# crates/jcode-provider-openai-runtime/src/openai_ws_connect.rs

- WsTransport · enum · L33-L36 — pub(crate) enum WsTransport
- OpenAIWs · type · L39-L39 — pub(crate) type OpenAIWs = WebSocketStream<DeflateStream<WsTransport>>;
- poll_read · function · L42-L51 — fn poll_read(
- poll_write · function · L55-L64 — fn poll_write(
- poll_flush · function · L66-L71 — fn poll_flush(self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<std::io::Result<()>>
- poll_shutdown · function · L73-L78 — fn poll_shutdown(self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<std::io::Result<()>>
- tls_config · function · L82-L107 — fn tls_config() -> Result<Arc<rustls::ClientConfig>>
- CONFIG · constant · L83-L84 — static CONFIG: OnceLock<std::result::Result<Arc<rustls::ClientConfig>, String>> =
- connect_ws_with_mode · function · L114-L160 — pub(crate) async fn connect_ws_with_mode<R>(
- connect_ws · function · L164-L169 — pub(crate) async fn connect_ws<R>(request: R) -> Result<(OpenAIWs, Response), WsError>
