## Why

Jcode needs native `/apply` and `/apply:all` workflows that execute approved feature contracts without inheriting Codex- or Claude-specific command ownership. Portable apply guidance contributes useful lifecycle invariants, while Jcode already has typed tools, todo, initiatives, side panels, swarms, task DAGs, schedules, telemetry, and durable Command Center evidence. These should become one native execution system with distinct single-feature and explicit-queue intent boundaries.

## What Changes

- Add native Jcode skills named `apply` and `apply:all`, exposed directly by the skill registry with preserved trailing arguments.
- Back both commands with one canonical scheduling engine and shared schedule and wave-plan contracts.
- Make `/apply` execute one approved feature and `/apply:all` execute only an explicitly selected ordered queue, never every open item implicitly.
- Resolve authoritative proposal, issue, repository, dependency, conflict, mutable-resource, verification, telemetry, and runtime-capability state before mutation.
- Select direct, reviewed single-agent, light-swarm, deep-DAG, or durable initiative orchestration from observable risk and topology.
- Keep Jcode authoritative for intent, approvals, schedules, idempotency, checkpoints, and outcomes; keep Orca authoritative for supported projects, worktrees, tasks, dispatches, workers, terminals, and runtime health.
- Permit a declared Jcode-native execution path when it satisfies the approved work and Orca is unavailable; never silently downgrade an Orca-dependent run.
- Continue independent queue branches after a feature failure while pausing all transitive dependents with exact blockers.
- Require cross-provider review for high-risk and critical work, not normal-risk work by default.
- Reconstruct interrupted work from durable evidence rather than conversation memory and assign a new attempt identity to every retry.
- Project compact execution state, evidence, recovery obligations, and actions into the side pane while keeping terminal output bounded and event-driven.
- Check harness telemetry on every invocation and prefer tokenless native tools, structured output, bounded batching, and source-side output caps.

## Capabilities

### New Capabilities

- `native-apply-workflow`: Native single-feature execution, verification, review, recovery, and truthful closeout.
- `native-apply-all-workflow`: Explicit queue selection, dependency-aware waves, partial-progress failure handling, integration gates, and queue settlement.
- `apply-scheduling-core`: Shared canonical schedule, orchestration selection, authority boundary, risk policy, telemetry, evidence, recovery, and user-visible projection.

### Modified Capabilities

None.

## Preconditions

- depends on: `add-native-feature-workflow` for implementation-ready authoritative feature contracts.
- depends on: `add-native-explore-workflow` for shared repository integration consent and telemetry preflight.
- The selected feature or queue is approved, current, and executable under exactly one repository authority.
- beads: unavailable in this repository at authoring time; the user was previously asked once for initialization and proposal authoring does not mutate Beads.

## Decisions

- **decided-by: user**: expose distinct `/apply` and `/apply:all` commands backed by one shared engine.
- **decided-by: user**: `/apply:all` pauses failed-feature dependents but continues unrelated valid branches.
- **decided-by: user**: require cross-provider review only for high-risk and critical work.
- **decided-by: user**: allow explicit capability-based Jcode-native fallback when Orca is unavailable; never silently downgrade.
- **decided-by: user**: recover from durable evidence and show compact execution and recovery state in the side pane.
- **decided-by: prior workflow policy**: ask once before initializing missing OpenSpec or Beads, always check harness telemetry, and minimize shell-token cost.

## Impact

- New native Jcode `apply` and `apply:all` skills.
- Shared scheduling and wave-plan validation used by both command surfaces.
- Integration with repository authority, todo, initiatives, side panel, swarm task graphs, telemetry, Command Center, and capability-gated Orca execution.
- Acceptance coverage across single and queued execution, risk tiers, conflicts, partial failures, retries, interruption recovery, unavailable runtime capabilities, telemetry degradation, and bounded output.

## Done Means

- `/apply` and `/apply:all` activate native Jcode workflows without invoking Codex- or Claude-owned implementations.
- Both commands reject stale, invalid, ambiguous, unsupported, or legacy scheduling inputs before mutation.
- `/apply:all` never broadens an explicit queue and never runs dependent or conflicting work concurrently.
- Every feature receives its complete implementation, verification, review, persistence, and settlement contract.
- High-risk work receives independent cross-provider review; lower risk tiers do not pay that cost by default.
- Interrupted runs resume from durable evidence without duplicate mutation or false completion.
- Side-pane and terminal projections remain bounded, correlated, and truthful.

## Testing

- Exercise native slash activation and trailing argument preservation for both commands.
- Exercise one-feature execution and explicit queue selection through public Jcode interfaces.
- Exercise dependency cycles, shared paths, mutable resources, stale schedules, invalid queues, and unsupported legacy inputs.
- Exercise feature failure with paused dependents and continued unrelated branches.
- Exercise all risk tiers, including high-risk cross-provider review and critical human approval.
- Exercise Orca-supervised, declared Jcode-native, missing-capability, interruption, retry, resume, and cleanup paths.
- Exercise telemetry available and unavailable states plus bounded terminal and side-pane projections.
- Run focused repository tests and strict OpenSpec validation.
