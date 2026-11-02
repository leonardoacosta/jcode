## ADDED Requirements

### Requirement: Shadow-only semantic verdict
The system SHALL, when shadow mode is enabled and supervision is configured, record a Jev verdict for each continuation decision without changing the executed deterministic decision.

#### Scenario: Verdict logged beside decision
- **WHEN** shadow mode is on and a continuation decision is made
- **THEN** the verdict, latency and call count SHALL be logged and the executed decision SHALL equal the deterministic one

#### Scenario: Failure never loops or grants authority
- **WHEN** Jev times out, errors, or returns malformed output
- **THEN** the verdict SHALL be recorded as `uncertain`, no extra turn SHALL be queued, and no check SHALL be skipped

### Requirement: Replay comparison
The system SHALL provide an offline replay comparing shadow verdicts with deterministic decisions, reporting missed required checks and redundant turns separately.

#### Scenario: Missed check flagged
- **WHEN** a verdict would stop while a required check had not passed
- **THEN** the report SHALL count it as a missed required check

#### Scenario: Missing evidence
- **WHEN** replay input lacks evidence records
- **THEN** the replay SHALL fail with a clear error instead of reporting zero misses
