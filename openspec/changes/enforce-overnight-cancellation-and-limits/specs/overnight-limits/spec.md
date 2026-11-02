## ADDED Requirements

### Requirement: Mid-turn cancellation

The supervisor SHALL abort an active coordinator turn and run-owned child work when cancellation is requested.

#### Scenario: Cancel during long turn

- **WHEN** the user cancels while a coordinator turn is running
- **THEN** the turn SHALL be aborted within one sample interval, run-owned workers SHALL be stopped, and no later turn SHALL launch

#### Scenario: Cancel already finished

- **WHEN** cancel arrives after the run completed
- **THEN** the request SHALL be rejected and the terminal state SHALL remain unchanged

### Requirement: Enforced run limits

Configured agent, turn-time and token limits SHALL be enforced, not advisory.

#### Scenario: Agent cap reached

- **WHEN** the coordinator spawns a worker beyond `max_agents`
- **THEN** the spawn SHALL be refused with a visible reason

#### Scenario: Within limits

- **WHEN** usage stays under every limit
- **THEN** the run SHALL proceed without limit events
