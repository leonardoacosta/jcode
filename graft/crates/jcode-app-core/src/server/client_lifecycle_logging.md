# crates/jcode-app-core/src/server/client_lifecycle_logging.rs

- interrupt_request_log_fields · function · L3-L40 — pub(super) fn interrupt_request_log_fields(
- request_type_from_line · function · L42-L52 — pub(super) fn request_type_from_line(line: &str) -> String
- request_type_is_read_only · function · L54-L75 — pub(super) fn request_type_is_read_only(kind: &str) -> bool
- request_payload_summary · function · L77-L117 — pub(super) fn request_payload_summary(kind: &str, line: &str) -> Vec<(String, String)>
- ServerRequestLifecycleFields · struct · L119-L130 — pub(super) struct ServerRequestLifecycleFields<'a>
- server_request_lifecycle_fields · function · L132-L180 — pub(super) fn server_request_lifecycle_fields(
