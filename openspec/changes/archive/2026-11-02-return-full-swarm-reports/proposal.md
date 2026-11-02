---
execution:
  version: 1
  depends_on: []
---
# Proposal

Roadmap owner: [P04: Swarm collaboration and handoffs](../../../docs/plans/PRODUCT_ROADMAP.md#p04-swarm-collaboration-and-handoffs).

## Why

Coordinators lose worker results. `summary` (`crates/jcode-app-core/src/tool/communicate.rs:2706`) renders via `format_tool_summary` (`:1484`), which lists tool calls only, not the worker's final report. Report text shown to the coordinator is cut to 60 chars per line (`:2505`). `await_members` (`:3089`) treats `ready` as done even without a report, and a worker that reported then stopped is not counted done, producing late "timed out" wake-ups.

## What Changes

- `summary` and `await_members` return the worker's full final report body (no per-line truncation), plus tool-call list after it.
- A member in `ready` without a submitted report is reported `incomplete`, not done.
- A member that submitted a report and later stopped counts as done; no timeout wake-up for it.

## Capabilities

### Modified Capabilities

- `swarm-communicate-reliability`: report delivery and await completion semantics.

## Impact

`crates/jcode-app-core/src/tool/communicate.rs`, comm await handling in `crates/jcode-app-core/src/server/`. Output format of `summary`/`await_members` grows.

## Open decisions

1. Cap on report size. Recommended: no truncation below 16 KB, then tail-truncate with marker. Alternative: no cap (context bloat risk).

## Approval

Approved 2026-11-01 with recommended defaults for all open decisions.
