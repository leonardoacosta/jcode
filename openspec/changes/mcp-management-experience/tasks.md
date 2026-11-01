## 1. Visibility
- [ ] 1.1 `/mcp` slash command listing servers with scope/state/tools/error; snapshot test incl. empty state
- [ ] 1.2 Info-widget MCP row with ellipsis truncation; narrow-width test

## 2. Surfacing and scoping
- [ ] 2.1 Config fields `expose`, `mcp.pin_tools`, `workspaces`, `lazy`; parsing tests
- [ ] 2.2 Register direct tools as `mcp__<server>__<tool>`; omit when degraded; tests
- [ ] 2.3 Workspace glob gating and lazy connect in pool; tests for out-of-scope and first-call connect

## 3. Import and login
- [ ] 3.1 `jcode mcp import --from claude|codex [--apply]` with dry-run default and conflict skip; tests
- [ ] 3.2 `/mcp login <server>` reusing `oauth.rs` flow; timeout test

## 4. Verify
- [ ] 4.1 `cargo test -p jcode-base mcp`, TUI tests, `openspec validate mcp-management-experience --strict`
