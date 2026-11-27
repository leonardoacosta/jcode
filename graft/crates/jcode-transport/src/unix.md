# crates/jcode-transport/src/unix.rs

- is_socket_path · function · L8-L10 — pub fn is_socket_path(path: &std::path::Path) -> bool
- remove_socket · function · L12-L14 — pub fn remove_socket(path: &std::path::Path)
- stream_pair · function · L17-L19 — pub fn stream_pair() -> std::io::Result<(Stream, Stream)>
- tests · module · L22-L92 — mod tests
- a_listener_accepts_and_round_trips_bytes · function · L31-L62 — async fn a_listener_accepts_and_round_trips_bytes()
- a_stream_pair_is_connected_in_both_directions · function · L65-L75 — async fn a_stream_pair_is_connected_in_both_directions()
- an_absent_path_is_not_a_socket · function · L78-L82 — fn an_absent_path_is_not_a_socket()
- removing_an_absent_socket_is_a_no_op · function · L87-L91 — fn removing_an_absent_socket_is_a_no_op()
