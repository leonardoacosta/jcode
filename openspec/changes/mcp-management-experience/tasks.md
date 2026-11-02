## 1. Visibility
- [x] 1.1 `/mcp` slash command listing servers with scope/state/tools/error; snapshot test incl. empty state
  - Note: failure class + stderr tail live in the daemon pool and are not on the wire yet (`McpStatus` carries `name:count`); `/mcp` points to `mcp status`/logs. Follow-up: extend `McpStatus` with health.
- [x] 1.2 Info-widget MCP row with ellipsis truncation; narrow-width test
  - Note: header row shows `mcp ready/total`; degraded count pending the same wire follow-up.

## 2. Surfacing and scoping
- [x] 2.1 Config fields `expose`, `mcp.pin_tools`, `workspaces`, `lazy`; parsing tests
- [x] 2.2 Register direct tools as `mcp__<server>__<tool>`; omit when degraded; tests
- [x] 2.3 Workspace glob gating and lazy connect in pool; tests for out-of-scope and first-call connect

## 3. Import and login
- [x] 3.1 `jcode mcp import --from claude|codex [--apply]` with dry-run default and conflict skip; tests
- [x] 3.2 `/mcp login <server>` reusing `oauth.rs` flow; timeout test
  - Note: timeout is oauth.rs's existing 5-minute callback bound (covered by oauth_tests); TUI test covers usage/dispatch only.

## 4. Verify
- [ ] 4.1 `cargo test -p jcode-base mcp`, TUI tests, `openspec validate mcp-management-experience --strict`
