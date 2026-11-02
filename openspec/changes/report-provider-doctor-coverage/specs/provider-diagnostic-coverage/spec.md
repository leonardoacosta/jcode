## ADDED Requirements

### Requirement: Provider coverage matrix

`provider-doctor --coverage` SHALL report each supported provider's coverage for each failure scenario.

#### Scenario: Scenario not probed

- **WHEN** a provider has no check for a scenario
- **THEN** the matrix SHALL show `not-covered`, never pass

#### Scenario: Skipped live check

- **WHEN** a live check is skipped for missing credentials
- **THEN** the matrix SHALL show `skipped` with the reason
