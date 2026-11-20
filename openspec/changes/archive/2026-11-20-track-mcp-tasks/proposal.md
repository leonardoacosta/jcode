---
execution:
  version: 1
  depends_on: [handle-mcp-server-notifications]
---
# Track long-running MCP tool tasks

Roadmap owner: [P02: Discoverable native tools, MCP and skills provenance](../../../docs/plans/PRODUCT_ROADMAP.md#p02-discoverable-native-tools-mcp-and-skills-provenance).

## Why

Jcode routes MCP tool calls through `McpManager::call_tool` and the MCP client, but the pinned protocol models no task capability, task result, or task lifecycle (`crates/jcode-base/src/mcp/manager.rs:658-713`; `crates/jcode-base/src/mcp/protocol.rs:77`). Jcode already has a durable background-task manager and user-facing task surfaces to reuse, rather than creating a second task ledger (`crates/jcode-base/src/background.rs:35-67`; `crates/jcode-app-core/src/server/background_tasks.rs:78-161`).

The official MCP Tasks extension snapshot inspected here is dated 2026-07-28. It uses extension negotiation (`io.modelcontextprotocol/tasks`), a `resultType: "task"` task-augmented `tools/call` result, `tasks/get`, `tasks/update`, `tasks/cancel`, and `notifications/tasks`. The separate 2025-11-25 experimental core Tasks page instead describes `capabilities.tasks`, `tasks/result`, and `notifications/tasks/status`. This change targets the 2026-07-28 extension; it does not mix the incompatible wire contracts.

## What Changes

- Negotiate the `io.modelcontextprotocol/tasks` extension through per-request client capabilities and server `server/discover` capabilities. Accept ordinary `tools/call` results or task handles at the server's discretion; servers without the extension keep synchronous calls.
- Persist server identity, task ID, original tool/call context, lifecycle state, timestamps, TTL, and result/error association in Jcode's existing background task tracking. Resume polling after reconnect or daemon restart without replaying the original tool call.
- Treat `notifications/tasks` as optional status hints. Poll `tasks/get` as the source of current state, honoring `pollIntervalMs` with bounded backoff when hints are lost or transport is unavailable.
- Support `tasks/cancel`, terminal-state races, task expiry, and visible status/result/error in the existing background-task UI. Never claim cancellation succeeded if the server reports a competing terminal state.

## Capabilities

### New Capabilities

- `mcp-task-tracking`: durable tracking and presentation of MCP task-augmented tool calls.

## Impact

Primary integration points: `crates/jcode-base/src/mcp/protocol.rs`, MCP client and manager call/result paths, existing background task persistence, server notification dispatch (owned by dependency `handle-mcp-server-notifications`), and existing background-task UI. Use the server configuration and authorization identity already bound to the MCP connection for every poll, result, and cancellation request.

## Non-goals

No support for the incompatible 2025-11-25 core Tasks wire protocol in this change; no task creation/hosting by Jcode; no arbitrary MCP event subscriptions; no replay of a tool call after an ambiguous response; no new independent task ledger or UI; no change to synchronous behavior for servers that do not negotiate the extension; no silent fallback to another server or authorization context.
