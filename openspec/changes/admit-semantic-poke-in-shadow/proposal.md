---
execution:
  version: 1
  depends_on: [add-structured-continuation-evidence, route-jev-supervision-explicitly]
---
Roadmap owner: [P03: Jev semantic poke continuation](../../../docs/plans/PRODUCT_ROADMAP.md#p03-jev-semantic-poke-continuation)

Selected outcome: P03.O3, both `semantic-poke-admission` requirements and all scenarios, tasks 1–6 in file order. Depends on accepted P03.O1 evidence and P03.O2 routing. Shadow observation only, never production semantic decisions.

## Why

Continuation is driven by open todo counts and fingerprints in `FollowUpPolicy` (`crates/jcode-todo-policy/src/followup.rs:111`, kinds at `:63`), so it can poke on unchanged waits or stop after a persuasive final answer. The roadmap requires shadow evaluation before any semantic decision is enabled; `JevClient::evaluate` already special-cases supervision (`crates/jcode-base/src/jev.rs:358-359`).

## What Changes

- Behind an opt-in flag, send each structured continuation evidence record to Jev supervision and log its verdict (`continue`/`stop`/`uncertain`) beside the deterministic decision.
- Shadow only: the deterministic decision is always the one executed.
- An offline replay compares verdicts with recorded decisions and reports missed required checks, hypothetical redundant turns avoided, call count, latency and uncertain count separately.

## Non-scope

- Enabling semantic decisions live.
- Any savings claim.
- Granting Jev tools or authority.

## Capabilities

### New Capabilities

- `semantic-poke-admission`: shadow semantic continuation verdicts and replay comparison.

## Impact

`crates/jcode-tui/src/tui/app/input.rs`, `crates/jcode-todo-policy`, a replay entry point; tests.

## Resolved defaults

1. Gate for later live enablement: (a) zero missed required checks plus fewer redundant turns on matched replay; (b) disagreement rate only. Selected default: (a), matches the roadmap gate.
