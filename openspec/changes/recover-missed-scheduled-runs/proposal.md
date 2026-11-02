---
execution:
  version: 1
  depends_on: []
---
Roadmap owner: [P05: Durable overnight, scheduling and recovery](../../../docs/plans/PRODUCT_ROADMAP.md#p05-durable-overnight-scheduling-and-recovery)

## Why

`ScheduleQueue::pop_ready` and `take_ready_direct_items` remove ready items and save the queue before delivery (`crates/jcode-app-core/src/ambient/persistence.rs:94-146`). A crash between save and delivery loses the item with no record. Items overdue at startup are delivered silently with no indication they were missed, and `schedule list` shows only pending items, so a user cannot inspect what ran, was missed or was lost.

## What Changes

- Mark items `dispatched` (not removed) on pop; remove only after delivery is acknowledged.
- At startup, items `dispatched` without acknowledgement become `undelivered` and are redelivered once with a `recovered` note; items overdue by more than a grace window are delivered with a `missed by <duration>` note.
- `schedule list` includes the last N terminal items with status `delivered|missed|recovered|cancelled`.

## Non-scope

- Recurring schedules (conditional on a concrete recurring-work requirement).
- Overlap policy for recurring runs.

## Capabilities

### New Capabilities

- `scheduled-run-recovery`

## Impact

`ambient/persistence.rs` queue item state, ambient manager delivery, `schedule` tool list output.

## Pending user decision

1. Redelivery of unacknowledged items. Options: (a) redeliver once; (b) mark only. Recommended: (a) with a visible `recovered` marker, because one-shot tasks are otherwise lost; at-most-once retry avoids loops.

The recommendation above is not a selected default. The dependent behavior remains blocked until this choice is settled; this proposal does not authorize it.
