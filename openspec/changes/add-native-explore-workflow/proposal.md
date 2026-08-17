## Why

Jcode can activate any installed skill as a slash command, but it lacks a native `/explore` workflow that turns an unclear request into verified context, a ranked recommendation, and a reusable handoff. Copying Claude or Codex command implementations would import harness-specific assumptions instead of using Jcode's own memory, session search, swarm, initiative, todo, side-panel, telemetry, and tool surfaces.

## What Changes

- Add a native Jcode skill named `explore`, exposed directly as `/explore [topic]` by the existing skill registry.
- Add a shared workflow-environment preflight that checks OpenSpec, Beads, and harness telemetry before workflow work begins.
- When OpenSpec or Beads is absent, ask once per repository whether Jcode should initialize it; persist the answer in repository-scoped Jcode state, never initialize silently, and continue in explicit degraded mode after a decline.
- Gather prior context from repository guidance, code and Git state, Jcode memory, session history, active initiatives, existing planning systems, and canonical Recon records.
- Use `todo` for the current exploration cycle, optional read-only `swarm` work for independent evidence domains, `initiative` for durable decision maps, and `side_panel` for live synthesis.
- Produce verified facts, assumptions, conflicts, unresolved decisions, alternatives, a ranked execution queue, one default route, and a structured handoff consumable by native `/feature`.
- Check harness telemetry availability on every invocation and emit workflow start, phase, route, completion, and degradation events when supported without making telemetry availability a success prerequisite.
- Prefer tokenless Jcode tools and structured outputs over shell commands; when shell is necessary, use bounded direct execution, batch independent probes, request JSON, and cap output at the source.

## Capabilities

### New Capabilities

- `native-explore-workflow`: Native Jcode exploration, evidence gathering, decision mapping, ranked routing, telemetry, efficient execution, and feature handoff behavior.
- `workflow-environment-preflight`: Shared one-time OpenSpec/Beads initialization prompting and per-invocation harness telemetry capability detection.

### Modified Capabilities

None.

## Preconditions

- The skill registry continues to expose a skill's `name:` as its slash command and preserve trailing prompt text.
- OpenSpec and Beads remain optional repository integrations; initialization requires explicit user approval.
- depends on: none.
- beads: unavailable in this repository at authoring time; the user was asked once for `bd init`, and proposal authoring continued without mutating Beads.

## Decisions

- **decided-by: user**: use Claude and Codex explore sequences only as design inputs, not implementations to port.
- **decided-by: user**: ask once before initializing missing OpenSpec or Beads.
- **decided-by: user**: always detect harness telemetry and aim for tokenless or token-efficient shell execution.
- **decided-by: default**: a declined initialization continues in degraded mode and is not asked again unless the user resets the repository preference.

## Impact

- New native Jcode `explore` skill and shared workflow preflight support.
- Jcode memory, session search, todo, swarm, initiative, side-panel, telemetry, and structured tool-use guidance.
- Acceptance coverage for direct `/explore` invocation, missing integrations, telemetry present/absent, shell avoidance, decision-map persistence, and `/feature` handoff.

## Done Means

- `/explore <topic>` runs natively without activating `codex-explore` or Claude compatibility paths.
- Missing OpenSpec or Beads produces one consent prompt per repository and no silent mutation.
- Exploration reports provenance and freshness and selects one defensible default route.
- The structured handoff prevents native `/feature` from repeating completed discovery.
- Telemetry and shell-efficiency behavior are observable and covered by acceptance tests.

## Testing

- Exercise public `/explore` activation and trailing prompt preservation.
- Exercise ready, missing, accepted, declined, failed, and reset integration states.
- Verify telemetry-enabled and telemetry-unavailable behavior.
- Verify native-tool preference and bounded shell/output behavior.
- Exercise explore-to-feature handoff and durable decision-map resume.
