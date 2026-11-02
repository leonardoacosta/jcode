## ADDED Requirements

### Requirement: Record binary identity, hardware and workload with every resource measurement
The system SHALL record binary identity, hardware and workload with every resource measurement.

#### Scenario: Success path
- **WHEN** the bench completes **THEN** the JSON SHALL include version, revision, hardware and workload fields and SHALL NOT contain the home directory path

#### Scenario: Failure path
- **WHEN** binary identity cannot be determined **THEN** the field SHALL be `unknown` and the result SHALL be marked non-comparable
