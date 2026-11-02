## ADDED Requirements

### Requirement: Queue messages for busy workers
The system SHALL queue non-interrupt messages addressed to a busy worker and deliver them at that worker's turn end.

#### Scenario: DM to busy worker
- **WHEN** a coordinator sends a DM to a worker mid-turn
- **THEN** the sender SHALL receive a `queued` acknowledgement and the worker SHALL receive the message after its turn ends

#### Scenario: Queue full
- **WHEN** the queue exceeds its bound
- **THEN** the oldest message SHALL be dropped and its sender notified
