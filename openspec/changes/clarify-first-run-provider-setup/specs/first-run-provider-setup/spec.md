## ADDED Requirements

### Requirement: Show the resolved provider or an actionable login next step at the end of first run and on first-prompt auth failure
The system SHALL show the resolved provider or an actionable login next step at the end of first run and on first-prompt auth failure.

#### Scenario: Success path
- **WHEN** onboarding finishes with a stored credential **THEN** the status row SHALL name the selected resolved provider/model within the available cell width, label it configured/untested and SHALL NOT claim healthy or working without an observed request

#### Scenario: Failure path
- **WHEN** the selected route has no credential or its first prompt fails authentication **THEN** the TUI SHALL show the classified missing/auth-failed state and supported `jcode login --provider <provider>` next step without probing other routes or showing only a raw error
