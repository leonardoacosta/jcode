## ADDED Requirements

### Requirement: Full lifecycle state reporting
Jcode SHALL report `idle`, `working`, `blocked` or `done` to Herdr, and the coordinator SHALL include its headless workers and background tasks with precedence blocked > working > done > idle.

#### Scenario: Worker blocked
- **WHEN** a headless worker owned by the session is blocked or stalled
- **THEN** the session's Herdr pane state SHALL be `blocked`

#### Scenario: Work finished unseen
- **WHEN** the session's turn and owned work finish while the client is unfocused
- **THEN** the state SHALL be `done` until the client takes focus, then `idle`
