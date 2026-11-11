## ADDED Requirements

### Requirement: Stall and block visibility
The info widget SHALL show stalled workers with stall duration and blocked workers with a short reason, without wrapping.

It SHALL project existing swarm/background/control types without losing stopped, crashed, cancelled, draining or unresolved outcomes. Display projection SHALL NOT establish execution settlement or successful work. Actions SHALL be capability/owner checked; unavailable pane focus SHALL be omitted.

#### Scenario: Stalled worker
- **WHEN** the watchdog flags a worker as stalled
- **THEN** its widget row SHALL show `stalled` and the stall age, truncated to the row width

#### Scenario: Unseen completion
- **WHEN** a worker or background task finishes and the user has not viewed it
- **THEN** the row SHALL be marked unseen by completion-event ID and this consumer SHALL count it until viewed, without clearing another consumer's cursor or changing execution state
