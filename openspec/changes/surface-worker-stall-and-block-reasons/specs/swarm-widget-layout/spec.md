## ADDED Requirements

### Requirement: Stall and block visibility
The info widget SHALL show stalled workers with stall duration and blocked workers with a short reason, without wrapping.

#### Scenario: Stalled worker
- **WHEN** the watchdog flags a worker as stalled
- **THEN** its widget row SHALL show `stalled` and the stall age, truncated to the row width

#### Scenario: Unseen completion
- **WHEN** a worker or background task finishes and the user has not viewed it
- **THEN** the row SHALL be marked unseen and the header SHALL count it until viewed
