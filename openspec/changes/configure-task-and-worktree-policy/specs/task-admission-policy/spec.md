## ADDED Requirements

### Requirement: Monetary dispatch requires authoritative pre-send reservations
Task dispatch SHALL reserve a provider-enforced upper monetary bound before each chargeable send. Reservations SHALL be nonrefundable across crashes, cancellation, retries and restart, and all child work SHALL share the original task ceiling. Estimated prices, model text and configuration alone SHALL NOT create an authoritative capability.

#### Scenario: Unsupported provider cannot send
- **WHEN** a task selects a provider without an authoritative bounded monetary capability
- **THEN** dispatch SHALL stop before inference transport or child provider effects

#### Scenario: Concurrent bounded requests exhaust the original ceiling
- **WHEN** bounded requests, retries or child work compete for the remaining task budget
- **THEN** atomic reservations SHALL prevent their aggregate upper bound from exceeding the original authorization or a stricter current limit

#### Scenario: Reservation state cannot be refunded by restart
- **WHEN** an attempt restarts after cancellation, failure or an incomplete send
- **THEN** it SHALL retain consumed reservations and SHALL fail closed on missing or corrupt accounting state

### Requirement: Typed task policy and provenance
Jcode SHALL load typed task-admission policy through its existing global TOML configuration and expose the effective settings with their source. New unattended admission SHALL default to disabled and configuration support SHALL NOT itself grant task authorization.

#### Scenario: No new settings
- **WHEN** existing configuration has no task-admission section
- **THEN** it SHALL load compatibly and SHALL NOT enable unattended dispatch

#### Scenario: Invalid or conflicting policy
- **WHEN** a task limit is invalid or a project override broadens a user-scoped restriction
- **THEN** the affected policy SHALL be rejected or disabled with a specific reason rather than silently granting broader access

### Requirement: Explainable deterministic admission
Admission SHALL distinguish ready, waiting, blocked and needs-human outcomes using persisted authorization, task scope, dependencies, cancellation, finite remaining limits and workspace readiness. Time alone SHALL NOT grant or revoke the task's authorization.

#### Scenario: Task eligible outside an overnight window
- **WHEN** an explicitly authorized task satisfies current policy, dependencies, workspace and remaining limits without a configured overnight window
- **THEN** it MAY be admitted with an attributable decision and SHALL NOT require a new clock window

#### Scenario: Unknown authority or cancellation
- **WHEN** authorization is unknown, cancellation is set, or required ownership cannot be verified
- **THEN** the task SHALL NOT dispatch and its state SHALL explain the blocker or human decision required

### Requirement: Policy drift does not widen work
Restart and re-admission SHALL revalidate current policy against the task's original authorization and attempt ownership. A broader new configuration SHALL NOT silently expand the task's effects or budget.

#### Scenario: Broader configuration after restart
- **WHEN** current config allows more filesystem access or spend than the task originally received
- **THEN** the task SHALL retain the original narrower authority and SHALL NOT acquire those additional effects

#### Scenario: Existing live attempt
- **WHEN** admission finds a live matching owner for the task
- **THEN** it SHALL report or attach to the existing attempt and SHALL NOT create a duplicate attempt
