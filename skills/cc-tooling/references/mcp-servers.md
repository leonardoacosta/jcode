# MCP Servers

## MCP Server Inventory

| Server                 | Purpose                       | Auth                  | Docker | Status    |
| ---------------------- | ------------------------------ | --------------------- | ------ | --------- |
| `sequential-thinking`  | Multi-step reasoning chains   | None                  | No     | Connected |
| `context7`             | Library docs search           | API key (env)         | No     | Connected |
| `sentry`               | Error tracking                | Auth token (env)      | No     | Connected |
| ~~github~~             | **Removed** — use `gh` CLI directly     | —                     | —      | Removed   |
| ~~vercel~~             | **Removed** — use `vercel` CLI directly | —                     | —      | Removed   |
| `figma`                | Design-to-code workflows      | OAuth (browser)       | No     | Connected |
| ~~nova-\*~~ (9 servers) | **Deprecated** — use `nx-send` instead   | —                     | —      | Removed   |

**Deprecated:** nova-\* (`nova-memory`, `nova-messages`, `nova-channels`, `nova-discord`,
`nova-teams`, `nova-schedule`, `nova-graph`, `nova-meta`, `nova-azure`) — full deprecation, not a
re-scoping (cowork-audit-remediation C3, decided 2026-07-12). `nx-send` is the successor for
nova's notification/messaging surface. This is nova's THIRD removal from `mcp.json` (git history:
`1dc2d066` removed, `c55625e6` explicitly re-added all 9 "to restore access", now removed again)
— `scripts/bin/validate-cc` Tier 3's `nova-absent` row now catches a future re-add automatically
instead of relying on someone noticing.

**Removed:** `playwright` MCP — replaced by `agent-browser` CLI tool (Rust, Vercel Labs).
Use via Bash: `agent-browser open <url>`, `agent-browser snapshot -i`, `agent-browser click @ref`.
All `mcp__playwright__*` tools are denied in `settings.json`. Playwright MCP also pollutes repos
with `.playwright-mcp/` user data directories.

**Benchmark result** (methodology: `agent-tooling` skill's `references/mcp-selection.md`
§ Choosing Between Two Tools for the Same Job): `agent-browser -i` produces 86% fewer tokens than
Playwright MCP snapshots (4.4K vs 31.5K tokens across 4 test pages). Prefer `-i` as the default
mode; drop to full snapshot only when the task needs DOM detail `-i` omits.

Config location: `~/.claude/mcp.json`.

### MCP Resilience Bundle (env)

Three independent `settings.json` env knobs harden MCP against connection + tool-call hangs.
They are different axes — set all three for long autonomous runs (`/apply:all`, ralph loops):

| Env var | Axis | cc value | What it bounds |
| ------- | ---- | -------- | -------------- |
| `MCP_CONNECTION_NONBLOCKING` | connection mode | `1` | `-p`/parallel-agent startup doesn't block on a slow server handshake |
| `MCP_TIMEOUT` | connection startup | `5000` (5s) | server-init timeout — fail fast if a server won't connect |
| `CLAUDE_CODE_MCP_TOOL_IDLE_TIMEOUT` | per-tool-call idle | `180000` (3min) | aborts a hung *remote* tool call (Slack/Atlassian claude.ai-proxy stalls are the known culprits) instead of freezing the orchestrator |

The idle-timeout default is 5min when unset; cc pins it to 3min (set 2026-06-27 via `/workflow:evolve`,
v2.1.187 signal) for faster wave recovery while leaving headroom for legitimately slow tools (large
Notion/Linear queries). `MCP_TOOL_TIMEOUT` (the ~28h hard wall-clock cap) is intentionally left at
default — it is not the recovery lever. Capability discovery (`tools/list`) auto-retries transient
network errors with backoff since v2.1.191 — no config needed.

## MCP Selection Criteria

Portable — see `agent-tooling` skill's `references/mcp-selection.md` for the criteria table,
anti-patterns, and adding-a-new-server procedure. When you add a server here, also add it to the
§ MCP Server Inventory table above (this repo's inventory, not upstream's job to track).
