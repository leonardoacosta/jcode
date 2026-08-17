## ADDED Requirements

### Requirement: Repository integration detection
The preflight SHALL determine repository identity and readiness of OpenSpec, Beads, and telemetry without mutating repository state.

#### Scenario: All integrations exist
- **WHEN** OpenSpec and Beads are initialized and telemetry is detectable
- **THEN** the preflight records readiness and continues without prompting

### Requirement: One-time initialization consent
The preflight SHALL ask once per repository before initializing missing OpenSpec or Beads and SHALL never initialize silently.

#### Scenario: One integration is missing
- **WHEN** one integration is absent and no preference exists
- **THEN** Jcode asks one focused consent question

#### Scenario: Both integrations are missing
- **WHEN** both integrations are absent and no preference exists
- **THEN** Jcode asks one combined question allowing both, either, or neither

#### Scenario: User accepts
- **WHEN** the user approves initialization
- **THEN** Jcode runs the canonical non-interactive initializer
- **AND** rechecks readiness

#### Scenario: User declines
- **WHEN** the user declines
- **THEN** Jcode records the repository-scoped decline
- **AND** continues in explicit degraded mode without repeating the prompt

### Requirement: Preference reset
Jcode SHALL provide an explicit reset for repository integration preferences.

#### Scenario: User resets a decline
- **WHEN** the preference is reset or setup is explicitly requested
- **THEN** a later workflow may ask again

### Requirement: Fail-soft integration behavior
A missing or failed optional integration SHALL NOT prevent a truthful degraded route.

#### Scenario: Initializer fails
- **WHEN** approved initialization fails
- **THEN** Jcode reports the exact failure and does not claim readiness
- **AND** continues only through an identified degraded route
