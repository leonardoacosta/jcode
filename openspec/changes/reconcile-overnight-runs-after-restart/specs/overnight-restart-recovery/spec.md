## ADDED Requirements

### Requirement: Restart reconciliation of overnight runs

On server start, the system SHALL reconcile every non-terminal overnight manifest whose supervisor is no longer alive.

#### Scenario: Resume before wake

- **WHEN** the server restarts while a run is `Running` and before its wake time
- **THEN** the supervisor SHALL resume from the manifest and record `run_resumed`

#### Scenario: Cancelled before crash

- **WHEN** the run was `CancelRequested` when the server died
- **THEN** the run SHALL become terminal cancelled and SHALL NOT resume

#### Scenario: Past wake time

- **WHEN** the server restarts after the run's wake time
- **THEN** the run SHALL be marked interrupted with a final report
