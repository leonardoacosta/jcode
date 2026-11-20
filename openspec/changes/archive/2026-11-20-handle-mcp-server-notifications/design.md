# Design

## Context

At the pinned source revision, MCP client capabilities are empty and `RemoteState::dispatch` drops server-originated notifications. The current MCP specification, 2026-07-28, replaces `resources/subscribe` with a filtered `subscriptions/listen` stream. The older 2025-11-25 contract uses per-feature server capability flags and `resources/subscribe`. Implementations must not combine these wire contracts.

## Decisions

### Version-gated protocol support

Use the negotiated MCP protocol version to select the notification contract. For 2025-11-25, use advertised `resources.subscribe` and the feature-specific `listChanged` flags. For 2026-07-28, open `subscriptions/listen`, request only configured filters, correlate acknowledgments and events by subscription ID, and tolerate unsupported acknowledged filters. Do not send the old subscribe RPC under the current protocol version. Unknown protocol versions do not enable notifications.

This avoids sending obsolete methods to current servers while retaining compatibility with the pinned client's supported historical protocol. The implementation must verify what protocol versions the current transport/client can negotiate before claiming current-version support; if a transport cannot provide the long-lived response stream, that version does not advertise notification support on that transport.

### Invalidation, not event execution

Treat every supported notification as an invalidation hint. Re-read changed resources and refresh changed catalogs from the same server. Preserve last-known state on refresh failure. This handles loss, duplication, and reordering without requiring exactly-once delivery, which the protocol does not guarantee.

### Independent wake authorization

Keep refresh permission separate from agent-wake permission. Per-server allowlists opt in notification classes/resource URIs for refresh, and independently opt in classes for a session or ambient wake. A wake routes only a server/class identifier through existing admission and queue mechanisms; notification content and resource data remain untrusted. This avoids turning arbitrary server data into instructions or authority.

### Bounded work

Coalesce by server plus notification target and keep queue/rate limits per server. On overload, discard/coalesce only invalidation hints, then reconcile from authoritative state when the server is available. Never queue raw payloads or allow one event source to block interactive MCP requests.

## Risks

A long-lived current-version subscription stream may require transport work beyond the pinned POST-response dispatch. Support must be gated by actual transport behavior and tested with fragmented, closed, and cancelled streams. Re-reading can fail or return large/untrusted data, so existing resource-size and content policies remain in force. Ambient queue semantics are owned by existing schedule work; this change must reuse them and must not promise reliable delivery beyond their contract.
