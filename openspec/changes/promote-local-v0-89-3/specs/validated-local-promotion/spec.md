## ADDED Requirements

### Requirement: Validate exact candidate before activation
Promotion SHALL identify frozen source and binary provenance and SHALL pass isolated daemon, real-provider and read-only MCP checks before changing shared-server activation.

#### Scenario: Candidate acceptance succeeds
- **WHEN** the recorded candidate passes regression, build and isolated live checks
- **THEN** it becomes eligible for coordinated activation

#### Scenario: Candidate acceptance fails or source changes
- **WHEN** a required live check fails or candidate source changes after verification
- **THEN** activation waits for repair and fresh validation of the exact candidate

### Requirement: Coordinate shared session activation
Promotion SHALL preserve shared settings and SHALL coordinate active sessions and pending activation ownership before supported reload.

#### Scenario: Conflicting activation owner
- **WHEN** an unrelated active owner has a pending activation
- **THEN** promotion does not replace that activation until coordinated

#### Scenario: Shared activation succeeds
- **WHEN** the supported reload finishes
- **THEN** launcher and running shared daemon report the validated v0.89.3 candidate and read-only MCP access and session continuity checks pass

### Requirement: Preserve rollback
Promotion SHALL record and retain the previous immutable binary and channel targets.

#### Scenario: Post-activation acceptance fails
- **WHEN** the promoted daemon fails required acceptance checks
- **THEN** the previous binary remains available and supported rollback is performed or a precise blocker is reported without claiming success
