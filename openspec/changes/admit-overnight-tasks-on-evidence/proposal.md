---
execution:
  version: 1
  depends_on: []
---
Roadmap owner: [P05: Durable overnight, scheduling and recovery](../../../docs/plans/PRODUCT_ROADMAP.md#p05-durable-overnight-scheduling-and-recovery)

## Why

`start_overnight_run` always starts: the usage projection is advisory (`crates/jcode-app-core/src/overnight.rs:515`, "This is a warning only; the run starts regardless"). The supervisor loop re-prompts the coordinator every turn until wake time (`overnight.rs:266-383`) with no admission check that eligible authorized work remains, so an idle run keeps spending turns re-checking unchanged state. The roadmap gate requires waiting for a real worker/event or the report boundary instead.

## What Changes

- Record an admission decision event before each coordinator turn: `admitted` (named eligible task or new signal) or `waiting` (no eligible work).
- When `waiting`, the supervisor sleeps until a worker/background completion event, user message, or the handoff/wake boundary, instead of issuing another turn.
- Preflight records a `blocked` admission when auth is missing/expired for every configured provider, and the run does not start a coordinator turn.

## Non-scope

- Cost/token caps (see `enforce-overnight-cancellation-and-limits`).
- Recurring schedules.
- P03 semantic supervision.

## Capabilities

### New Capabilities

- `overnight-admission`

## Impact

`crates/jcode-app-core/src/overnight.rs` supervisor loop and preflight; `crates/jcode-overnight-core` event names. No new daemon.

## Resolved defaults

1. Signal source for `waiting` wake-ups. Options: (a) poll manifest/events every N seconds; (b) subscribe to bg/swarm completion channels. Selected default: (b), because polling recreates the "recheck unchanged receipts" behavior the gate forbids.
