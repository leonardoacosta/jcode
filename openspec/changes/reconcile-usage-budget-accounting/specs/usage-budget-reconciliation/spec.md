## ADDED Requirements

### Requirement: Per-run usage reconciliation

Each overnight run SHALL record projected versus observed usage at completion.

#### Scenario: Reported usage

- **WHEN** providers report usage during the run
- **THEN** the report SHALL show observed usage marked `reported` alongside the projection

#### Scenario: Missing usage

- **WHEN** a provider reports no usage
- **THEN** the report SHALL show that provider as `estimated` or `unavailable`, never as zero
