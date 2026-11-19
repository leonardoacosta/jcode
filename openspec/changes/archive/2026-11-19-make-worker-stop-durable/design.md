# Design

## Context

See [proposal](proposal.md) for motivation and [research](../../../docs/research/worker-session-control-2026-11-07.md) for source/incident evidence. The target is one existing daemon. Stop removes session/swarm entries while a live-turn future can retain its owned agent guard. A later lightweight Comm request can recreate membership from a session stub with no owner. Existing process-global turn-cancel signals, session storage, swarm snapshots/mutation records, waits and native mutation claims provide the reuse points.

Relevant source exemplars: `server/comm_session.rs:1077–1328`, `server/client_lightweight_control.rs:107–222`, `server/live_turn.rs:102–202`, `turn_cancel_registry.rs:53–169`, `server/state.rs:750–775`, `server/comm_await.rs:13–74,352–387`, `server/swarm/claims.rs:424–469`, all under `crates/jcode-app-core/src/`. These are inspection evidence, not a fresh runtime stop test. Source HEAD is `fd25ee0c57d71d6119bb7372a9cf7584f150d24e`.

## Goals / Non-Goals

**Goals:** one authority for identity, owner, admission and cancellation within this daemon; no false stopped/completed result; safe new-generation work; reuse current persistence and public swarm interfaces. Native mutation settlement includes blocking filesystem work that outlives an async caller.

**Non-Goals:** another service/database, multi-host coordination, a general workflow engine, new permission grants, arbitrary OS-process termination, universal shell sandboxing, external-effect rollback, Desktop implementation, unrelated application repair or automatic shared-daemon rollout.

## Decisions

### 1. Session control outlives membership

Extend the existing persisted session/control representation with schema version, stable session ID, retained owner edge, monotonic control revision, last allocated execution generation, admitted generation, stop disposition and latest stop request identity. Swarm membership and UI status are projections, not the source of runnability. Keep one authoritative owner/control record across removal, member GC and snapshot recreation, rather than storing independently writable authority copies in every registry.

Resolve exact IDs through this daemon's live/persisted sessions before authorization. Retain the owner chain needed for the existing self/ancestor policy when members leave; never infer a new owner from a coordinator slot or friendly name. Authorization failures stay distinct from unknown identity. Aliases remain swarm-local. Preserve the current explicit `force` boundary for non-owned sessions rather than turning global lookup into broader rights. A reconnect or session attachment rename must reconcile all old references under the same control revision, not duplicate a live execution.

Alternative rejected: deleting membership as the stop record. Deletion loses both revocation and ownership and is reversible by legitimate auto-registration code. New manager/database is unnecessary for this daemon boundary.

### 2. Admission and stop share a per-session linearization boundary

Use a per-session control operation lock, revision-checked persistence and durable atomic replacement through existing storage helpers. No global map/agent lock may remain held across provider work, cancellation joins or filesystem settlement. Existing swarm async persistence ordering remains intact.

Under the session operation boundary, stop checks authority, closes local admission, persists the revocation for generation G and records the stop request. Only then acknowledge `stop_requested`. Persistence failure keeps a local fail-closed barrier and returns error, not durable acceptance. Retry reconciles that failure explicitly. Stop intent is not cleared by mutation TTL expiry.

Every path that starts work must obtain admission: spawn, assigned task run, direct wake/resume/retry, scheduled delivery, recovery, attached client message, lightweight headless control and native mutation dispatch. Admission carries `(session_id, generation)` through nested/batch tool contexts. A missing membership entry cannot bypass this check. Legacy requests bind at acceptance; they cannot supply stale work as a fresh explicit resume.

A new accepted work request allocates a generation even if queued. Track the active generation separately from pending generation identities. Its actual execution waits for the predecessor to settle. Stop revokes the active generation and every accepted pending generation at its linearization point, recording that revoked-through generation boundary durably. Work dedup uses request identity plus generation, not merely session/status strings. Native tool revocation validation and reservation of outstanding work occur atomically relative to stop, so the check-to-mutation interval cannot escape settlement accounting. Resume allocates above the highest previously allocated generation, including revoked queued work, rather than assuming the active generation plus one is unused.

### 3. Stop state and work outcome are separate

| Current control | Event / guard | Next control | Effect |
| --- | --- | --- | --- |
| Runnable/idle or admitted G | Authorized stop, durable persistence succeeds | `stop_requested` G | Revoke current/pending pre-stop admission and preserve owner/claims |
| `stop_requested` G | Cancellation dispatched | `draining` G | Signal actual active turns and await owned work |
| `draining` G | All admitted turns/tools/known child work settled | `stopped` G | Persist settlement, release G claims and allow later explicit resume |
| `draining` G | Deadline, unknown child liveness or failed settlement accounting | `unresolved` G | Keep revocation and claims, identify outstanding categories |
| `unresolved` G | Later reconciliation proves settlement | `stopped` G | Persist evidence and release only G resources |
| `stopped` G | Authorized explicit new-work request | Runnable N, N greater than every previously allocated generation | Atomically allocate one new generation, never reuse a revoked or queued identity |
| Any stopped/draining/unresolved G | Stale event, notification or implicit registration | Unchanged | Refuse execution; history may remain readable |
| Any | Persistence/control corruption | Unresolved/error | Fail closed, retain bytes and diagnostic |

A normal work report may be successful, failed or cancelled independently of whether control is draining/stopped. `ready` is not quiescence. Unknown membership is not task success. Duplicate stop returns current/later state for the same request/generation. Concurrent resume attempts compare the same predecessor revision, allowing one winner; others receive its identity or a conflict.

### 4. Reuse cancellation, add settlement accounting

Signal the actual active turns through `turn_cancel_registry` and the existing control handle. Keep per-generation turn completion handles and owned tool-operation completion accounting in the existing runtime, not just a map entry count. Lock-free cancellation registration remains useful but its guard dropping alone is insufficient if a blocking mutation or child process survives.

Do not remove agent/member/control handles until cancellation is issued and the required execution resources are accounted. Keep a retained terminal/control projection so exact-ID status and waits remain meaningful after membership cleanup. Settlement waits are deadline-bounded and event-driven, not busy polling. Repeated inspection may observe progress after the original client times out.

Known process/background operations must retain session/generation ownership. Cancellation uses only existing safe owned-process controls. Shell daemonization, external MCP requests or remote effects may remain unprovable. Those generations stay `unresolved`, with categories and supported inspection instructions. Do not kill unknown processes, claim rollback, or silently permit another writer. Process exit does not reverse effects already committed elsewhere.

### 5. Claims release follows settlement, never membership alone

Keep existing canonical-path/claim reservation mechanisms and configured off/warn/block conflict modes. Add generation identity to lifecycle cleanup and reservation ownership. Removal cannot unregister a revoked worker into a privileged non-worker bypass. Revoked native mutations fail independently of conflict mode or coordinator membership.

A mutation already admitted when stop wins stays tracked until its actual filesystem work completes, including blocking tasks outliving the caller. Only after settlement release generation G claims. An old cleanup cannot remove G+1 claims. Unresolved shell/external mutation ownership blocks automatic reassignment even if a native file-claim conflict cannot fence that operation.

### 6. Generation-bound reports, waits and delivery

Persist accepted work-generation identity, report generation and wait target generation. `await_members` without an explicit generation snapshots the current accepted work request at invocation. If newer work is queued, that identity, not the old ready report, is the default target. Include the generation vector in wait/mutation cache identity. Do not replay an old final response just because status text still satisfies the predicate.

A caller explicitly waiting for stop may complete with settled stopped/cancelled, but not a successful task result. Historical reports remain accessible by their old generation. Late reports/notifications cannot overwrite a newer run or clear stop. Legacy reports migrate as historical/unbound unless their generation is actually established. `add-swarm-handoff-contracts` still owns revision/check/budget enrichment; shared schema edits serialize and revalidate both changes, without making that enrichment a prerequisite here.

### 7. Recovery and compatibility are conservative

Persist stop intent before cancellation acknowledgment and final settlement before releasing claims. On restart reconcile persisted owner/control first, then projections and runnable recovery. No old queued wake or crash recovery silently resumes cancelled work. A dead daemon proves its in-process turns ended, not that detached tools/remote effects settled. Keep uncertain cases unresolved. Refuse corrupt/unsupported control records without overwriting their source bytes or inventing ownership.

Additive fields use compatible defaults for reading legacy histories. New work allocates explicit generations; missing legacy fields do not fabricate fresh reports or revocation. Older clients receive existing error/nonterminal diagnostics for pending stop rather than unconditional `Done`. New structured result fields expose generation, request identity, state, outstanding categories and timestamp, not private bodies. Numeric timeout defaults can use existing action deadlines, but tests freeze their configured values and timeout always means unresolved.

## Risks / Trade-offs

- More conservative stop can block reassignment where old behavior appeared successful. Mitigation: expose outstanding categories and an event-driven reconciliation path rather than weakening the guarantee.
- Persistent owner chains/control tombstones may grow beyond member retention. Mitigation: keep compact records and explicit safe-retention rules; do not garbage-collect authority while stale requests can revive the session. No retention migration is executed by this proposal.
- Mutation futures can outlive cancellation. Mitigation: account actual owned operations before returning settled, tested with real native writes and deferred completion.
- Existing force allows explicit non-owned control. Mitigation: preserve its current authority contract, never invoke it automatically or treat discovery as permission.
- Old binaries may ignore new stop state. Mitigation: record a compatibility floor and refuse unsafe downgrade against new control state through the supported launcher/reload boundary. No silent data deletion or false downgrade safety.
- Parallel authors may touch the same schema later. Mitigation: exact-file claims and serialized edits with `add-swarm-handoff-contracts`; no fabricated dependency on unrelated work.

## Migration Plan

1. Add backward-readable control/generation data and load legacy snapshots without modifying original evidence. Bind existing ownership conservatively; ambiguous sessions remain unverified, not newly runnable.
2. Wire every admission path and generation-carrying tool context before advertising durable stop. Cover transitional mixed-version clients and persistence errors with tests. No user schema/data migration or shared runtime restart is part of authoring.
3. Enable truthful results only when all relevant admission/cancellation/settlement paths are exercised. Update `docs/SWARM_ARCHITECTURE.md`, server/runtime command docs and protocol guidance to separate task completion, stopped execution and external effects.
4. Before any later activation, use the actual built binary with isolated `JCODE_HOME`, HOME/runtime/install directories and a unique socket. Freeze binary/source hashes and exact test protocol. Do not touch shared sessions.
5. Rollback must preserve records and the stop barrier. An older binary that cannot enforce the compatibility floor cannot safely serve this state; keep the capable runtime stopped or use an explicitly isolated legacy snapshot for diagnostics. No automatic destructive downgrade or database migration.

## Verification route

Existing project routes: `cargo test -p jcode-app-core --lib` for controls/claims/recovery, `cargo test -p jcode-swarm-core`, `cargo test -p jcode-protocol`, and root `cargo test -p jcode --test e2e`. `tests/e2e/text_framing.rs` uses real daemon/bridge sockets with scripted inference; `tests/e2e/binary_integration.rs` demonstrates real spawned binary isolation. These routes are available by source/config inspection, not run in this authoring session.

Add scoped worker-control regressions to those existing surfaces, then exercise the actual selfdev binary as its own daemon over the public CLI/swarm/harness interfaces on a unique private socket. Cover idle, provider-busy and native-tool-busy stop; stop/wake races; queued stale work; cross-swarm retained owner versus unrelated denial; restart mid-stop; old wait replay; legacy clients; deferred native filesystem completion; and unresolved child/external work. Capture zero post-settlement mutations and observed generation/result identity. Scripted provider fixtures are supplemental integration evidence, not a substitute for actual binary/control acceptance. A loopback provider endpoint can supply deterministic workload input without external credentials or remote service, but remains scripted inference. Label that component synthetic; the acceptance claim concerns the actual binary, native mutations and public control path, not live-provider behavior.

The later executor must implement and document that isolated acceptance command before task 7.1 can pass; no existing test name is claimed for the new behavior. Shared debug control is disabled, so do not enable it on the shared daemon. Stop/settlement remains unverified until the isolated public acceptance succeeds. No Open Questions alter the selected contract; internal field names and fixed deadline defaults can follow existing code during implementation.
