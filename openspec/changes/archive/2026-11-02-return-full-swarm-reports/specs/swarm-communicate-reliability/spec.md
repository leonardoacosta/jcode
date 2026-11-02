## ADDED Requirements

### Requirement: Full worker reports returned to coordinator
The system SHALL return a worker's complete final report from `summary` and `await_members`, without per-line truncation below the size cap.

#### Scenario: Summary after report
- **WHEN** a worker has submitted a report and the coordinator calls `summary` for it
- **THEN** the output SHALL contain the full report body before any tool-call listing

### Requirement: Report-based completion
The system SHALL treat a member as done only when it submitted a report, and SHALL keep it done after it stops.

#### Scenario: Ready without report
- **WHEN** a member reaches `ready` without submitting a report
- **THEN** `await_members` SHALL list it as `incomplete`

#### Scenario: Reported then stopped
- **WHEN** a member submitted a report and then stopped
- **THEN** it SHALL count as done and SHALL NOT trigger a timeout wake-up
