---
execution:
  version: 1
  depends_on: [add-config-composition, workspace-aware-context]
---
# Bind issue boards to verified worlds

Roadmap owner: [P09: TUI/onboarding everyday UX](../../../docs/plans/PRODUCT_ROADMAP.md#p09-tuionboarding-everyday-ux).

## Why

The verified board audit found no native issue-board projection. The existing `todo` tool is Jcode's execution-plan data, not a tracker view (`crates/jcode-app-core/src/tool/mod.rs:534`; `crates/jcode-app-core/src/tool/todo.rs:3-9` at `dc0008591`). One authenticated Linear connection exposed both Priceless and BrownandBrown teams, so connection visibility alone cannot establish world scope (`scratch/verify-linear-kanban.md:12-16`). Existing MCP `workspaces` patterns restrict which session cwd may start a server, not which tracker records a query can return (`crates/jcode-base/src/mcp/protocol.rs:238-245,264-279` at `dc0008591`).

## What Changes

- Add an explicit tracker binding per verified world, including backend and stable team identity, with optional project narrowing. Reuse configuration composition where the existing typed config can own this setting; do not introduce a second ad hoc precedence or save model.
- Resolve world from the verified canonical repository path and applicable workspace policy. Personal and Priceless worlds may bind to Linear; Brown worlds use ADO and must never query BrownandBrown through Linear. Unknown or ambiguous worlds show no board and never inherit another world's binding.
- Add a TUI Kanban view with columns derived from the selected team's workflow states, issue cards, keyboard navigation and Escape. Truncate chrome using display-cell widths. Show distinct loading, empty, provider error and unauthorized states.
- Define status changes as a narrowly authorized operation: issue-state update only, explicit confirmation for each change, expected-prior-state check on every attempt, idempotent retry, append-only local audit log and verified-world recheck before write. V1 does not write labels, assignees or comments. Status writes are Linear-only in v1; Brown/ADO boards are read-only until ADO's conditional-update and retry contract is verified (decision 3a).

## Capabilities

### New Capabilities
- `world-issue-board`: world-bound tracker query, Kanban presentation and constrained status-change contract.

### Modified Capabilities
None.

## Impact

Expected surfaces include typed configuration and its composition, verified workspace resolution, Linear/ADO adapter boundaries, TUI routing/rendering and tests. Implementation must inventory existing tracker adapters and task/status types before selecting APIs. The audit establishes a Linear MCP namespace with read and mutation tools, but does not verify an ADO adapter or the exact write API contract. Do not treat the whole MCP tool namespace as an allowlist.

## Non-goals

No cross-world board, implicit/global tracker fallback, Brown access through Linear, writes beyond issue state, automatic tracker migration, live configuration migration, external tracker mutation during proposal authoring, or replacement of Jcode's internal todo/task view. Proposal authoring does not authorize implementation or live tracker writes.

## Unresolved decisions

1. **Binding ownership and syntax.** Recommendation: add typed world tracker entries to the existing world configuration owner after verifying its current schema, with stable backend/team/project IDs and no credential material. Alternative: a separate dedicated board config. This blocks implementation schema and migration details; either choice must preserve existing config precedence and unknown-world fail-closed behavior.
2. **Default project scope.** Recommendation: require explicit team binding and permit optional project narrowing, with team-wide board as configured behavior rather than silently selecting a project. Alternative: require a project for every binding. This blocks only default query scope; provider calls must not infer scope from visible projects.
3. **ADO and status mapping.** Evidence does not establish the available ADO transport, its query/update API, or how provider-specific workflow states map to columns. Recommendation: discover and specify the actual installed integration during implementation; show its provider workflow states directly, and fail unavailable/unauthorized rather than emulate. This blocks the ADO adapter contract and acceptance fixtures.
