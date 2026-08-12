## Why

Jcode lacks a native `/feature` authoring workflow that converts a clarified outcome into a decision-complete, repository-authoritative implementation contract. Claude and Codex provide useful refinement patterns, but their implementations assume specific command systems, verification scripts, and proposal ceremony that should not define Jcode's native behavior.

## What Changes

- Add a native Jcode skill named `feature`, exposed directly as `/feature [description]` by the existing skill registry.
- Consume a structured native `/explore` handoff when present, freshness-check its revisions and evidence, and avoid repeating settled discovery.
- Reuse the workflow-environment preflight from `add-native-explore-workflow` to check OpenSpec, Beads, and telemetry, ask once before missing integration initialization, and degrade truthfully after decline or failure.
- Classify uncertainties as discoverable facts, safe defaults, user-only judgments, or later evidence-dependent gates before authoring.
- Inventory affected consumers, files, interfaces, schemas, integrations, operations, tests, and edge cases; detect active-work conflicts and dependencies.
- Author through the repository's accepted authority: OpenSpec when initialized, an existing issue/planning system when explicitly authoritative, or a durable Jcode initiative plus attached design artifact in degraded mode.
- Define user-observable done means, requirement-specific verification, expected results, touched paths, dependencies, and implementation handoff.
- Check harness telemetry on every invocation and emit supported authoring phase and outcome events without requiring telemetry.
- Prefer tokenless native tools and structured CLI output; use bounded, batched, direct shell execution only where no typed surface exists.
- Require independent semantic review and deterministic repository validation before reporting a feature ready for implementation.

## Capabilities

### New Capabilities

- `native-feature-workflow`: Native Jcode feature refinement, authority selection, artifact authoring, review, telemetry, efficient execution, and implementation handoff.

### Modified Capabilities

None.

## Preconditions

- depends on: `add-native-explore-workflow` for the shared preflight and structured handoff contract.
- The repository authority selected for a feature is explicit and singular.
- beads: unavailable in this repository at authoring time; the user was asked once for `bd init`, and proposal authoring continued without mutating Beads.

## Decisions

- **decided-by: user**: use Claude and Codex sequences as context only, not implementations.
- **decided-by: user**: prompt once before initializing missing OpenSpec or Beads.
- **decided-by: user**: always check harness telemetry and optimize shell work for zero or minimal model-token cost.
- **decided-by: default**: OpenSpec is preferred when initialized; degraded initiative authoring is allowed only when the user declined or initialization failed and the output names that limitation.

## Impact

- New native Jcode `feature` skill and explore-handoff consumption.
- Integration with repository planning authority, Jcode initiative, todo, side panel, telemetry, and review primitives.
- Acceptance coverage across direct and explore-fed invocation, missing integrations, active conflicts, telemetry states, efficient execution, validation failures, and implementation handoff.

## Done Means

- `/feature` runs natively without invoking `codex-feature` or Claude compatibility commands.
- Critical uncertainty is resolved or explicitly gated before authoritative artifacts are written.
- Exactly one repository authority owns the feature contract and no duplicate task ledger is created.
- Every requirement maps to observable acceptance evidence and implementation tasks.
- Independent review and repository-native validation pass on unchanged artifact bytes before readiness is reported.

## Testing

- Exercise direct `/feature` and explore-to-feature public workflows.
- Exercise OpenSpec, alternate-authority, and degraded initiative authoring paths.
- Verify one-time setup prompting, telemetry detection, token-efficient execution, conflict handling, freshness rejection, and review invalidation after mutation.
- Run strict OpenSpec validation for this proposal and repository-native validation for generated feature artifacts.
