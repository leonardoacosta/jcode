## ADDED Requirements

### Requirement: Structured continuation evidence record
The system SHALL emit one bounded, redacted structured evidence record for every continuation decision, containing decision kind, reason, todo counts by status, fingerprint-changed flag, observed required-check results, and wait/blocked markers. The record SHALL NOT alter the decision.

#### Scenario: Decision recorded with check results
- **WHEN** a continuation decision is made after a turn in which a required check ran and failed
- **THEN** the record SHALL list that check with result `fail` and the decision SHALL equal the decision made without recording

#### Scenario: Oversized or sensitive content rejected
- **WHEN** evidence inputs contain tool output text or exceed the size bound
- **THEN** the record SHALL omit the text, be marked `truncated`, and SHALL NOT fail or delay the decision
