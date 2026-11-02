# Proposal

Roadmap owner: [P04: Swarm collaboration and handoffs](../../../docs/plans/PRODUCT_ROADMAP.md#p04-swarm-collaboration-and-handoffs). Supporting: P09.

## Why

`crates/jcode-app-core/src/server/swarm_watchdog.rs` detects stalled workers, but only `swarm list` and `swarm status` show it; the info widget (`crates/jcode-tui/src/tui/info_widget_swarm_background.rs`) renders `blocked` without a reason and never shows `stalled`. Status strings are matched ad hoc in the widget. Finished items have no unseen marker, so completions are easy to miss.

## What Changes

- Share one lifecycle enum (`spawned`, `running`, `waiting`, `blocked`, `stalled`, `done`, `failed`) between swarm members and background tasks; the widget renders from it.
- Show `stalled Nm` and a short blocked reason (awaiting approval, waiting on dependency, waiting on network) with age, within existing display-cell truncation.
- Mark done/failed rows unseen until the widget or `/agents` view is shown; show an unseen count in the widget header.
- On a stall, show a coordinator toast with nudge / stop / open-pane actions instead of relying on the opt-in silent nudge.

## Capabilities

### Modified Capabilities

- `swarm-widget-layout`: lifecycle rows show stall and block reasons and unseen completions.

## Impact

`swarm_watchdog.rs`, swarm member status types, background task status, `info_widget_swarm_background.rs`. Persisted status strings keep backward-compatible parsing.

## Resolved defaults

1. Stall actions use an inline toast with keys, with `/agents` as the fallback.

## Evidence

Survey: [plans/2026-11-01-background-agent-ux-gaps.md](../../../plans/2026-11-01-background-agent-ux-gaps.md) gaps 3, 7, 8, 9, 11.
