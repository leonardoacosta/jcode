## ADDED Requirements

### Requirement: Verify that the serving daemon runs the just-built revision after self-dev reload
The system SHALL verify that the serving daemon runs the just-built revision after self-dev reload.

#### Scenario: Success path
- **WHEN** reload completes and hashes match **THEN** the result SHALL report verified with the revision

#### Scenario: Failure path
- **WHEN** the daemon reports a different hash **THEN** the result SHALL report failed with both hashes and SHALL NOT claim success
