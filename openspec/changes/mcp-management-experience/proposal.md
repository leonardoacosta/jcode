---
execution:
  version: 1
  depends_on: [mcp-failure-backoff]
---
# Proposal

## Why

MCP management in jcode is opaque. Research (ranked, local evidence):

1. **Silent permanent failures** (`shadcn` 22x, `Grafana` 21x today; bad `cd` path and HTTP 502 visible only in `~/.jcode/logs`). Addressed by `mcp-failure-backoff`; this change surfaces it.
2. **Tools hidden behind search.** `searchable_tools` (`manager.rs:476`) exposes MCP only via `mcp_search`/`mcp call`; Claude Code users call `mcp__aperture__*` directly ~10k times in 60d. Frequently used tools cost an extra round trip each time.
3. **No status view.** No `/mcp` slash command or info-widget row; health is only in logs and the `mcp status` tool.
4. **Global-only mental model across workspaces.** Project `.mcp.json` is merged (`pool.rs:498`), but the user cannot see which scope a server came from, and personal/Priceless/Brown-specific servers (e.g. `shadcn` pinned to one repo path) start in every workspace.
5. **OAuth login is CLI-only** (`jcode mcp login`), forcing users out of the TUI.
6. **One-time Codex import only** (`protocol.rs:507`); Claude Code `~/.claude.json` servers are intentionally not imported and there is no manual import.
7. Startup cost: every enabled stdio server spawns `npx`/`op read` per pool even if unused.

## What Changes

- `/mcp` slash command: table of servers (scope source, state, tool count, last error class + stderr tail), with connect/disconnect/reload/login actions.
- Compact info-widget row: `MCP n/m` with degraded count; must not wrap.
- Per-server `expose: "direct" | "search"` (default `search`) and a global `mcp.pin_tools` list; direct tools register as `mcp__<server>__<tool>`.
- Per-server `workspaces` glob list; servers start only when the session cwd matches. Show scope (`global`, `project`) in `/mcp`.
- Lazy start: servers with `lazy: true` connect on first search/call, not at session start.
- `jcode mcp import --from claude|codex` (dry-run by default, writes on `--apply`).
- In-TUI OAuth: `/mcp login <server>` opens browser and completes the existing callback flow.

## Scope relative to MCPProxy

Long-lived self-hosted servers are migrating to `homelab-mcpproxy` (homelab changes `complete-mcpproxy-routing-cutover`, `migrate-mcp-to-all-supported-harnesses` H04). Repo-level and session-bound servers that cannot move stay locally launched by jcode. This change applies to both: local servers get full lifecycle management; the proxy appears as one server whose upstream status is surfaced in `/mcp`. Per-server routing verdicts are owned by `mcpproxy-connection-validation`.

## Capabilities

### New Capabilities

- `mcp-management`: Visibility, scoping, exposure and import of MCP servers.

## Open decisions

1. Default exposure. Recommended: `search` (protects context budget); `direct` opt-in per server. Alt: auto-pin top-N by usage (adaptive but less predictable).
2. Lazy default. Recommended: `lazy: false` for compatibility; consider flipping after telemetry. Alt: default lazy (faster startup, first-call latency).
3. Import of secrets. Recommended: copy env var names, never values, unless `--include-env`.
