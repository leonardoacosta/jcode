## Why

Jcode now has a durable Command Center model and Orca has a rich runtime orchestration model, but the policy bridge between them is fragmented across generic skills and an incomplete adapter. Without a single authority and projection contract, Command Center can select the wrong ownership mode, confuse Orca runtime IDs with canonical project IDs, or settle durable state without sufficient runtime evidence.

## What Changes

- Establish a layered projection bridge in which Jcode owns durable intent, scheduling, permissions, idempotency, and outcome settlement while Orca owns canonical executable identity and live runtime truth.
- Define explicit selection rules for full handoff, supervised Run/Task/Dispatch, direct terminal action, observation-only projection, and decision gates.
- Keep `orca-cli` focused on version-matched runtime mechanics and keep `orchestration` focused on generic supervised coordination.
- Remove obsolete `llmtrim` guidance from the generic orchestration skill.
- Add a focused `jcode-command-center-orchestration` skill for Command Center policy, identifier preservation, lifecycle projection, scheduling correlation, safe mutation boundaries, and acceptance evidence.
- Require unsupported or unavailable Orca capabilities to fail closed rather than inventing CLI calls or silently downgrading orchestration modes.
- Correct the Command Center integration contract so Orca runtime IDs can never substitute for canonical repository or project IDs.
- Add representative skill evaluations and deterministic checks for routing, authority, identifier handling, degraded states, and lifecycle settlement.

## Capabilities

### New Capabilities

- `command-center-orca-orchestration`: Defines how Command Center selects, launches, observes, and settles Orca-backed execution while preserving Jcode and Orca authority boundaries.

### Modified Capabilities

None.

## Impact

- Installed skill sources for `orca-cli`, `orchestration`, and the new `jcode-command-center-orchestration` skill.
- Command Center documentation and the approved mobile architecture brief under `docs/`.
- Command Center Orca adapter behavior in `crates/jcode-app-core` where canonical project identity is projected.
- OpenSpec initiative state for the broader Command Center orchestration program.
- Skill evaluation fixtures and verification scripts used to prove routing and policy behavior.

## Preconditions

- base-commit: jcode@e832aa39c7e79fcbf5df77396c67c1cf66e7c414
- The canonical Orca skill sources remain `/home/nyaptor/dev/agents/skills` and project into `~/.agents/skills` through the repository reconciliation scripts.
- The installed Orca runtime exposes version-matched `orca-cli` and `orchestration` guides.
- The linked OpenSpec initiative `command-center-orchestration` exists in context store `jcode`.

## Decisions

- Jcode owns durable intent and settlement; Orca owns canonical executable identity and runtime truth.
- Every action selects one explicit orchestration pattern and never silently downgrades.
- Generic Orca skills remain generic; Jcode policy belongs in a focused Command Center orchestration skill.
- Unsupported runtime mutations fail closed, and Orca runtime ID never substitutes for project ID.

## Done Means

- Canonical and installed skill projections agree, contain the focused policy skill, and contain no `llmtrim` guidance.
- Representative skill evaluations prove the routing, authority, identifier, scheduling, degraded-state, and cleanup contracts.
- Focused app-core tests prove canonical project identity is not sourced from runtime ID and unsupported mutations remain fail closed.
- OpenSpec strict validation, feature artifact verification, and path-scoped repository checks pass.

## Testing

- Run canonical skill projection reconciliation and verification in `/home/nyaptor/dev/agents` and expect zero projection differences.
- Run the paired skill evaluation suite and expect every required policy assertion to pass without a baseline regression.
- Run focused `jcode-app-core` and `jcode-command-center` checks and tests and expect zero failures.
- Run OpenSpec strict validation and the Codex feature verifier and expect every required row to report `PASS`.
