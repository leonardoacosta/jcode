---
execution:
  version: 1
  depends_on: [make-worker-stop-durable]
---
# Proposal

Roadmap owner: [P04: Swarm collaboration and handoffs](../../../docs/plans/PRODUCT_ROADMAP.md#p04-swarm-collaboration-and-handoffs). Supporting: P09.

## Why

`crates/jcode-app-core/src/server/swarm_watchdog.rs` detects stalled workers, but only `swarm list` and `swarm status` show it; the info widget (`crates/jcode-tui/src/tui/info_widget_swarm_background.rs`) renders `blocked` without a reason and never shows `stalled`. Status strings are matched ad hoc in the widget. Finished items have no unseen marker, so completions are easy to miss.

## What Changes

- Derive a shared presentation projection from existing swarm/background types and durable control state without replacing those types. Preserve stopped, crashed, cancelled, draining and unresolved meaning; never collapse them into successful done. Display state is not execution authority.
- Show `stalled Nm` and a short blocked reason (awaiting approval, waiting on dependency, waiting on network) with age, within existing display-cell truncation.
- Track a completion-event ID and per-consumer seen cursor. Viewing the widget or `/agents` marks only that consumer's completion seen; it does not change execution state or implicitly clear Herdr's cursor.
- On a stall, show only supported authorized actions. Stop consumes the durable control contract; open-pane is offered only with verified pane identity from focus-worker-herdr-pane. Its absence does not block stall visibility. Nudge never revives revoked work.

## Capabilities

### Modified Capabilities

- `swarm-widget-layout`: lifecycle rows show stall and block reasons and unseen completions.

## Impact

`swarm_watchdog.rs`, swarm member status types, background task status, `info_widget_swarm_background.rs`. Persisted status strings keep backward-compatible parsing.

## Upstream v1.0.1 reconciliation

The approved integration adopts the upstream dedicated swarm page and report coalescing rather than adding a competing inline swarm strip. This change retains truthful stall/block/completion projection and consumer-specific seen state on the upstream page. Preserve full report and report-completion behavior. The `make-worker-stop-durable` dependency and existing stop/owner safeguards remain unchanged; its acceptance is unfinished and is neither expanded nor marked complete here. Do not use status presentation or coalescing to weaken stop, owner, cancellation, or completion semantics.

## Resolved defaults

1. Stall actions use the upstream swarm page and its supported controls; do not add a fork-only inline strip. Keep `/agents` as the fallback entry point where available.

## Evidence

Survey: [plans/2026-11-01-background-agent-ux-gaps.md](../../../plans/2026-11-01-background-agent-ux-gaps.md) gaps 3, 7, 8, 9, 11.
