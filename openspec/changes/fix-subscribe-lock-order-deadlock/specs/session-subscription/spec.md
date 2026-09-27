## ADDED Requirements

### Requirement: Subscribe remains responsive during swarm cleanup

The server MUST complete a session subscribe without waiting on a cyclic lock dependency between swarm membership state and the swarm index. Concurrent coordinator election and subscribe/reconnect bookkeeping MUST NOT hold both maps while waiting for the other map.

#### Scenario: Coordinator is removed while a client reconnects

- **WHEN** a swarm coordinator is removed while a client subscribes or reconnects
- **THEN** coordinator election and subscribe bookkeeping can make progress
- **AND** the subscribe request returns its normal session identity and completion events
