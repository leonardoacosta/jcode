# Proposal

Roadmap owner: [P04: Swarm collaboration and handoffs](../../../docs/plans/PRODUCT_ROADMAP.md#p04-swarm-collaboration-and-handoffs).

## Why

DMs and summary requests to a worker mid-turn fail with "is busy" (`crates/jcode-app-core/src/server/jade_relay.rs:1214`); 456 such log lines this session. Coordinators retry or give up, losing instructions.

## What Changes

- Messages to a busy worker are queued per session (FIFO) instead of rejected.
- Queue drains at the worker's turn end, injected as next input; sender gets `queued` ack.
- `interrupt` delivery mode keeps current behavior.

## Capabilities

### Modified Capabilities

- `swarm-communicate-reliability`: busy-target message delivery.

## Impact

`crates/jcode-app-core/src/server/jade_relay.rs`, comm message routing, agent turn-end hook.

## Open decisions

1. Queue bound. Recommended: 32 messages per worker, oldest dropped with sender notice. Alternative: unbounded (memory risk).

## Approval

Approved 2026-11-01 with recommended defaults for all open decisions.
