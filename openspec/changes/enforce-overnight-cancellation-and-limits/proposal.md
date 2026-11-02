---
execution:
  version: 1
  depends_on: []
---
Roadmap owner: [P05: Durable overnight, scheduling and recovery](../../../docs/plans/PRODUCT_ROADMAP.md#p05-durable-overnight-scheduling-and-recovery)

## Why

Cancellation is only checked between coordinator turns: `run_supervisor` reads `CancelRequested` before (`overnight.rs:268`) and after (`overnight.rs:318`) a turn, while `run_turn_monitored` selects only on the turn future and sampling timers (`overnight.rs:405-430`), so an active long turn runs to completion after the user stops it. Concurrency is guidance only (`max_agents_guidance: 2`, `overnight.rs:116`; `jcode-overnight-core/src/lib.rs:70`), and there are no time/token caps enforced.

## What Changes

- Add a cancellation branch to `run_turn_monitored` that aborts the in-flight turn within one sample interval of `CancelRequested`, cancels child bg tasks and swarm workers started by the run, and records `run_cancelled` with what was stopped.
- Enforce `max_agents` as a hard cap: worker spawns beyond it from the run's coordinator are refused with a visible reason.
- Add optional `max_turn_minutes` and `max_run_tokens`; exceeding either ends the turn/run with a recorded `limit_reached` event.
- User stop takes precedence over any in-progress wrap-up prompt.

## Non-scope

- Monetary cost caps (needs P06 reconciliation first).
- Admission/waiting (separate change).

## Capabilities

### New Capabilities

- `overnight-limits`

## Impact

`overnight.rs` turn monitor and supervisor; swarm spawn path for run-owned coordinators; overnight manifest fields.

## Open decisions

1. Token cap source. Options: (a) provider-reported usage per turn; (b) local estimate. Recommended: (a) with (b) only when the provider reports none, labelled `estimated`, because P06 requires distinguishing enforcement from estimates.
