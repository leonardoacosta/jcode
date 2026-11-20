---
execution:
  version: 1
  depends_on: []
---
# Proposal

## Why
New TUI clients can connect but wait indefinitely for session history because cached MCP advertisement synchronously waits for shared health metadata. Logs on 2026-11-02 show Subscribe handlers unfinished beyond 60 seconds while established sessions remain active.

## What Changes
- Publish cached tool status without awaiting live MCP health; keep cached proxies available synchronously.
- Release pool connection coordination before waiting on other maps, preserving connection leadership and notification semantics.
- Collect health from separately scoped snapshots.
- Add deterministic contention regressions and isolated wire-level startup acceptance.

## Capabilities

### New Capabilities
None.

### Modified Capabilities
- `mcp-connection-health`: health collection must not block session identity/history bootstrap; unrelated connection coordination must progress during map contention.

## Impact
Changes are limited to MCP registration, shared-pool coordination, and their tests. No protocol, configuration, dependency, or migration changes. Local validation uses a separate daemon socket; shared-runtime installation, commits, and publication are outside authorization.
