## 1. Non-blocking persistence

- [x] 1.1 Wrap `persist_swarm_state` and removal IO in `spawn_blocking` from `persist_swarm_state_for` / `remove_persisted_swarm_state_for` in `server.rs`, inside the operation lock; log join errors
- [x] 1.2 Move `load_state`/`ensure_pending_state`/`save_state` calls under `runtime.sync` in `swarm_mutation_state.rs` into awaited `spawn_blocking`
- [x] 1.3 Replace the duplicated lock registries in `swarm_persistence.rs` with one generic helper

## 2. Tests

- [x] 2.1 Concurrent same-swarm persist test: final snapshot parses with the highest plan version
- [x] 2.2 Stale-snapshot skip test
- [x] 2.3 Pending mutation older than `PENDING_STATE_TTL` re-dispatches; a fresh pending one does not

## 3. Verify

- [x] 3.1 `cargo test -p jcode-app-core swarm_`
- [x] 3.2 `openspec validate swarm-state-async-safety --strict`
