# Spec Delta

## ADDED Requirements

### Requirement: Nonblocking session bootstrap
The system SHALL deliver session identity and requested conversation history without waiting for live MCP health collection or connection handshakes. Cached tools SHALL remain available in the initial tool catalog, and live health SHALL be published after the delayed work completes.

#### Scenario: Health collection is blocked during cached startup
- **WHEN** a client subscribes with cached MCP schemas while live health collection is pending
- **THEN** cached tool registration and session identity/history delivery complete before the health dependency is released
- **AND** cached tool counts are available without a fabricated healthy state

#### Scenario: Delayed MCP setup completes
- **WHEN** delayed MCP setup completes after the client has loaded its session
- **THEN** the client receives live MCP status without reconnecting

### Requirement: Independent connection coordination
Waiting for a server's handle or transport map SHALL NOT retain global connection coordination. Concurrent requests for the same server SHALL still share one connection attempt and observe its completion.

#### Scenario: A connection waits on a map
- **WHEN** a connection attempt is blocked on a handle or transport map
- **THEN** unrelated connection coordination remains accessible

#### Scenario: Connected server has concurrent followers
- **WHEN** concurrent requests encounter an existing live connection
- **THEN** they complete without an extra transport spawn or an abandoned waiter
