---
execution:
  version: 1
  depends_on: [return-full-swarm-reports]
---
Roadmap owner: [P04: Swarm collaboration and handoffs](../../../docs/plans/PRODUCT_ROADMAP.md#p04-swarm-collaboration-and-handoffs)

## Why

Plan items carry `file_scope`, `blocked_by` and `assigned_to` (`crates/jcode-plan/src/lib.rs:18-31`) but no base revision, acceptance check or budget, so an assignment is free-text prompt and a report is not tied to the revision it started from or the checks it ran. Active changes cover report delivery (`return-full-swarm-reports`), recovery (`recover-swarm-workers-after-reload`), claims (`add-swarm-file-claims`) and block/stall display (`surface-worker-stall-and-block-reasons`), not the assignment and report contract itself.

## What Changes

- Add optional `base_rev`, `acceptance_check` (command string) and `turn_budget` to plan items; spawn/assign records current HEAD as `base_rev` if unset.
- Worker reports gain structured `base_rev`, `head_rev`, changed files, and each acceptance check with exit status.
- The coordinator view warns when changed files fall outside `file_scope` or the acceptance result is missing.
- Exceeding `turn_budget` marks the member `blocked` with reason `budget`.

## Non-scope

- Running checks on the worker's behalf.
- Reload recovery, file claims, stall display (covered by the changes above).
- Merge automation.

## Capabilities

### New Capabilities

- `swarm-handoff-contracts`: bounded assignment and evidence-bearing reports.

## Impact

`crates/jcode-plan/src/lib.rs`, `crates/jcode-app-core/src/server/` swarm handlers, swarm tool schema; tests in `jcode-app-core`.

## Resolved defaults

1. Out-of-scope edits: (a) warn; (b) reject the report. Selected default: (a), avoids losing work; enforcement belongs to `add-swarm-file-claims`.
