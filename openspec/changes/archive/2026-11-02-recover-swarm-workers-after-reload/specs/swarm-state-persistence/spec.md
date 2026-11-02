## ADDED Requirements

### Requirement: Worker recovery after reload
The system SHALL persist each worker's lane and prompt and, after reload, SHALL resume or mark it orphaned.

#### Scenario: Session restorable
- **WHEN** the server reloads and a worker's session restores
- **THEN** the worker SHALL resume in its lane

#### Scenario: Session lost
- **WHEN** the server reloads and a worker's session cannot be restored
- **THEN** the worker SHALL be marked `orphaned` and `retry` SHALL respawn it with its persisted lane and prompt
