---
execution:
  version: 1
  depends_on: [make-worker-stop-durable]
---
Roadmap owner: [P05: Durable overnight, scheduling and recovery](../../../docs/plans/PRODUCT_ROADMAP.md#p05-durable-overnight-scheduling-and-recovery)

## Why

`ScheduleQueue::pop_ready` and `take_ready_direct_items` remove ready items and save the queue before delivery (`crates/jcode-app-core/src/ambient/persistence.rs:94-146`). A crash between save and delivery loses the item with no record. Items overdue at startup are delivered silently with no indication they were missed, and `schedule list` shows only pending items, so a user cannot inspect what ran, was missed or was lost.

## What Changes

- Mark items `dispatched` (not removed) on pop; remove only after delivery is acknowledged. Delivery acknowledgment means the intended target accepted the generation-bound message, not that the work executed successfully. Recheck target cancellation/revocation before delivery and suppress stale wakes.
- At startup, items `dispatched` without acknowledgement become `uncertain` and require explicit manual retry; overdue items are marked `missed` with their delay. Neither state automatically triggers delivery.
- `schedule list` includes the last N terminal items with status `delivered|missed|uncertain|cancelled`.

## Non-scope

- Recurring schedules (conditional on a concrete recurring-work requirement).
- Overlap policy for recurring runs.

## Capabilities

### New Capabilities

- `scheduled-run-recovery`

## Impact

`ambient/persistence.rs` queue item state, ambient manager delivery, `schedule` tool list output.

## Resolved defaults

Unacknowledged deliveries are marked uncertain and require explicit manual retry with a duplicate-effect warning: missing acknowledgment does not establish that the first attempt did not run. Record a new attempt and preserve prior history. Never automatically redeliver an item merely because its acknowledgment is missing; the first delivery may already have executed.
