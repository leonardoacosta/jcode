# Proposal

Roadmap owner: [P04: Swarm collaboration and handoffs](../../../docs/plans/PRODUCT_ROADMAP.md#p04-swarm-collaboration-and-handoffs). Supporting: P05.

## Why

One server reload this session crashed/stopped 5 workers. `crates/jcode-app-core/src/server/swarm_persistence.rs` (`persist_swarm_state` :618, `load_runtime_state` :449) restores swarm membership, and `crates/jcode-app-core/src/server/reload_recovery.rs` tracks per-session recovery, but lane and initial prompt are not enough to resume a worker, so it is lost.

## What Changes

- Persist each worker's lane, prompt, model and last status with swarm state.
- On reload: resume workers whose session restores; otherwise mark `orphaned`.
- `swarm retry <worker>` respawns an orphan with persisted lane+prompt in one step.

## Capabilities

### Modified Capabilities

- `swarm-state-persistence`: worker recovery across reload.

## Impact

`crates/jcode-app-core/src/server/swarm_persistence.rs`, `crates/jcode-app-core/src/server/reload_recovery.rs`, `crates/jcode-app-core/src/tool/communicate.rs` (retry). Persisted state schema gains fields (backward-compatible defaults).

## Open decisions

1. Auto-respawn orphans? Recommended: no, mark orphaned and offer retry (avoids duplicate work). Alternative: auto-respawn.

## Approval

Approved 2026-11-01 with recommended defaults for all open decisions.
