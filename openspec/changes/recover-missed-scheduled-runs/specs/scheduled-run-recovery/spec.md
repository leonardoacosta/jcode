## ADDED Requirements

### Requirement: Durable scheduled delivery

Scheduled items SHALL remain persisted until delivery is acknowledged.

#### Scenario: Crash before delivery

- **WHEN** the server dies after an item becomes ready but before delivery
- **THEN** on restart the item SHALL be redelivered once marked `recovered`

#### Scenario: Normal delivery

- **WHEN** an item is delivered and acknowledged
- **THEN** it SHALL move to history as `delivered` and SHALL NOT be redelivered

### Requirement: Inspectable schedule history

`schedule list` SHALL show recent terminal items with their outcome.

#### Scenario: Missed item

- **WHEN** an item was overdue beyond the grace window at startup
- **THEN** the list SHALL show it as `missed` with the delay
