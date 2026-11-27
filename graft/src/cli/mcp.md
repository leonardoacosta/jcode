# src/cli/mcp.rs

- remote_server · function · L9-L26 — fn remote_server<'a>(config: &'a McpConfig, name: &str) -> Result<(&'a McpServerConfig, &'a str)>
- static_auth · function · L28-L33 — fn static_auth(server: &McpServerConfig) -> bool
- auth_status · function · L35-L47 — fn auth_status(server: &McpServerConfig, stored: Option<&oauth::OAuthStatus>) -> &'static str
- server_rows · function · L49-L69 — fn server_rows(config: &McpConfig) -> Result<Vec<Value>>
- render_list · function · L71-L91 — fn render_list(rows: &[Value]) -> String
- import_plan · function · L93-L115 — fn import_plan(target: &mut McpConfig, source: &str, imported: McpConfig, apply: bool) -> String
- run · function · L117-L210 — pub(crate) async fn run(action: McpCommand) -> Result<()>
- tests · module · L213-L328 — mod tests
- config · function · L216-L222 — fn config() -> McpConfig
- mcp_cli_resolves_configured_remote_server_and_actionable_errors · function · L225-L257 — fn mcp_cli_resolves_configured_remote_server_and_actionable_errors()
- mcp_cli_list_is_sorted_redacted_and_shows_static_auth_precedence · function · L260-L269 — fn mcp_cli_list_is_sorted_redacted_and_shows_static_auth_precedence()
- mcp_cli_status_does_not_claim_stored_tokens_are_valid · function · L272-L292 — fn mcp_cli_status_does_not_claim_stored_tokens_are_valid()
- mcp_cli_list_reports_corrupt_oauth_store_without_exposing_contents · function · L295-L306 — fn mcp_cli_list_reports_corrupt_oauth_store_without_exposing_contents()
- import_dry_run_reports_and_skips_conflicts · function · L309-L327 — fn import_dry_run_reports_and_skips_conflicts()
