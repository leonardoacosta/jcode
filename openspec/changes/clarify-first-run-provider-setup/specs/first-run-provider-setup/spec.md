## ADDED Requirements

### Requirement: Show the resolved provider or an actionable login next step at the end of first run and on first-prompt auth failure
The system SHALL show the resolved provider or an actionable login next step at the end of first run and on first-prompt auth failure.

#### Scenario: Success path
- **WHEN** onboarding finishes with a stored credential **THEN** the status row SHALL name the resolved provider and model within the available cell width

#### Scenario: Failure path
- **WHEN** no credential exists or the first prompt fails authentication **THEN** the TUI SHALL show "no working provider" and the `jcode login --provider <provider>` next step, and SHALL NOT show only a raw provider error
