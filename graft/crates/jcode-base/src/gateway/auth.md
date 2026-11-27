# crates/jcode-base/src/gateway/auth.rs

- WsAuth · struct · L2-L5 — pub(super) struct WsAuth
- WsAuthSource · enum · L8-L11 — pub(super) enum WsAuthSource
- extract_ws_auth · function · L17-L67 — pub(super) fn extract_ws_auth(
- parse_bearer_token · function · L69-L83 — pub(crate) fn parse_bearer_token(header_value: &str) -> Option<&str>
- parse_query_token · function · L85-L94 — pub(crate) fn parse_query_token(query: &str) -> Option<&str>
- is_valid_hex_token · function · L96-L98 — pub(crate) fn is_valid_hex_token(token: &str) -> bool
- AuthorizedDevice · struct · L102-L105 — pub(super) struct AuthorizedDevice
- authorize_ws_device · function · L117-L135 — pub(super) fn authorize_ws_device(
- ws_error_response · function · L137-L163 — pub(super) fn ws_error_response(
