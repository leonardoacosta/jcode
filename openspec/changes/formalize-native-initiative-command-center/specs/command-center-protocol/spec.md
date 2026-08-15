# Command Center Protocol Delta

## ADDED Requirements

### Requirement: Revision derivation is a documented protocol property

The protocol documentation SHALL state that `Revision` is derived from the initiative's `updated_at` timestamp in milliseconds and is not an independently persisted counter.

#### Scenario: A client reasons about revision uniqueness

- **WHEN** protocol documentation describes optimistic concurrency
- **THEN** it SHALL state that revision uniqueness is bounded by millisecond resolution
- **AND** it SHALL state that two saves within one millisecond yield an identical revision
- **AND** it SHALL cite `crates/jcode-app-core/src/command_center.rs`

#### Scenario: Revision checking behavior is unchanged

- **WHEN** a client submits a mutation carrying a stale revision
- **THEN** the daemon SHALL reject it exactly as it does today
- **AND** this change SHALL NOT alter the wire shape or the rejection behavior

### Requirement: Idempotency scope is a documented protocol property

The protocol documentation SHALL state that the idempotency record is process-local.

#### Scenario: A client relies on retry safety

- **WHEN** protocol documentation describes idempotency keys
- **THEN** it SHALL state that records are held in daemon process memory
- **AND** it SHALL state that a restart clears them and a subsequent retry re-applies the command
- **AND** it SHALL reference the durable command-envelope work tracked by `optimize-orca-command-center-orchestration`

### Requirement: Unavailability vocabulary matches the implementation

Protocol documentation SHALL describe only the unavailability states the protocol implements.

#### Scenario: Orca capability is unsupported

- **WHEN** documentation describes a fail-closed Orca mutation
- **THEN** it SHALL name the `UnsupportedCapability` and `OrcaUnavailable` states
- **AND** it SHALL NOT claim an equivalent scheduler-unavailable or browser-host-unavailable protocol state
- **AND** it SHALL record that the protocol has no `degraded` variant today
