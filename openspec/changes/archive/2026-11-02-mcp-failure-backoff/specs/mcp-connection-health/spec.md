## ADDED Requirements

### Requirement: Exponential reconnect backoff
The MCP pool SHALL delay automatic reconnects to a failed server using exponential backoff starting at 30 seconds, doubling per consecutive failure, capped at 30 minutes, and SHALL reset the backoff after a successful connect or a change to that server's config.

#### Scenario: Repeated failure grows delay
- **WHEN** a server fails to connect three consecutive times
- **THEN** the next automatic attempt is suppressed for at least 120 seconds

#### Scenario: Config edit resets backoff
- **WHEN** a backed-off server's config entry changes and MCP is reloaded
- **THEN** the server is attempted immediately

### Requirement: Classified failure reason with stderr tail
The failure record SHALL include a reason class and the last 20 stderr lines of a stdio server that exited before initialization.

#### Scenario: Missing working directory
- **WHEN** a server command exits with `cd: ... No such file or directory` before initialize
- **THEN** the record has class `exited_before_init` and includes that stderr line

#### Scenario: Server emits no stderr
- **WHEN** a server exits silently before initialize
- **THEN** the record has class `exited_before_init` and an empty stderr tail without error

### Requirement: Degraded state
After 5 consecutive failures the server SHALL become `degraded` and SHALL NOT be reconnected automatically until an explicit connect, reload, or config change.

#### Scenario: Degraded server stops spawning
- **WHEN** a degraded server's tool is requested by a session
- **THEN** no process is spawned and the call returns the stored reason and how to retry

#### Scenario: Manual connect clears degraded
- **WHEN** the user runs an explicit connect for a degraded server and it succeeds
- **THEN** the server returns to `connected` with zero failures

### Requirement: Deduplicated notices
The system SHALL log and surface at most one user-visible notice per server per health-state transition.

#### Scenario: Many sessions, one notice
- **WHEN** ten sessions share a pool and the server fails
- **THEN** exactly one failure notice per transition is emitted
