---
execution:
  version: 1
  depends_on: []
---
Roadmap owner: [P05: Durable overnight, scheduling and recovery](../../../docs/plans/PRODUCT_ROADMAP.md#p05-durable-overnight-scheduling-and-recovery)

Related (not a hard dependency): `shared-server-crash-resilience` restarts the server; this change only needs a startup hook, which exists today.

## Why

The overnight supervisor is an in-process `tokio::spawn` (`overnight.rs:217-219`). If the server restarts or crashes, no code resumes or closes the run: the only status writes are at start (`overnight.rs:111`), on completion, or on supervisor error (`overnight.rs:203`), so a killed server leaves the manifest `Running` indefinitely and the morning view reports a live run that does not exist. `reattach-background-tasks-after-reload` covers bg processes and `shared-server-crash-resilience` covers chat sessions; neither covers overnight runs.

## What Changes

- Record the supervisor's server PID and a heartbeat timestamp in the manifest.
- On server start, scan manifests with `Running`/`CancelRequested` whose supervisor is gone: `CancelRequested` → `Cancelled`; past wake time → `Interrupted` with a final report; otherwise resume the supervisor from the persisted manifest with a `run_resumed` event.
- Never resume a run the user cancelled.

## Non-scope

- Background task reattach (covered by `reattach-background-tasks-after-reload`).
- Swarm worker recovery (`recover-swarm-workers-after-reload`).

## Capabilities

### New Capabilities

- `overnight-restart-recovery`

## Impact

`overnight.rs` manifest fields and a startup reconciliation hook in server boot; TUI overnight status shows `interrupted`/`resumed`.

## Pending user decision

1. Auto-resume vs mark-only. Options: (a) auto-resume runs before wake time; (b) mark `Interrupted` and require user resume. Recommended: (a), the run was already authorized for this window; (b) remains for runs past wake time.

The recommendation above is not a selected default. The dependent behavior remains blocked until this choice is settled; this proposal does not authorize it.
