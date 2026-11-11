## ADDED Requirements

### Requirement: Shared protocol fixtures

Rust and TypeScript clients SHALL parse the same golden protocol fixtures, and version constants SHALL match across both.

#### Scenario: Fixtures round-trip
- **WHEN** the TypeScript test suite parses the Rust-generated fixtures
- **THEN** every fixture is accepted and re-serializes to equivalent JSON

#### Scenario: Version drift
- **WHEN** `API_VERSION_MINOR` differs between Rust and TypeScript
- **THEN** the contract check fails naming both values

### Requirement: Error and cancel ordering

The server SHALL reject unsupported versions and malformed frames with typed errors. After `Cancel`, it SHALL emit exactly one completion boundary (`TurnDone`) for the cancelled execution generation and no later content deltas for that generation. `TurnStopped` and typed error/status events MAY precede that boundary according to the existing protocol; they SHALL NOT be counted as duplicate completion boundaries. Cancellation acknowledgment SHALL NOT be treated as durable session-stop settlement.

#### Scenario: Malformed frame
- **WHEN** a malformed request frame reaches the real server protocol boundary
- **THEN** it returns a typed error without creating a session or fabricating a successful turn completion

#### Scenario: Unsupported version
- **WHEN** a client sends `hello` with `min_version` above the server major
- **THEN** the server replies with `unsupported_version` and does not open a session

#### Scenario: Cancel mid-stream
- **WHEN** a client sends `Cancel` during streaming
- **THEN** the allowed abnormal-stop/status events precede exactly one `TurnDone`, no content deltas for that generation follow it, and another generation is not mistaken for a late delta
