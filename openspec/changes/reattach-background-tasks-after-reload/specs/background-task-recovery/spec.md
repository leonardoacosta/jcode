## ADDED Requirements

### Requirement: Background tasks survive reload
The server SHALL persist background tasks and, after reload, SHALL reattach live tasks and mark missing ones `lost`.

It SHALL reuse existing persisted task identity/manager records and retain owning execution generation. Reconciliation SHALL NOT wake revoked work or treat unknown process liveness as settled execution. Reattachment SHALL validate PID and process start identity, not PID alone.

#### Scenario: Task still running
- **WHEN** the server reloads while a background task's process is alive
- **THEN** the task SHALL remain listed and its completion SHALL be delivered to the owning session

#### Scenario: Process gone
- **WHEN** the server reloads and the task's process no longer exists
- **THEN** the task SHALL be marked `lost` with its output file still available
