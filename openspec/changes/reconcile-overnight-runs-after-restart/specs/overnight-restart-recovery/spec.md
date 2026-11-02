## ADDED Requirements

### Requirement: Restart recovery is admission-driven
After restart the system SHALL reconcile interrupted runs against current configured task admission policy and persisted authorization. An overnight wake time alone SHALL NOT authorize or exclude work. Cancellation, scope and finite resource limits SHALL remain enforced.

#### Scenario: Eligible authorized task
- **WHEN** a supervisor is gone and its persisted task is still authorized, within remaining resource limits, dependency-ready and accepted by current admission policy
- **THEN** the supervisor SHALL resume from the persisted manifest with a `run_resumed` event without requiring a new overnight time window

#### Scenario: Cancellation requested
- **WHEN** the persisted state is `CancelRequested`
- **THEN** the run SHALL become `Cancelled` and SHALL NOT restart

#### Scenario: Authorization or policy mismatch
- **WHEN** current admission policy rejects the task, remaining authorization is ambiguous, or the worktree would require ungranted access
- **THEN** the run SHALL remain interrupted with the blocking reason and SHALL NOT widen its scope or create a worktree

### Requirement: Restart recovery cannot duplicate ownership
The system SHALL verify live supervisor/task ownership before starting another supervisor and SHALL retain attempt history.

#### Scenario: Supervisor still alive
- **WHEN** reconciliation finds a matching live owner
- **THEN** it SHALL reattach or report that owner and SHALL NOT launch a duplicate attempt

#### Scenario: Policy changed since admission
- **WHEN** configuration now permits broader work than the persisted task authorization
- **THEN** reconciliation SHALL retain the narrower authorization and SHALL NOT infer permission for the additional effects
