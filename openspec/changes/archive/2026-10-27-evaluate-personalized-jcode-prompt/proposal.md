## Why

The personal Jcode prompt combines strong execution guidance with competing scope, publication, planning, and tool-routing instructions. Static review identifies testable hypotheses, but cannot establish whether wording changes improve agent behavior or preserve useful autonomy.

## What Changes

- Define custom-first evaluation using a frozen shipped-default base, frozen personal base, and a personal candidate containing one intended improvement. Keep all surrounding prompt layers and runtime settings matched.
- Cover assessment/completion, isolation/publication, proportional planning, policy/tool routing, and current-task continuity with challenge and counterexample fixtures.
- Start with the approved four-case scope/completion pilot: three conditions and three repetitions, totaling 36 scheduled trials. Keep transfer cases and combined-candidate evaluation separate from initial tuning.
- Judge boundary compliance, useful task outcomes, execution burden, and trial validity separately. Treat an unauthorized attempted effect as a failure even when enforcement blocks it.
- Reuse existing prompt, session, and benchmark surfaces; evaluate skill-pointer loss and prompt lifetime separately through deterministic checks.
- Preserve both live prompt files, shared policy, existing sessions, unrelated work, and the serving daemon. Paid trials and eventual prompt adoption require their own execution scope.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `workflow-behavior-evaluation`: Extend the existing controlled-evaluation contract with personal-system-prompt conditions, paired scenario families, the bounded initial pilot, and separate prompt-assembly evidence. Preserve existing workflow fixtures and acceptance requirements.

## Impact

- Planning artifacts in this change; no implementation or live evaluation in this authoring step.
- Future evaluation should reuse relevant parts of `scripts/benchmark_discovery.py`, `scripts/benchmark_discovery_rate.py`, existing prompt/session tests, and runtime context inspection. The Discovery scorers and shared-daemon benchmark assumptions do not directly fit this pilot.
- Candidate wording and evaluation evidence belong to the personal setup. No upstream default change, new provider, dependency, evaluation service, tracker mutation, publication, or automatic adoption is included.
