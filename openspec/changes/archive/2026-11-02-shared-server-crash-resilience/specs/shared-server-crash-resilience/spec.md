## ADDED Requirements

### Requirement: Server exit cause capture
The shared server SHALL persist an exit record identifying its PID and exit cause (clean shutdown, signal name, or panic message and location) before exiting whenever the process can run code at exit.

#### Scenario: Caught termination signal
- **WHEN** the server receives SIGTERM and shuts down
- **THEN** an exit record with cause `signal` and name `SIGTERM` exists for its PID

#### Scenario: Panic
- **WHEN** the server panics
- **THEN** an exit record with cause `panic`, the panic message, and source location is written before the process exits

#### Scenario: Uncatchable kill
- **WHEN** the server is killed by SIGKILL and no exit record exists
- **THEN** crash detection reports a cause from supervisor data if available, otherwise `unknown`, and never reports a clean shutdown

### Requirement: Crash cause classification
Crash detection SHALL record a classified cause on each crashed session instead of a fixed generic message.

#### Scenario: OOM under systemd
- **WHEN** the server ran under the supervised unit and the unit reports `Result=oom-kill`
- **THEN** affected sessions record cause `oom`

#### Scenario: Record unreadable
- **WHEN** the exit record is missing or corrupt
- **THEN** detection falls back to `unknown` and still marks the sessions crashed

### Requirement: Supervised shared server gate
When `~/.jcode/require-supervised-server` names a systemd user unit, a non-temporary `serve` SHALL run only inside that unit's cgroup.

#### Scenario: Started outside unit
- **WHEN** `jcode serve` starts outside the named unit's cgroup
- **THEN** it requests `systemctl --user start <unit>` and exits without serving

#### Scenario: No marker or temporary server
- **WHEN** the marker is absent or empty, or the server is temporary
- **THEN** `serve` starts normally

#### Scenario: Delegation fails
- **WHEN** `systemctl` is unavailable or the start request fails
- **THEN** `serve` exits with an error naming the marker file and unit

### Requirement: Grouped session recovery after crash
Sessions orphaned by the same server PID SHALL be grouped into one crash event recoverable in bulk, without automatically re-running in-flight turns.

#### Scenario: Mass crash
- **WHEN** a server PID owning 100 active sessions dies
- **THEN** the next client sees one crash notice with cause and count, and can restore all 100 sessions in one action

#### Scenario: In-flight turn
- **WHEN** a recovered session had a turn in progress at crash time
- **THEN** the session is restored with that turn marked interrupted and is not resumed automatically
