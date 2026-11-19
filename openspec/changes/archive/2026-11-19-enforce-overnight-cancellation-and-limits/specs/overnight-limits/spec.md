## ADDED Requirements

### Requirement: Mid-turn cancellation

The supervisor SHALL abort an active coordinator turn and run-owned child work when cancellation is requested.

#### Scenario: Cancel during long turn

- **WHEN** the user cancels while a coordinator turn is running
- **THEN** cancellation SHALL be observed and admission revoked within one sample interval, owned work SHALL drain through durable control, and no later turn SHALL launch; terminal stopped/cancelled settlement SHALL be recorded only when owned work settles, otherwise unresolved remains explicit

#### Scenario: Cancel already finished

- **WHEN** cancel arrives after the run completed
- **THEN** the request SHALL be rejected and the terminal state SHALL remain unchanged

### Requirement: Enforced run limits

Configured agent, turn-time and token limits SHALL be enforced, not advisory.

Aggregate usage SHALL include run-owned generations with deduplication. Reported versus estimated usage SHALL remain explicit; unavailable authoritative measurement SHALL NOT create an exact-enforcement claim. Token/time limits SHALL NOT replace required pre-send monetary authorization/reservations.

#### Scenario: Agent cap reached

- **WHEN** the coordinator spawns a worker beyond `max_agents`
- **THEN** the spawn SHALL be refused with a visible reason

#### Scenario: Within limits

- **WHEN** usage stays under every limit
- **THEN** the run SHALL proceed without limit events
