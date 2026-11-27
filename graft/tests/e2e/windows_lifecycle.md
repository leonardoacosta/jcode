# tests/e2e/windows_lifecycle.rs

- SpawnedWindowsServer · struct · L3-L13 — struct SpawnedWindowsServer
- jcode_binary · function · L16-L27 — fn jcode_binary() -> std::path::PathBuf
- spawn · function · L29-L83 — fn spawn(prefix: &str) -> Result<Self>
- wait_ready · function · L85-L87 — async fn wait_ready(&self) -> Result<()>
- apply_env · function · L89-L103 — fn apply_env<'a>(&self, command: &'a mut Command) -> &'a mut Command
- jcode_command · function · L105-L109 — fn jcode_command(&self) -> Command
- spawn_same_socket_child · function · L111-L134 — fn spawn_same_socket_child(
- dump_extra_logs · function · L136-L155 — fn dump_extra_logs(
- dump_logs · function · L157-L193 — fn dump_logs(&self, label: &str)
- drop · function · L197-L199 — fn drop(&mut self)
- wait_for_server_unreachable · function · L202-L234 — async fn wait_for_server_unreachable(socket_path: &std::path::Path) -> Result<()>
- windows_binary_server_accepts_clients_and_debug_cli · function · L237-L290 — async fn windows_binary_server_accepts_clients_and_debug_cli() -> Result<()>
- windows_binary_server_rebinds_named_pipe_after_exit · function · L293-L329 — async fn windows_binary_server_rebinds_named_pipe_after_exit() -> Result<()>
