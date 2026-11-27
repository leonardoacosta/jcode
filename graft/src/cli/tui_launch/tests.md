# src/cli/tui_launch/tests.rs

- ENV_LOCK · constant · L24-L24 — static ENV_LOCK: Mutex<()> = Mutex::new(());
- EnvVarGuard · struct · L27-L30 — struct EnvVarGuard
- set_path · function · L34-L38 — fn set_path(key: &'static str, value: &Path) -> Self
- set_value · function · L40-L44 — fn set_value(key: &'static str, value: &str) -> Self
- drop · function · L49-L55 — fn drop(&mut self)
- write_fake_handterm · function · L59-L68 — fn write_fake_handterm(temp: &tempfile::TempDir, output_path: &Path)
- wait_for_lines · function · L71-L86 — fn wait_for_lines(path: &Path, min_lines: usize) -> Vec<String>
- spawn_resume_in_new_terminal_uses_handterm_exec_mode · function · L90-L126 — fn spawn_resume_in_new_terminal_uses_handterm_exec_mode()
- resumed_window_title_includes_server_name_when_registry_matches_socket · function · L130-L162 — fn resumed_window_title_includes_server_name_when_registry_matches_socket()
- spawn_selfdev_in_new_terminal_uses_handterm_exec_mode · function · L166-L203 — fn spawn_selfdev_in_new_terminal_uses_handterm_exec_mode()
- suppresses_stale_server_spawning_phase_when_listener_is_already_live · function · L207-L218 — async fn suppresses_stale_server_spawning_phase_when_listener_is_already_live()
- keeps_server_spawning_phase_while_listener_is_not_live · function · L222-L236 — async fn keeps_server_spawning_phase_while_listener_is_not_live()
