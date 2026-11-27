# crates/jcode-app-core/src/server/window_closure_tests.rs

- Runtime · struct · L4-L7 — struct Runtime
- new · function · L9-L17 — fn new() -> Self
- drop · function · L20-L25 — fn drop(&mut self)
- connection · function · L27-L40 — fn connection(id: &str) -> ClientConnectionInfo
- closures · function · L41-L43 — fn closures() -> HashMap<String, chrono::DateTime<chrono::Utc>>
- only_final_transport_close_records_time_and_cleanup_does_not_refresh_it · function · L46-L70 — async fn only_final_transport_close_records_time_and_cleanup_does_not_refresh_it()
- explicit_detach_then_socket_close_is_not_a_window_close · function · L73-L91 — async fn explicit_detach_then_socket_close_is_not_a_window_close()
- server_reload_does_not_manufacture_window_close · function · L94-L109 — async fn server_reload_does_not_manufacture_window_close()
