# Tasks

All tasks are implementation work, not completed by authoring this proposal. Keep MCP Tasks out of this change; `track-mcp-tasks` owns that protocol extension. Confirm protocol and transport support at implementation time before advertising any wire version.

## 1. Protocol negotiation and dispatch

- [ ] 1.1 Inventory protocol-version negotiation, stdio/HTTP/SSE message readers, dispatch owners, catalog caches, reconnect handling and session/ambient wake paths in `jcode-base` and `jcode-app-core`; record exact paths and tests before implementation.
- [ ] 1.2 Add version-gated client/server capability modeling. Implement legacy 2025-11-25 `resources/subscribe` and `listChanged` only when negotiated and advertised; implement 2026-07-28 `subscriptions/listen` only where the transport supports its long-lived response stream, with filter acknowledgment, subscription-ID correlation and cancellation. Add focused protocol and transport tests for unsupported versions, filters, fragmented streams, closure and malformed messages.
- [ ] 1.3 Route valid `notifications/resources/updated` and tools/prompts/resources list-changed messages to the owning manager. Re-read the named resource or corresponding catalog from the same server/auth context; preserve last-known state on failures. Test stale, unknown, malformed and wrong-subscription notifications.

## 2. Reconnect, bounds and policy

- [ ] 2.1 Restore supported subscriptions after reconnect, then reconcile subscribed resources and opted-in catalogs with ordinary reads/lists. Add tests for missed, duplicated, reordered notifications, changed capabilities and failed reconnects.
- [ ] 2.2 Add per-server keyed deduplication/coalescing, bounded queues and rate limits without blocking interactive MCP requests. Test burst, overflow and independent-server fairness.
- [ ] 2.3 Add per-server notification allowlists that separately govern catalog/resource refresh and agent wakes. Ensure unknown or unlisted notifications cannot wake an agent and payloads/content never become trusted instructions. Test configured and denied cases.
- [ ] 2.4 Route permitted events through existing session admission or the ambient schedule/queue path. Coalesce duplicate pending wakes and test absent/busy/revoked sessions plus ambient disabled/paused behavior. Reuse current queue ownership and do not add a parallel scheduler.

## 3. Integrated verification

- [ ] 3.1 Run focused tests with `cargo test -p jcode-base mcp::protocol_tests`, `cargo test -p jcode-base mcp::remote_tests`, and `cargo test -p jcode-app-core ambient::runner`; add/adjust named tests as needed for notification parsing, dispatch, subscriptions, catalog refresh and wake policy. Then run `cargo clippy --workspace --all-targets -- -D warnings`, `git diff --check`, and `openspec validate handle-mcp-server-notifications --strict`.
- [ ] 3.2 Verify each spec scenario against transport fakes and existing session/ambient paths, including failures and bounds. Record exact commands/results; do not claim current-version transport support without a test exercising that path.
