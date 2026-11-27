# crates/jcode-app-core/src/agent_tests/concurrency_construction.rs

- IsolatedTelemetryEnv · struct · L3-L6 — struct IsolatedTelemetryEnv
- new · function · L9-L21 — fn new() -> Self
- drop · function · L25-L32 — fn drop(&mut self)
- provisional_connection_does_not_track_until_logical_ownership_commits · function · L36-L60 — async fn provisional_connection_does_not_track_until_logical_ownership_commits()
- headless_parent_is_set_before_concurrency_tracking_begins · function · L63-L86 — async fn headless_parent_is_set_before_concurrency_tracking_begins()
- provisional_agent_defers_session_context_until_committed · function · L92-L107 — async fn provisional_agent_defers_session_context_until_committed()
- provisional_agent_defers_presence_markers_until_committed · function · L112-L134 — async fn provisional_agent_defers_presence_markers_until_committed()
- provisional_agent_defers_telemetry_session_until_committed · function · L139-L155 — async fn provisional_agent_defers_telemetry_session_until_committed()
