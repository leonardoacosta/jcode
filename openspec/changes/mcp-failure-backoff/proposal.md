# Proposal

## Why

Broken MCP servers are retried forever on a fixed 30s cooldown (`FAILED_CONNECT_RETRY_COOLDOWN`, `crates/jcode-base/src/mcp/pool.rs:20`). Today `shadcn` failed 22x and `Grafana` 21x. Logs show causes are not transient: `shadcn` runs `cd /home/nyaptor/dev/personal/dash/nova/packages/ui` which does not exist (143 stderr lines), and `Grafana` (`npx mcp-remote`) gets HTTP 502. Each retry spawns `npx`/`op read`, costing startup time and log noise, and the user never sees why.

## What Changes

- Replace the fixed cooldown with per-server exponential backoff (30s doubling, capped at 30min), reset on success or config change.
- Capture the last stderr lines of a failed stdio server and attach a classified reason (`spawn_failed`, `exited_before_init`, `init_timeout`, `auth_required`, `remote_http_error`) to the failure record.
- After 5 consecutive failures, mark the server `degraded`: no automatic reconnects until a manual `mcp connect`/reload or config edit.
- Emit one deduplicated user-visible notice per server per state change instead of per attempt.

## Capabilities

### New Capabilities

- `mcp-connection-health`: Backoff, failure classification and degraded state for MCP servers.

## Impact

`crates/jcode-base/src/mcp/{pool.rs,manager.rs,client.rs}`. No config format change.

## Open decisions

1. Degraded threshold. Recommended: 5 consecutive failures. Alternatives: 3 (faster quiet, more false degrades on flaky networks) or never (current spam).
