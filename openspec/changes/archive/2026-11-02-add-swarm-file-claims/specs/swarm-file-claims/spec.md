## ADDED Requirements

### Requirement: Claim before edit
The swarm server SHALL record a claim for a file path by the first worker that writes it through an edit tool.

#### Scenario: First write claims
- **WHEN** worker A edits `src/lib.rs` with no existing claim
- **THEN** `src/lib.rs` is claimed by worker A

### Requirement: Conflicting edits warned or blocked
The swarm server SHALL warn a second worker writing a claimed path in `warn` mode and SHALL refuse the write in `block` mode.

#### Scenario: Warn mode
- **WHEN** worker B edits `src/lib.rs` claimed by worker A in `warn` mode
- **THEN** the edit proceeds and B receives a warning naming worker A

#### Scenario: Block mode
- **WHEN** worker B edits `src/lib.rs` claimed by worker A in `block` mode
- **THEN** the edit is refused with an error naming worker A

### Requirement: Claim release
The swarm server SHALL release all claims held by a worker when it finishes, is stopped, or is detected as orphaned.

#### Scenario: Stop releases
- **WHEN** worker A is stopped
- **THEN** none of its claims remain and worker B can claim those paths

#### Scenario: Orphan releases
- **WHEN** worker A is detected as orphaned after a reload
- **THEN** its claims are released
