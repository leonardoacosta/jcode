# crates/jcode-tui/src/tui/app/input/shell_commands.rs

- build_input_shell_command · function · L3-L17 — pub(super) fn build_input_shell_command(command: &str) -> std::process::Command
- combine_shell_output · function · L19-L47 — pub(super) fn combine_shell_output(stdout: &[u8], stderr: &[u8]) -> (String, bool)
- spawn_input_shell_command · function · L49-L94 — pub(super) fn spawn_input_shell_command(session_id: String, command: String, cwd: Option<String>)
