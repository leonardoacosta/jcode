# Proposal

## Why

Swarm state persistence runs blocking work on Tokio executor threads. `persist_swarm_state_for` (`crates/jcode-app-core/src/server.rs:359-372`) awaits the per-swarm operation lock, then calls the synchronous `persist_swarm_state` (`swarm_persistence.rs:619`), which takes a `std::sync::Mutex` file lock and does `storage::read_json` plus atomic writes inline. Swarm mutation dedup (`swarm_mutation_state.rs:218`, `:265`) holds the async `runtime.sync` write lock while doing synchronous `load_state`/`ensure_pending_state`/`save_state` filesystem IO. Slow disks stall the worker thread and every task queued behind it. The ordering guarantees are correct and must be kept; only where the blocking work runs is wrong. Two recovery behaviours also lack tests.

## What Changes

- Run `persist_swarm_state` and swarm removal file IO through `tokio::task::spawn_blocking` from async callers, still inside the per-swarm operation lock, so ordering is unchanged.
- Run mutation-state file IO under `runtime.sync` through `spawn_blocking` (awaited while the write guard is held), keeping the dedup lock semantics.
- Collapse the duplicated `swarm_operation_lock` / `swarm_file_lock` registries (`swarm_persistence.rs:33-58`) into one generic helper.
- Add tests: concurrent same-swarm persists never regress the plan version or corrupt the snapshot; a crashed pending mutation is re-dispatched once `PENDING_STATE_TTL` has elapsed.

## Capabilities

### New Capabilities

- `swarm-state-persistence`: Non-blocking, ordered swarm snapshot and mutation-state persistence.

## Impact

`crates/jcode-app-core/src/server.rs`, `server/{swarm_persistence.rs,swarm_mutation_state.rs}` and their `_tests.rs`. No on-disk format change.

## Evidence

Checked against current `~/.jcode` source:

| Finding | Result |
|---|---|
| `persist_swarm_state` takes std Mutex + sync fs from async callers | Confirmed. `swarm_persistence.rs:619-628`, called from async `persist_swarm_state_for` (`server.rs:366`) with no `spawn_blocking`. |
| Mutation state holds `runtime.sync.write().await` across load/persist | Confirmed. `swarm_mutation_state.rs:218` and `:265`; IO is sync. Async `RwLock`, so a stall risk, not a deadlock. |
| Watchdog holds member/coordinator read guards across `.await` sends | Dropped. `swarm_watchdog.rs:216-226` scopes both guards in a block that ends before the `fanout_session_event`/`queue_soft_interrupt_for_session` awaits. |
| Lock maps prune `Weak` only on insert, duplicated | Partly confirmed. Duplication confirmed (`:33-58`). Pruning claim wrong: `retain` runs on every lookup, not only insert. |
| Missing test: concurrent same-swarm persist | Confirmed. Only a lock identity/serialization test exists (`swarm_persistence_tests.rs:676`). |
| Missing test: crashed pending mutation retried after TTL | Confirmed. No test touches `PENDING_STATE_TTL` (`swarm_mutation_state.rs:13`). |
| Missing test: watchdog tool_running suppression + coordinator exclusion | Dropped. `swarm_watchdog_tests.rs:72` and `:79` cover both. |

## Resolved decisions

1. Split `swarm.rs` by action: out of scope; a separate follow-up change after these fixes land.
2. File lock type: keep `std::sync::Mutex`, used inside `spawn_blocking`, because sync callers in tests and recovery still need it.
