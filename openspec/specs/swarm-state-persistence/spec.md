# swarm-state-persistence Specification

## Purpose
Defines how swarm state persistence and mutation dedup perform file I/O without blocking async workers while preserving per-swarm ordering.

## Requirements

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

### Requirement: Worker recovery after reload
The system SHALL persist each worker's lane, prompt, stable identity, retained owner ancestry, execution generation and stop control state. After reload it SHALL reconcile actual execution and owned tool work before allowing runnable recovery. It SHALL resume only previously runnable authorized work whose predecessor is settled, or mark unavailable work orphaned. Persisted cancellation SHALL take precedence over automatic resume, retry and membership restoration.

#### Scenario: Session restorable
- **WHEN** the server reloads and an authorized worker's session restores without a stop barrier or unsettled predecessor
- **THEN** the worker SHALL recover in its lane under the persisted identity and owner, with any newly admitted execution assigned a new generation

#### Scenario: Session lost
- **WHEN** the server reloads, a worker's session cannot be restored, and no pending stop barrier exists
- **THEN** the worker SHALL be marked `orphaned` and an explicit authorized `retry` SHALL respawn it from its persisted lane and prompt only after predecessor settlement

#### Scenario: Restart during stop
- **WHEN** restart interrupts a persisted stop before settlement is recorded
- **THEN** recovery SHALL retain revoked admission and ownership, reconcile outstanding work, and report draining or unresolved until settlement is proved, without rerunning cancelled work

#### Scenario: Previously stopped session stub exists
- **WHEN** recovery finds a valid session stub for a durably stopped generation
- **THEN** it SHALL preserve the stopped disposition and SHALL NOT auto-register a runnable ownerless member

### Requirement: Control persistence is monotonic and fail closed
Persisted owner, generation and stop state SHALL survive swarm snapshot removal and ordinary terminal-member retention. Concurrent or stale writes SHALL NOT lower the generation or clear a stop barrier. Missing legacy fields SHALL preserve historical data without inventing ownership or treating old reports as fresh. Corrupt or unsupported control state SHALL refuse runnable recovery with an actionable diagnostic rather than discard state or assume permission. Unsupported downgrade SHALL NOT silently erase stop intent.

#### Scenario: Old snapshot arrives after stop
- **WHEN** a pre-stop snapshot is written after a newer durable stop record
- **THEN** the stopped generation SHALL remain revoked and its owner relationship SHALL remain available

#### Scenario: Legacy snapshot loads
- **WHEN** a snapshot lacks generation/control fields
- **THEN** history SHALL remain readable, unknown ownership or unsettled execution SHALL require reconciliation, and legacy reports SHALL NOT satisfy a newly admitted run

#### Scenario: Corrupt control record
- **WHEN** control persistence is corrupt or the state version is unsupported
- **THEN** the daemon SHALL retain the source bytes, deny execution for that session and report the recovery limitation without changing permissions

#### Scenario: Old binary cannot enforce control state
- **WHEN** a downgrade would ignore persisted revocation or generation data
- **THEN** the supported rollback procedure SHALL prevent execution against that state rather than delete records or claim safe compatibility
