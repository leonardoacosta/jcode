---
execution:
  version: 1
  depends_on: []
---
Roadmap owner: [P02: Discoverable native tools, MCP and skills provenance](../../../docs/plans/PRODUCT_ROADMAP.md#p02-discoverable-native-tools-mcp-and-skills-provenance)

## Why

`RemoteState::dispatch` discards inbound server messages without `result` or `error`, because the client advertises no capabilities (`crates/jcode-base/src/mcp/remote.rs:143-162`). The pinned client capability object is empty and server capability parsing only models existing tool, resource and prompt flags (`crates/jcode-base/src/mcp/protocol.rs:76-115`). Jcode cannot receive updates or refresh MCP catalogs after changes.

The current MCP specification is **2026-07-28**. It replaces `resources/subscribe` with `subscriptions/listen`, whose explicit filters cover resource URI updates and tools/prompts/resources list changes. The 2025-11-25 specification instead defines `resources/subscribe`, `notifications/resources/updated`, and the capability flags `resources.subscribe` and `listChanged`. These versions must not be mixed: `resources/subscribe` is a legacy contract, not the current method.

## What Changes

- Negotiate supported protocol-version capabilities and notification delivery. For 2025-11-25, support `resources.subscribe`, `resources.listChanged`, and tool/prompt list-change capabilities. For 2026-07-28, use `subscriptions/listen` with explicit notification filters, acknowledgment correlation, and cancellation; do not send the removed `resources/subscribe` method.
- Dispatch supported resource-update and catalog-change notifications. Treat resource updates as invalidation hints and re-read the resource; refresh tools, prompts, or resources catalogs after their matching list-changed hint.
- Restore subscriptions after reconnect, then reconcile by re-reading authoritative resource/catalog state. Do not treat missed or duplicate notifications as authoritative state.
- Bound event work with deduplication, coalescing, and per-server rate limits. Add per-server notification allowlists that separately control catalog refresh and whether a notification may wake an agent.
- Route permitted wake requests to a configured session or the existing ambient schedule/queue path. Notification payloads and re-read resource content are untrusted data, not instructions or authorization.

## Capabilities

### New Capabilities

- `mcp-server-notifications`: Negotiated MCP notifications, catalog invalidation, safe wake policy, and reconnect reconciliation.

### Modified Capabilities

None.

## Impact

Primary surfaces: `crates/jcode-base/src/mcp/protocol.rs`, `crates/jcode-base/src/mcp/remote.rs`, MCP client/manager and catalog refresh paths, per-server MCP configuration, and the existing session/ambient wake routing. Integrate with existing ambient queue behavior rather than adding another scheduler. See MCP specification 2026-07-28 [Subscriptions](https://modelcontextprotocol.io/specification/2026-07-28/basic/patterns/subscriptions) and historical 2025-11-25 [Resources](https://modelcontextprotocol.io/specification/2025-11-25/server/resources).

## Non-goals

MCP Tasks (`notifications/tasks`) are a separate change, `track-mcp-tasks`, and are excluded. No generic event bus or vendor-specific events; no autonomous instruction execution from notifications; no exactly-once delivery promise; no change to the MCP wire contract beyond explicitly negotiated versions; no new scheduler or replacement for the ambient queue.
