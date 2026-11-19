# Tasks

All items are implementation work. Authoring this proposal does not complete them. Recheck the pinned source, notification-dispatch dependency, and shared-file ownership before implementation. Do not add a second task registry or persist credential values.

## 1. Extension negotiation and response parsing

- [ ] 1.1 Model `io.modelcontextprotocol/tasks` capability negotiation for `tools/call`, parse the 2026-07-28 `resultType: "task"` response and task fields. Add MCP protocol/client fixture tests for supported and unsupported peers, malformed task handles, and synchronous fallback.
- [ ] 1.2 Add `tasks/get` and `tasks/cancel` requests, including terminal result/error, `input_required`, not-found/expiry, and cancellation error handling. Verify exact wire shapes against the official 2026-07-28 extension fixtures; do not implement the incompatible 2025-11-25 core `tasks/result` protocol.

## 2. Durable tracking and recovery

- [ ] 2.1 Reuse existing background-task persistence/UI records to store MCP server and authorization-context identity references, remote task ID, original call/session association, status, timestamps, TTL, and result/error. Add `mcp_task_tracking` tests for restart/reconnect, no original-call replay, identity mismatch, and persistence failure before exposing a task handle.
- [ ] 2.2 Implement polling from persisted task state. Honor `pollIntervalMs`, bound retry backoff, stop at terminal status/expiry, and retain last-known state during disconnect. Test lost notifications, transient transport errors, stale notifications, server restart, and expired/not-found tasks.
- [ ] 2.3 Integrate confirmed cancellation and cancellation races with the existing background-task UI. Test cancellation requested versus confirmed, completed/failed races, transport loss, and refusal to cancel terminal tasks.
- [ ] 2.4 Surface MCP task kind, remote state, available result/error, `input_required`, expiry, and unresolved-server state in the existing task UI. Verify keyboard operation, empty/loading/error states, and narrow-terminal row truncation without adding a separate task view or ledger.

## 3. Notification dispatch integration

- [ ] 3.1 Integrate through `handle-mcp-server-notifications` dispatch plumbing. Treat `notifications/tasks` as an optional wake hint, correlate by configured server identity and task ID, and refresh via `tasks/get`; do not treat it as the durable state source. Verify notifications cannot update another server's task or regress a newer state.

## 4. Integrated verification

- [ ] 4.1 Run focused tests: `cargo test -p jcode-base mcp_task_tracking`, `cargo test -p jcode-app-core mcp_task_tracking`, and relevant background-task UI tests. Run `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `git diff --check`, and `openspec validate track-mcp-tasks --strict`. Exercise every scenario, including lost notification, daemon/server restart, expiry, and cancel race; record exact commands and results.
- [ ] 4.2 Obtain independent final review against the 2026-07-28 extension and this spec after fresh gates. Repair findings, rerun affected checks and review, then complete implementation tasks only from observed evidence.
