# shared-server-lifecycle Specification

## Purpose
The shared jcode daemon stays resident regardless of idle time, temporary servers idle-exit only with no clients or pending work, SIGTERM and reload stop new work while pending work is preserved for recovery, and the CLI and TUI show live server and client counts with uptime.

## Requirements

### Requirement: Shared daemon remains resident

The shared `jcode serve` daemon MUST NOT exit because no clients are connected or because an idle timeout elapsed. Temporary and self-development servers MAY idle-exit only when no clients, active workers, or pending scheduled, ambient, or background work remain. A temporary server with an owner PID MUST check that owner's liveness and shut down when the owner is gone; owner liveness does not permit discarding pending work.

#### Scenario: Shared service remains idle

- **WHEN** the shared daemon has no connected clients for longer than its former idle timeout
- **THEN** the daemon remains running and does not exit with the idle-timeout status
- **AND** a systemd user unit configured with `Restart=always` does not enter a restart loop caused by normal idleness

#### Scenario: Temporary server owner exits

- **WHEN** a temporary server records an owner PID and that process is no longer alive
- **THEN** the temporary server begins orderly shutdown
- **AND** it preserves or hands off pending work before removing its temporary registry and socket artifacts

#### Scenario: Temporary server has pending scheduled work

- **WHEN** a temporary server reaches its idle timeout while scheduled, ambient, or background work is pending or active
- **THEN** it does not idle-exit and continues servicing that work

#### Scenario: Ownerless temporary server is idle

- **WHEN** a temporary server has no owner PID, no clients, no active workers, and no pending work for its configured idle interval
- **THEN** it exits through the temporary-server cleanup path

### Requirement: Shutdown and reload preserve work

The server MUST handle SIGTERM and reload as orderly transitions. It MUST stop admitting new work, drain in-flight session saves, and preserve pending work through the existing durable recovery contracts before process exit or replacement. It MUST NOT report pending work complete solely because shutdown began.

#### Scenario: systemd sends SIGTERM

- **WHEN** systemd stops or restarts the shared service while work is pending
- **THEN** the server exits cleanly after the configured shutdown procedure
- **AND** pending work remains available to the restarted daemon through its existing recovery path

#### Scenario: Reload replaces server process

- **WHEN** a reload replaces a server while scheduled or background work is pending or active
- **THEN** replacement preserves or reattaches that work according to its recovery contract
- **AND** no work is marked complete only because the old process exited

#### Scenario: Save drain exceeds its bound

- **WHEN** an in-flight session save does not finish within the shutdown drain bound
- **THEN** shutdown reports the incomplete drain and does not claim that the save completed
- **AND** recovery retains sufficient state to avoid silently treating pending work as complete

### Requirement: Server and client status is observable

The CLI and TUI MUST show the number of registered live servers, the connected-client count, and each listed server's uptime derived from registry `started_at`. Status reads MUST use the registry stale-entry cleanup behavior and MUST NOT present stale entries as live. Compact TUI rows MUST fit the available terminal-cell width or truncate with an ellipsis without wrapping.

#### Scenario: Multiple servers are live

- **WHEN** the user opens server status with multiple live server records
- **THEN** the CLI and TUI show the live server count, connected-client count, and uptime for each server

#### Scenario: Registry contains stale entries

- **WHEN** the registry contains an entry whose process or socket is no longer live
- **THEN** the status view excludes it from live counts and rows
- **AND** the stale entry is cleaned using the registry's existing cleanup behavior

#### Scenario: Narrow terminal

- **WHEN** a TUI server-status row exceeds the available terminal width
- **THEN** the row remains one visual row and truncates at terminal-cell boundaries with an ellipsis
- **AND** adjacent status rows do not wrap

#### Scenario: Invalid start timestamp

- **WHEN** a live registry entry has an invalid or future `started_at` value
- **THEN** status remains available and shows an explicit unavailable uptime instead of failing the whole view
