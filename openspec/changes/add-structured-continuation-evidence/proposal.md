---
execution:
  version: 1
  depends_on: []
---
Roadmap owner: [P03: Jev semantic poke continuation](../../../docs/plans/PRODUCT_ROADMAP.md#p03-jev-semantic-poke-continuation)

## Why

Continuation decisions are recorded only as free-text log lines: `AUTO_POKE_DECISION action=idle reason=no_todos` (`crates/jcode-tui/src/tui/app/input.rs:1694`), `unchanged_ownership_assessment` (`:1727`), `unchanged_todos` (`:1830`). The deterministic `FollowUpPolicy` (`crates/jcode-todo-policy/src/followup.rs:111`) sees only a `TodoSnapshot` (`:30`), so no consumer (replay, shadow evaluator, Jev) can see which checks actually ran, whether evidence changed, or whether work is waiting or blocked. The archived `continuation-evidence` spec established an offline baseline but no structured runtime record.

## What Changes

- Emit one structured `ContinuationEvidence` record per continuation decision: decision kind (`FollowUpKind`), reason, todo counts by status, fingerprint changed flag, required-check results observed this turn (name, pass/fail/not-run), and wait/blocked markers.
- The record is redacted (no prompt or tool-output text), size-bounded, and logged next to the existing line.
- `FollowUpPolicy` behavior, caps and fingerprints are unchanged.

## Non-scope

- Any change to continuation decisions or caps.
- Jev or model inference.
- New telemetry export.

## Capabilities

### Modified Capabilities

- `continuation-evidence`: adds a structured runtime evidence record.

## Impact

`crates/jcode-todo-policy/src/followup.rs`, `crates/jcode-tui/src/tui/app/input.rs`, tests in `jcode-todo-policy` and `jcode-tui`.

## Resolved defaults

1. Storage: (a) session log JSON line; (b) separate per-session evidence file. Selected default: (a), reuses the existing log and keeps correlation with `AUTO_POKE_DECISION`.
