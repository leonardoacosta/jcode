## ADDED Requirements

### Requirement: Swarm persistence does not block async executors

The server SHALL run swarm snapshot and mutation-state filesystem IO on blocking threads, never inline on a Tokio worker thread, while preserving per-swarm write ordering.

#### Scenario: Persist from async caller

- **WHEN** `persist_swarm_state_for` writes a snapshot
- **THEN** the file lock and IO run inside `spawn_blocking` while the per-swarm operation lock is held

#### Scenario: Blocking task fails

- **WHEN** the blocking persist task panics or is cancelled
- **THEN** the error is logged, the operation lock is released, and the previous snapshot stays intact

### Requirement: Concurrent same-swarm persists stay ordered

The server SHALL never let a concurrent persist for the same swarm regress the durable plan version or leave a partial snapshot.

#### Scenario: Concurrent persists

- **WHEN** many tasks persist the same swarm concurrently with increasing plan versions
- **THEN** the final snapshot parses and holds the highest plan version

#### Scenario: Stale snapshot arrives late

- **WHEN** a snapshot with a lower plan version than the one on disk is persisted
- **THEN** the write is skipped and the newer plan remains

### Requirement: Crashed pending mutations recover after TTL

The server SHALL treat a pending mutation record older than `PENDING_STATE_TTL` as stale and allow the mutation to run again.

#### Scenario: Pending record expired

- **WHEN** a pending record without a final response is older than `PENDING_STATE_TTL`
- **THEN** a new request with the same key dispatches the mutation

#### Scenario: Pending record still fresh

- **WHEN** a pending record is younger than `PENDING_STATE_TTL`
- **THEN** a duplicate request does not dispatch the mutation again
