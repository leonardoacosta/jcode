## ADDED Requirements

### Requirement: Report channel identity and roll back to a prior immutable version
The system SHALL report channel identity and roll back only to a compatible prior immutable version. Persisted control revocation/generation format and active configuration format SHALL be checked before changing channel targets; unsupported downgrade SHALL leave both channel and data unchanged with an actionable reason. Rollback SHALL NOT delete records or infer that an old binary can enforce new stop semantics.

#### Scenario: Success path
- **WHEN** the user runs `jcode update --rollback` with a compatible prior version present **THEN** the active channel SHALL point to it and the command SHALL print old and new versions

#### Scenario: Failure path
- **WHEN** no prior version exists or an update fails midway **THEN** the channel target SHALL remain unchanged and the command SHALL exit non-zero with the reason
