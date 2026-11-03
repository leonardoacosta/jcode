---
execution:
  version: 1
  depends_on: []
---
# Proposal

Roadmap owner: [P04: Swarm collaboration and handoffs](../../../docs/plans/PRODUCT_ROADMAP.md#p04-swarm-collaboration-and-handoffs). Supporting: P09.

## Why

Visible workers launched in Herdr get a pane (`crates/jcode-terminal-launch/src/lib.rs:662`), but the pane ID is not recorded on the swarm member, so the user cannot jump from a worker row to its pane. Herdr crash restore uses `resume_argv_for_session` (`crates/jcode-tui/src/tui/herdr.rs:369`) only for the client's own session, so worker panes are not restored as Jcode sessions.

## What Changes

- Record the Herdr pane ID on the swarm member at spawn and persist it with swarm state.
- Add a focus-pane action to `/agents` and the worker widget row that runs `herdr pane focus <id>`; hide it when no pane is known.
- Each worker pane reports its own session resume argv so Herdr can restore it.

## Capabilities

### New Capabilities

- `herdr-worker-panes`: worker pane identity, focus and restore through Herdr.

## Impact

`jcode-terminal-launch`, swarm member state and persistence, `/agents` view, `herdr.rs`. Depends on `recover-swarm-workers-after-reload` for persisted member fields.

## Resolved defaults

1. Provide the `/agents` focus action first, with a widget keybind deferred until later.

## Evidence

Survey: [plans/2026-11-01-background-agent-ux-gaps.md](../../../plans/2026-11-01-background-agent-ux-gaps.md) gaps 4, 10.
