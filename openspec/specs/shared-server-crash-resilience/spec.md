# shared-server-crash-resilience Specification

## Purpose
Record shared-server exits and recover interrupted sessions through bounded, classified supervision rather than silent session loss.

## Requirements

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

### Requirement: Resume does not exhaust worker stacks
The shared server SHALL resume a large persisted session without overflowing a runtime worker stack, in both selfdev (opt-level 0) and release builds.

#### Scenario: Large session resume
- **WHEN** a client sends `resume_session` for a persisted session with 842 messages to a selfdev-profile daemon
- **THEN** the session is restored and the daemon process is still alive

#### Scenario: Regression baseline
- **WHEN** the same request is sent to build `8a2141d07`
- **THEN** the daemon exits 134, demonstrating the harness detects the overflow

### Requirement: Resume poison quarantine
The server SHALL write `~/.jcode/resume-in-progress/<sanitized session id>` containing its pid before restoring a session and remove it after the restore returns. A marker whose pid is dead SHALL block resume of that session only.

#### Scenario: Marker from a dead pid
- **WHEN** a marker for session S names a pid that is not alive and a client resumes S
- **THEN** the resume fails with an error containing "died while restoring" and the marker path, and the server stays alive

#### Scenario: Other sessions unaffected
- **WHEN** a dead-pid marker exists for session S and a client resumes session T
- **THEN** T restores normally

#### Scenario: Live or own pid
- **WHEN** the marker names the current server pid or a live process
- **THEN** resume proceeds

#### Scenario: Unreadable marker
- **WHEN** the marker is missing, unreadable or not a pid
- **THEN** resume proceeds

#### Scenario: Operator retry
- **WHEN** the operator removes the marker named in the error and resumes again
- **THEN** the restore is attempted

### Requirement: Clients respect a supervisor that gave up
A client SHALL NOT run `systemctl --user start <unit>` when `systemctl --user show <unit> -p Result` reports `start-limit-hit`, and SHALL instead report the unit, last exit status and the `journalctl` / `reset-failed` recovery commands. `NRestarts` SHALL NOT be used as the signal.

#### Scenario: Start limit hit
- **WHEN** `JCODE_SERVER_SYSTEMD_UNIT` names a unit whose `Result=start-limit-hit` and a client needs a server
- **THEN** no `systemctl start` is issued and the client errors with the refusal text

#### Scenario: Unit merely stopped
- **WHEN** the unit's `Result` is `success` or any value other than `start-limit-hit`
- **THEN** the client may start it as before

#### Scenario: Both start sites gated
- **WHEN** either the supervised `serve` gate or the client spawn path would start the unit
- **THEN** both apply the same refusal check

### Requirement: Client output never corrupts a live TUI
Client-spawned `systemctl` processes SHALL have stdin, stdout and stderr detached, and `stderr_info` SHALL print nothing while the terminal is in raw mode.

#### Scenario: Reconnect during TUI
- **WHEN** a TUI in raw mode reconnects and the client starts or refuses the systemd unit
- **THEN** no bytes are written to the terminal outside the TUI's own frames

#### Scenario: Cooked-mode CLI
- **WHEN** raw mode is off and quiet mode is off
- **THEN** `stderr_info` still prints

### Requirement: Lock contention ends restart churn
A supervised `serve` that finds the daemon lock held SHALL retry every 250 ms for up to 10 s. Any `serve` that ends with an "already running" error SHALL exit with status 78.

#### Scenario: Unsupervised contention
- **WHEN** a second unsupervised `serve` targets an occupied socket
- **THEN** it exits 78 immediately

#### Scenario: Supervised contention persists
- **WHEN** `JCODE_SERVER_SYSTEMD_UNIT` is set and the lock stays held
- **THEN** `serve` exits 78 after about 10 s

#### Scenario: Lock released during wait
- **WHEN** a supervised `serve` waits and the holder exits within 10 s
- **THEN** the new `serve` acquires the lock and starts serving

### Requirement: Supervised daemon stays resident
When `JCODE_SERVER_SYSTEMD_UNIT` is set to a non-empty value, the server SHALL NOT exit on idle timeout.

#### Scenario: Supervised idle
- **WHEN** a supervised server has no clients past the idle timeout
- **THEN** it keeps running and logs that the idle monitor is skipped

#### Scenario: Unsupervised idle
- **WHEN** the variable is unset or blank
- **THEN** idle-timeout behaviour is unchanged

### Requirement: Startup scan ignores non-record files
The pending await-members startup scan SHALL parse only `.json` files.

#### Scenario: Orphaned backup
- **WHEN** a `.bak` file sits in the await-members state directory at daemon start
- **THEN** it is skipped without a parse warning

### Requirement: Promotion proves resume
`jcode server promote` SHALL, before repointing `shared-server`, boot the candidate binary with a private `--socket` and a temporary copy of `JCODE_HOME`, resume a copy of the newest real persisted session through `Client::resume_session_with_options`, and wait for `ServerEvent::Done` within a bounded timeout. Any error, timeout or candidate exit SHALL abort promotion and leave `shared-server` unchanged. `--skip-resume-check` SHALL bypass the check and say so in the report.

#### Scenario: Bad binary
- **WHEN** the candidate crashes resuming the session (for example build `8a2141d07`)
- **THEN** promotion is refused, the report names the candidate exit status, and `shared-server` still points at the previous version

#### Scenario: Good binary
- **WHEN** the candidate resumes and reports Done
- **THEN** promotion proceeds and the temporary home and socket are removed

#### Scenario: No session to test
- **WHEN** no persisted session exists
- **THEN** the check reports that it was skipped for lack of a session and promotion proceeds

#### Scenario: User data untouched
- **WHEN** the check runs
- **THEN** the real `~/.jcode` sessions and the shared daemon socket are not modified

#### Scenario: Opt-out
- **WHEN** `--skip-resume-check` is passed
- **THEN** no candidate is booted and the report states the check was skipped by flag
