## Context

Portable `apply` defines repository-neutral lifecycle invariants, and the Codex queue flow contributes useful dependency, conflict, wave, and reconstruction concepts. They are source material rather than command implementations. Native Jcode owns the public commands and composes existing scheduling, swarm, initiative, side-panel, telemetry, and Command Center primitives. Orca participates only through verified capabilities and remains runtime authority for the resources it owns.

## Goals / Non-Goals

**Goals:**

- Provide distinct native `/apply` and `/apply:all` intent boundaries.
- Share one canonical scheduling and lifecycle engine.
- Select orchestration from risk and topology rather than command name alone.
- Preserve Jcode and Orca authority boundaries and fail closed on missing capabilities.
- Make partial progress, recovery, verification, review, and user-visible state durable and truthful.
- Keep execution token-efficient through typed tools and bounded structured output.

**Non-Goals:**

- Port Codex or Claude command implementations.
- Make `/apply:all` mean every open or ready feature.
- Require Orca for work that a declared Jcode-native path can safely execute.
- Route policy from telemetry availability.
- Treat agent completion messages or browser state as durable settlement evidence.

## Decisions

### 1. Separate command surfaces share one engine

`/apply` accepts one approved feature. `/apply:all` accepts an explicit ordered queue. Both compile into the same versioned schedule and wave-plan contracts and reuse preflight, conflict, orchestration, verification, recovery, and settlement logic. Separate commands preserve authorization, discoverability, telemetry, and policy boundaries without duplicating implementation.

### 2. Reject implicit queue broadening

`/apply:all` never derives authority to execute all open work from its name. It resolves only the user-selected queue, reports exclusions and blockers, and requires a fresh canonical schedule revision before mutation. Missing, stale, unsupported, ambiguous, legacy, or cyclic input fails closed.

### 3. Build dependency- and conflict-safe waves

Hard dependencies, touched paths, claims, repositories, workspaces, external systems, schemas, deployments, and mutable resources form graph edges or serialization constraints. Only proven-independent nodes share a wave. Every feature remains a composite lifecycle containing implementation, verification, review, persistence, and settlement.

### 4. Continue safe branches after failure

A failed attempt blocks and pauses every transitive dependent. Unrelated nodes continue when their schedule inputs and preconditions remain current. The ready frontier is recomputed after every terminal event. Queue settlement reports completed, failed, blocked, paused, skipped, and remaining work rather than collapsing the queue to a false binary result.

### 5. Select orchestration from observable risk and topology

The scheduler evaluates package and repository count, dependency depth, mutable-resource conflicts, migration/security/deployment/destructive risk, verification cost, duration, interruption likelihood, provider-diversity requirements, and live runtime capabilities.

- Low risk may execute directly with deterministic checks.
- Normal risk uses an independent reviewer when warranted, which may use the same provider.
- High risk requires a different provider family for adversarial review plus separate acceptance verification.
- Critical risk requires multiple independent frontier reviewers, cross-provider coverage, acceptance verification, and a human gate.
- Light swarm is used for independent work and review dimensions.
- Deep task DAG is used for dependency-heavy features and nontrivial queues.
- Durable initiatives wrap long-running, scheduled, recoverable, or multi-worktree execution.

Users may increase rigor. Lowering rigor below policy requires an explicit authorized gate.

### 6. Preserve Jcode and Orca authority

Jcode owns initiatives, selected work, approvals, schedules, permissions, idempotency, checkpoints, rollback intent, and durable outcomes. Orca owns supported Project and Repository identity, host setup, worktrees, Runs, Tasks, Dispatches, workers, terminals, gates, and runtime health. Runtime observations are evidence, not settlement authority.

Before execution, Jcode records the selected ownership pattern, identity envelope, preconditions, idempotency scope, expected receipts, cleanup obligations, and unavailable-capability behavior. Unsupported launch, retry, cancel, or supervision fails closed.

### 7. Allow explicit capability-based fallback

If Orca is unavailable, `/apply` may use a declared Jcode-native execution path when all required isolation, supervision, validation, and recovery needs are satisfied. `/apply:all` may do so only when the selected queue can safely run without unavailable multi-worktree or durable runtime features. The selected path is frozen during preflight and surfaced to the user; no run silently downgrades after mutation begins.

### 8. Recover from durable evidence

Resume reconstructs current state from authoritative proposals and issues, frozen schedule revision, Git commits and worktrees, Jcode initiative checkpoints, Orca Task and Dispatch receipts, and fresh verification results. Conversation memory is non-authoritative. A retry receives a new attempt or Dispatch ID linked to the original. Completed features are not rerun unless their inputs or integration assumptions became stale.

### 9. Bind review and verification to immutable inputs

Reviewers receive requirements, final diff, and verification contract independently rather than the implementer's conclusions. Findings include evidence, severity, affected requirement, and a reproducible check. Disagreement goes to a synthesis gate, not majority voting. Artifact or diff mutation invalidates affected review and verification receipts.

### 10. Project bounded execution state

The side pane shows selected execution mode and rationale, current wave and ready frontier, active provider/model and runtime owner, feature states, receipts, missing capabilities, and recovery obligations. It exposes only authorized pause, approve, retry, inspect, cancel, and resume actions. Terminal output is compact and event-driven; complete evidence remains in durable records.

### 11. Check telemetry and optimize token cost

Every invocation detects harness telemetry and emits supported command, schedule, risk, orchestration, wave, review, verification, degradation, recovery, and settlement events. Telemetry absence never weakens correctness. Typed Jcode tools, structured output, batching, timeouts, and source-side caps are preferred over shell transcript ingestion.

## Risks / Trade-offs

- **[Risk] Separate commands drift** → compile both through one canonical scheduling engine and shared conformance tests.
- **[Risk] Automatic orchestration surprises users** → surface the selected level and rationale before mutation.
- **[Risk] Partial queue progress creates integration ambiguity** → require per-feature settlement and defer queue integration gates until required branches settle.
- **[Risk] Cross-provider review becomes expensive** → reserve it for high and critical risk.
- **[Risk] Jcode-native fallback weakens supervision** → freeze the execution path in preflight and reject unmet capabilities.
- **[Risk] Resume duplicates mutation** → reconstruct from durable receipts and use attempt-scoped idempotency.
- **[Risk] Side-pane event volume becomes unbounded** → retain summarized projections and stable links to complete evidence.

## Migration Plan

1. Define versioned schedule, wave-plan, risk, receipt, and execution-path contracts.
2. Add native `apply` single-feature intake and lifecycle execution.
3. Add native `apply:all` explicit queue intake, graph construction, wave scheduling, and partial-progress semantics.
4. Add orchestration selection, provider policy, Jcode/Orca authority adapters, and capability-based fallback.
5. Add durable recovery, idempotency, review, verification, side-pane projection, and telemetry.
6. Add public acceptance workflows and failure-boundary tests.
7. Roll back by disabling the native skills; authoritative feature and run evidence remain readable.

## Open Questions

None.
