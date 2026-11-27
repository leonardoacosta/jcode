# crates/jcode-base/src/terminal_launch.rs

- apply_scoped_client_terminal_env · function · L10-L13 — pub fn apply_scoped_client_terminal_env(command: &mut std::process::Command)
- client_terminal_env_snapshot · function · L15-L20 — pub fn client_terminal_env_snapshot() -> Vec<(String, String)>
- configured_spawn_hook · function · L26-L34 — pub fn configured_spawn_hook() -> Option<String>
- runtime_env_facade_tests · module · L37-L62 — mod runtime_env_facade_tests
- app_core_portable_runtime_filter_reference_is_exported · function · L39-L42 — fn app_core_portable_runtime_filter_reference_is_exported()
- contextual_companion_error_requests_safe_local_fallback · function · L45-L61 — fn contextual_companion_error_requests_safe_local_fallback()
- spawn_command_in_new_terminal · function · L71-L84 — pub fn spawn_command_in_new_terminal(command: &TerminalCommand, cwd: &Path) -> Result<bool>
- try_spawn_via_companion · function · L88-L96 — fn try_spawn_via_companion(command: &TerminalCommand, cwd: &Path) -> bool
- try_spawn_via_companion_with · function · L100-L121 — fn try_spawn_via_companion_with(
- try_spawn_via_configured_hook · function · L128-L141 — pub fn try_spawn_via_configured_hook(command: &TerminalCommand, cwd: &Path) -> bool
- spawn_via_hook · function · L143-L170 — fn spawn_via_hook(hook: &str, command: &TerminalCommand, cwd: &Path) -> Result<()>
- tests · module · L173-L244 — mod tests
- spawn_via_hook_runs_hook_with_metadata_env · function · L178-L221 — fn spawn_via_hook_runs_hook_with_metadata_env()
- spawn_via_hook_reports_immediate_nonzero_exit · function · L224-L243 — fn spawn_via_hook_reports_immediate_nonzero_exit()
