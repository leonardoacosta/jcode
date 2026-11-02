## ADDED Requirements

### Requirement: Report channel identity and roll back to a prior immutable version
The system SHALL report channel identity and roll back to a prior immutable version.

#### Scenario: Success path
- **WHEN** the user runs `jcode update --rollback` with a prior version present **THEN** the active channel SHALL point to it and the command SHALL print old and new versions

#### Scenario: Failure path
- **WHEN** no prior version exists or an update fails midway **THEN** the channel target SHALL remain unchanged and the command SHALL exit non-zero with the reason
