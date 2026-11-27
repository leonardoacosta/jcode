# crates/jcode-app-core/src/tool/communicate/transport.rs

- SERVER_NOT_RUNNING · constant · L6-L6 — const SERVER_NOT_RUNNING: &str = "jcode server is not running; start it with jcode server start";
- server_is_unavailable · function · L8-L19 — fn server_is_unavailable(err: &anyhow::Error) -> bool
- map_socket_connection_error · function · L21-L27 — fn map_socket_connection_error(err: anyhow::Error) -> anyhow::Error
- connect_swarm_socket · function · L29-L58 — async fn connect_swarm_socket(path: &std::path::Path) -> Result<crate::transport::Stream>
- request_type_from_json · function · L60-L70 — fn request_type_from_json(json: &str) -> String
- READ_DEADLINE · constant · L72-L72 — const READ_DEADLINE: std::time::Duration = std::time::Duration::from_secs(30);
- MUTATING_DEADLINE · constant · L73-L73 — const MUTATING_DEADLINE: std::time::Duration = std::time::Duration::from_secs(120);
- is_mutating_request_type · function · L77-L82 — pub(super) fn is_mutating_request_type(request_type: &str) -> bool
- default_deadline · function · L84-L90 — pub(super) fn default_deadline(request_type: &str) -> std::time::Duration
- SwarmRequestError · enum · L94-L97 — pub(super) enum SwarmRequestError
- fmt · function · L100-L113 — fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
- send_request · function · L118-L120 — pub(in crate::tool) async fn send_request(request: Request) -> Result<ServerEvent>
- send_request_with_timeout · function · L122-L221 — pub(super) async fn send_request_with_timeout(
- tests · module · L224-L256 — mod tests
- missing_daemon_socket_has_actionable_error · function · L228-L238 — async fn missing_daemon_socket_has_actionable_error()
- refused_daemon_socket_has_actionable_error · function · L242-L255 — async fn refused_daemon_socket_has_actionable_error()
