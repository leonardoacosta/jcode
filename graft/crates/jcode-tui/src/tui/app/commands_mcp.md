# crates/jcode-tui/src/tui/app/commands_mcp.rs

- CONFIG_PATHS · constant · L9-L10 — const CONFIG_PATHS: &str = "~/.jcode/mcp.json, ~/.claude.json, ~/.claude/mcp.json, \
- tool_list_lines · function · L12-L43 — pub(crate) fn tool_list_lines(
- show_tool_list · function · L45-L51 — pub(super) fn show_tool_list(app: &mut App, entries: Vec<crate::protocol::ToolListEntry>)
- poll_tool_list · function · L53-L70 — pub(super) fn poll_tool_list(app: &mut App) -> bool
- handle_tools_command · function · L72-L99 — pub(super) fn handle_tools_command(app: &mut App, trimmed: &str) -> bool
- server_state · function · L101-L118 — fn server_state(
- render_mcp_status · function · L122-L172 — pub(super) fn render_mcp_status(
- handle_mcp_command · function · L174-L213 — pub(super) fn handle_mcp_command(app: &mut App, trimmed: &str) -> bool
- start_login · function · L215-L256 — fn start_login(app: &mut App, config: &McpConfig, name: &str)
- tests · module · L259-L382 — mod tests
- tool_discovery_rows_fit_cells_and_sanitize_controls · function · L263-L290 — fn tool_discovery_rows_fit_cells_and_sanitize_controls()
- config · function · L292-L301 — fn config() -> McpConfig
- status_table_shows_scope_state_and_exposure · function · L304-L327 — fn status_table_shows_scope_state_and_exposure()
- degraded_status_includes_failure_and_stderr · function · L330-L343 — fn degraded_status_includes_failure_and_stderr()
- connected_server_without_tools_shows_zero · function · L346-L360 — fn connected_server_without_tools_shows_zero()
- empty_state_names_config_paths · function · L363-L367 — fn empty_state_names_config_paths()
- login_without_server_shows_usage · function · L370-L381 — fn login_without_server_shows_usage()
