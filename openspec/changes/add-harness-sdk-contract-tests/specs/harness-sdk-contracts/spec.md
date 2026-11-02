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

The server SHALL reject unsupported versions and malformed frames with typed errors, and SHALL emit exactly one terminal event after `Cancel`.

#### Scenario: Unsupported version
- **WHEN** a client sends `hello` with `min_version` above the server major
- **THEN** the server replies with `unsupported_version` and does not open a session

#### Scenario: Cancel mid-stream
- **WHEN** a client sends `Cancel` during streaming
- **THEN** no deltas follow the single terminal event for that generation
