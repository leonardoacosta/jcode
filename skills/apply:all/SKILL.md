---
name: apply:all
description: Native Jcode explicit-queue execution workflow. Use to execute a user-selected ordered queue of approved features with dependency-safe waves, conflict serialization, per-feature apply lifecycle preservation, recovery, and partial outcome reporting.
---

# Native Apply All

Treat `/apply:all` as the queue-level form of native `/apply`. It schedules only the explicit selected queue, never every open or ready item implied by the command name. It does not activate `codex-apply-all`, `codex-apply`, or Claude-owned workflows.

## Invocation and explicit queue

- Preserve the ordered queue arguments exactly.
- Require an explicit selected queue. Missing, ambiguous, unsupported, legacy, invalid, stale, or cyclic input fails closed before mutation.
- Run the shared workflow preflight from `explore` for repository identity, setup preferences, telemetry, and degraded routes.
- Resolve every selected feature from exactly one accepted repository authority and report excluded or blocked work without broadening the queue.

## Canonical schedule and waves

Load `references/native-apply-contracts.md` when constructing, validating, resuming, reviewing, or settling a native apply-all schedule.

Compile the selected queue into the same versioned schedule and wave-plan contracts as `/apply`. For each feature record:

- Authority, revision, provenance, dependencies, conflicts, touched paths, claims, repositories, workspaces, external systems, schemas, deployments, mutable resources, verification contract, risk, lineage, idempotency scope, and extension fields.

Build dependency- and conflict-safe waves:

- Hard dependencies always precede dependents.
- Shared touched paths, claims, repositories, schemas, deployment targets, external systems, runtime resources, or other mutable resources serialize work.
- Only proven-independent features may run concurrently.
- Recompute the ready frontier from durable evidence after every terminal event.

## Per-feature lifecycle preservation

Every queued feature executes the full native `/apply` contract: preflight, implementation, verification, review, persistence, settlement, recovery, and truthful closeout. Delegated workers may implement and validate bounded independent work, but the queue coordinator retains scheduling, authority, archive, issue closure, persistence, integration gates, and final settlement.

## Partial failure policy

When a feature fails:

- Pause every transitive dependent with the failed feature as blocker.
- Continue unrelated valid branches whose schedule inputs and preconditions remain current.
- Assign retries new attempt identities linked to the original.
- Report completed, failed, blocked, paused, skipped, remaining, and newly ready work separately.

Queue-level integration gates run only after all required branches settle. A paused branch blocks dependent integration gates without invalidating truthful outcomes for completed unrelated features.

## Orchestration and runtime authority

Select direct, reviewed, light-swarm, deep-DAG, or durable-initiative execution from observable queue risk and topology. Preserve Jcode authority for selected work, approvals, schedules, checkpoints, idempotency, outcomes, and recovery. Treat Orca as runtime authority only for supported project, worktree, task, dispatch, worker, terminal, gate, and health resources it controls.

Freeze the execution path before mutation. If Orca is unavailable, use Jcode-native fallback only when the full selected queue can satisfy isolation, supervision, recovery, and verification requirements without missing runtime capabilities. Otherwise fail closed with exact missing capabilities.

## Recovery, projection, telemetry, and efficiency

- Resume from frozen schedule, feature authorities, Git state, Jcode checkpoints, Orca receipts when present, and fresh verification evidence. Do not use conversation memory as execution state.
- Prevent duplicate mutation through attempt-scoped idempotency and current artifact checks.
- Project bounded queue state, waves, frontier, active runtime owners, receipts, recovery obligations, and authorized actions in `side_panel` when useful.
- Keep terminal output compact and event-driven, with durable evidence links for details.
- Check telemetry every invocation and emit best-effort schedule, risk, orchestration, wave, review, verification, degradation, recovery, and settlement observations when supported.
- Prefer typed Jcode tools, structured output, batching, timeouts, and source-side caps. Use shell only when no typed surface exists.

## Output contract

Report the frozen schedule revision, wave plan, execution path, per-feature outcome, queue integration-gate status, validation and review evidence, persistence results, blockers, recovery obligations, and follow-up work. Never collapse a partial queue into a false binary success or failure.
