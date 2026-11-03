---
execution:
  version: 1
  depends_on: []
---
# Proposal

Roadmap owner: [P04: Swarm collaboration and handoffs](../../../docs/plans/PRODUCT_ROADMAP.md#p04-swarm-collaboration-and-handoffs). Supporting: P09.

## Why

Herdr can show `idle`, `working`, `blocked` and `done`, but Jcode reports only two states: `AgentState { Idle, Working }` in `crates/jcode-tui/src/tui/herdr.rs:31`. The only caller of `herdr::sync` is the client TUI (`crates/jcode-tui/src/tui/app/terminal_title.rs:76`), so headless swarm workers and background tasks never reach Herdr. A session waiting on approval or a stalled worker looks the same as one that is working, and a finished turn looks idle, so the user has no signal to switch panes.

## What Changes

- Add `Blocked` (approval or ask-user prompt pending, or any owned worker blocked or stalled) and `Done` (turn or owned work finished and not yet seen) to `AgentState`.
- The coordinator session reports an aggregate state that includes its headless workers and background tasks: blocked > working > done > idle.
- `Done` clears to `idle` when the session's client takes input focus.
- When a worker or background task completes or fails while the client is unfocused, also emit a terminal notification (OSC 9 or bell), off by default via config.

## Capabilities

### New Capabilities

- `herdr-agent-lifecycle`: Jcode lifecycle states reported to Herdr.

## Impact

`crates/jcode-tui/src/tui/herdr.rs`, `crates/jcode-tui/src/tui/app/terminal_title.rs`, swarm member and background task status sources read by the info widget. No Herdr changes; uses its existing state set.

## Resolved defaults

1. Aggregate worker state into the coordinator's pane, since headless workers have no pane.
2. Terminal notifications default to off.

## Evidence

Survey: [plans/2026-11-01-background-agent-ux-gaps.md](../../../plans/2026-11-01-background-agent-ux-gaps.md) gaps 1, 2, 5.
