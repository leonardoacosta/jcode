# judge-feedback-admission Specification

## Purpose
Bound automatic judge feedback by recorded sender identity, first-message admission and per-user-turn limits, while keeping reviewer advice distinct from user authorization.

## Requirements

### Requirement: Judge feedback admission is bounded
The system SHALL act only on the first judge DM per judge session that begins with `CONTINUE:` or `STOP:`, SHALL cap automatic continuations per user turn and SHALL admit feedback as advisory content.

#### Scenario: Valid continue under cap
- **WHEN** a judge sends one `CONTINUE:` DM and the cap is not reached
- **THEN** the parent SHALL continue with the feedback marked as reviewer advice

#### Scenario: Cap reached
- **WHEN** a `CONTINUE:` DM arrives after the cap is reached
- **THEN** the parent SHALL NOT continue and SHALL show `judge cap reached`

#### Scenario: Malformed or duplicate DM
- **WHEN** a judge DM lacks a valid prefix or is the second DM from that judge session
- **THEN** it SHALL be displayed as a note and SHALL NOT trigger continuation
