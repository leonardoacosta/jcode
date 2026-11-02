---
execution:
  version: 1
  depends_on: [configure-task-and-worktree-policy]
---
Roadmap owner: [P05: Durable overnight, scheduling and recovery](../../../docs/plans/PRODUCT_ROADMAP.md#p05-durable-overnight-scheduling-and-recovery)

Related (not a hard dependency): `shared-server-crash-resilience` restarts the server; this change only needs a startup hook, which exists today.

## Why

The overnight supervisor is an in-process `tokio::spawn` (`overnight.rs:217-219`). If the server restarts or crashes, no code resumes or closes the run: the only status writes are at start (`overnight.rs:111`), on completion, or on supervisor error (`overnight.rs:203`), so a killed server leaves the manifest `Running` indefinitely and the morning view reports a live run that does not exist. `reattach-background-tasks-after-reload` covers bg processes and `shared-server-crash-resilience` covers chat sessions; neither covers overnight runs.

## What Changes

- Record the supervisor's server PID and a heartbeat timestamp in the manifest.
- On server start, scan manifests with `Running`/`CancelRequested` whose supervisor is gone: `CancelRequested` → `Cancelled`; otherwise send the task through current configured admission/triage policy before resuming from its manifest with a `run_resumed` event. A historical overnight wake time alone neither grants nor denies task authorization.
- Revalidate task scope, remaining budgets, dependencies, cancellation and live ownership against current configuration. A configuration change cannot expand the task's persisted authorization; ambiguous work remains interrupted with an actionable reason.
- Never resume a run the user cancelled.

## Non-scope

- Background task reattach (covered by `reattach-background-tasks-after-reload`).
- Swarm worker recovery (`recover-swarm-workers-after-reload`).

## Capabilities

### New Capabilities

- `overnight-restart-recovery`

## Impact

`overnight.rs` manifest fields and a startup reconciliation hook in server boot; TUI overnight status shows `interrupted`/`resumed`.

## Resolved direction and remaining design work

The user selected automatic resumption of eligible work, then explicitly refined eligibility to a configuration-driven task triage mechanism rather than an overnight clock window. Eligibility must therefore be re-evaluated from task authorization and configured task/worktree policy, not inferred from time alone. Cancellation and finite task/resource limits remain enforced.

Research the existing global config and worktree mechanisms before finalizing the typed task/worktree schema and precedence. That schema is design work, not a request for blanket proposal approval. Until the admission contract is specified, the recovery implementation task remains dependent on it; independent manifest/ownership tests can proceed when implementation is separately requested. Research/mining does not authorize worktree creation.
