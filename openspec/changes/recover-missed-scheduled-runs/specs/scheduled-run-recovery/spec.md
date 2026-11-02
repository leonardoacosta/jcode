## ADDED Requirements

### Requirement: Durable scheduled delivery

Scheduled items SHALL remain persisted until delivery is acknowledged.

#### Scenario: Delivery acknowledgment missing

- **WHEN** the server restarts with a dispatched item whose delivery was not acknowledged
- **THEN** the item SHALL be marked `uncertain` and SHALL NOT be automatically redelivered

#### Scenario: Delivery executed before crash

- **WHEN** an item executed but the server crashed before recording its acknowledgment
- **THEN** missing acknowledgment SHALL NOT trigger duplicate execution and explicit manual retry SHALL be required

#### Scenario: Explicit manual retry

- **WHEN** the user explicitly retries an uncertain delivery
- **THEN** a new recorded attempt SHALL be created while preserving the uncertain attempt's history

#### Scenario: Normal delivery

- **WHEN** an item is delivered and acknowledged
- **THEN** it SHALL move to history as `delivered` and SHALL NOT be redelivered

### Requirement: Inspectable schedule history

`schedule list` SHALL show recent terminal items with their outcome.

#### Scenario: Missed item

- **WHEN** an item was overdue beyond the grace window at startup
- **THEN** the list SHALL show it as `missed` with the delay without automatically delivering it
