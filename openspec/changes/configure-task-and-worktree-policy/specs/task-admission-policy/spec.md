## ADDED Requirements

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
