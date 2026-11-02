# Proposal

Roadmap owner: [P04: Swarm collaboration and handoffs](../../../docs/plans/PRODUCT_ROADMAP.md#p04-swarm-collaboration-and-handoffs). P04 permits optional high-risk arbitration only for a demonstrated gap, so apply must record that evidence before implementation.

## Why

Hard decisions (architecture, risky refactors, review disputes) benefit from several independent viewpoints argued against each other. jcode has swarm workers but no structured debate: the coordinator must hand-spawn agents, relay messages, judge agreement, and remember to stop them. Empryo ships a `council` tool (clean-room notes: `scratch/empryo-re/council-behavior.md`), but it has no cost cap, no early stop, prompt-only seat limits, and no vote tally. A native tool lets jcode enforce read-only seats and budgets in Rust.

## What Changes

- New native tool `council` with actions `open`, `round`, `say`, `log`, `close`, plus a one-shot `run` (open + N rounds + verdict + close).
- 2-4 seats, each with a distinct brief; opening statements then fixed rounds (default 1) with a per-round timeout and partial-reply tolerance.
- Early stop when every live seat's vote line agrees (jcode addition).
- Separate verdict pass returns `PASS` or `PARTIAL` with vote tally and dissent list.
- Hard caps on seats, rounds, and total model calls/cost (jcode addition), checked before spawning.
- Seats are read-only: a Rust tool allowlist, not prompt text.
- Seats always stopped on close, abort, or coordinator crash; transcript persisted under the session.
- Built on the swarm runtime in `crates/jcode-app-core/src/tool/communicate.rs` (spawn/message/await/stop), not a parallel agent system.

## Capabilities

### New Capabilities

- `council-tool`: Bounded, read-only multi-seat deliberation with verdict.

## Impact

`crates/jcode-app-core/src/tool/` (new `council.rs`, registration), swarm spawn options in `communicate.rs` (tool allowlist, lane `council`), session storage for transcripts. Depends on `return-full-swarm-reports`, `queue-messages-to-busy-workers`, `recover-swarm-workers-after-reload`. Optional: `add-role-model-slots` (`council` slot).

## Open decisions

1. Default call cap. Recommended: `seats * (rounds + 1) + 1`. Alternative: user-configured only (risk of runaway cost).
2. Interactive actions vs one-shot only. Recommended: both; `run` covers the common case, actions allow convener interjection.

## Approval

Approved 2026-11-01 with recommended defaults for all open decisions.
