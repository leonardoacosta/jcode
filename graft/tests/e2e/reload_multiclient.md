# tests/e2e/reload_multiclient.rs

- start_inprocess_server · function · L15-L39 — async fn start_inprocess_server(
- read_until_reloading · function · L44-L63 — async fn read_until_reloading(
- subscribe_new_session · function · L65-L75 — async fn subscribe_new_session(socket_path: &std::path::Path) -> Result<(server::Client, String)>
- reload_notifies_clients_across_independent_sessions · function · L82-L128 — async fn reload_notifies_clients_across_independent_sessions() -> Result<()>
- reload_notifies_successor_after_session_takeover · function · L134-L196 — async fn reload_notifies_successor_after_session_takeover() -> Result<()>
- reload_interrupts_in_flight_streaming_turn_without_hanging · function · L203-L283 — async fn reload_interrupts_in_flight_streaming_turn_without_hanging() -> Result<()>
- reload_marker_is_active_when_clients_are_notified · function · L289-L332 — async fn reload_marker_is_active_when_clients_are_notified() -> Result<()>
