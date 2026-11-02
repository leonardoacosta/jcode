## 1. Visibility
- [x] 1.1 `/mcp` slash command listing servers with scope/state/tools/error; snapshot test incl. empty state
  - Optional `McpStatus.health` carries session-scoped failure class and redacted stderr for shared and session-owned servers. Lazy/manual connection outcomes publish updates. Legacy-wire, scope-isolation, redaction, owned-failure/reset, and TUI table tests pass; private-socket TUI `/mcp` confirmed degraded and connected-zero-tool rows.
- [x] 1.2 Info-widget MCP row with ellipsis truncation; narrow-width test
  - Header shows connected/total and `!N` degraded count using the existing display-cell truncation; tests cover widths 0–59 and connected servers with zero tools.

## 2. Surfacing and scoping
- [x] 2.1 Config fields `expose`, `mcp.pin_tools`, `workspaces`, `lazy`; parsing tests
- [x] 2.2 Register direct tools as `mcp__<server>__<tool>`; omit when degraded; tests
- [x] 2.3 Workspace glob gating and lazy connect in pool; tests for out-of-scope and first-call connect

## 3. Import and login
- [x] 3.1 `jcode mcp import --from claude|codex [--apply]` with dry-run default and conflict skip; tests
- [x] 3.2 `/mcp login <server>` reusing `oauth.rs` flow; timeout test
  - Note: timeout is oauth.rs's existing 5-minute callback bound (covered by oauth_tests); TUI test covers usage/dispatch only.

## 4. Verify
- [x] 4.1 `cargo test -p jcode-base mcp`, TUI tests, `openspec validate mcp-management-experience --strict`
  - Note: all pass except pre-existing `mcp_collision_manual_lifecycle_real_stdio` (fails on base too: untrusted mise config in test env).
