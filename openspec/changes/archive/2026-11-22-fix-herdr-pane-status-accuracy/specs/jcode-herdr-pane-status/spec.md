## Purpose

Keep the state that Herdr shows for a Jcode pane truthful: protect the pane from agents that Jcode tools start, report blocked and waiting-on-work states, and deliver each report until Herdr accepts it.

## ADDED Requirements

### Requirement: Tool commands do not claim the Jcode pane

Commands that the Jcode bash tool starts, in foreground, background or detached mode, SHALL NOT receive the bare `HERDR_ENV` variable. They SHALL continue to receive every other forwarded client terminal variable, including `HERDR_PANE_ID`, `HERDR_SOCKET_PATH`, `HERDR_BIN_PATH`, `HERDR_TAB_ID`, `HERDR_WORKSPACE_ID` and the `JCODE_CLIENT_HERDR_*` copies, including `JCODE_CLIENT_HERDR_ENV`. Lifecycle hook commands and Jcode's own terminal launch routing SHALL keep their current environment and Herdr routing behavior.

#### Scenario: Nested agent integration stays silent
- **WHEN** the client runs inside a Herdr pane and the bash tool starts a command
- **THEN** that command sees no `HERDR_ENV` variable, even if the server process environment has one
- **AND** an agent integration that requires `HERDR_ENV=1` does not report to the Jcode pane

#### Scenario: Herdr control from a tool command still works
- **WHEN** a bash tool command runs `herdr pane split` or `herdr pane current --current` inside a Herdr pane
- **THEN** the command reaches the client's Herdr session and resolves the client's pane

#### Scenario: Background and detached commands
- **WHEN** the bash tool starts a background or detached command
- **THEN** that command receives the same Herdr environment as a foreground command

#### Scenario: Client outside Herdr
- **WHEN** the client environment has no Herdr variables
- **THEN** tool command environments are unchanged from current behavior

### Requirement: Blocked state for user-required todos

While the active session has an incomplete todo with a nonblank `requires_user_auth` reason, the Jcode client in a Herdr pane SHALL report state `blocked` with that reason as the report message. When several such todos exist, the client SHALL use the first in todo order. The message SHALL contain no control characters and SHALL be at most 200 bytes, truncated at a character boundary.

#### Scenario: Login needed
- **WHEN** the active todo list contains an incomplete todo whose `requires_user_auth` reason is "Type login password into the Unlock Login Keyring prompt"
- **THEN** Herdr receives state `blocked` with a message that contains that reason

#### Scenario: Blocker resolved
- **WHEN** that todo becomes completed or cancelled, or its `requires_user_auth` value is cleared
- **THEN** the client reports the state that the remaining conditions give

#### Scenario: Blocked during a turn
- **WHEN** a turn is processing and a `requires_user_auth` todo is incomplete
- **THEN** the reported state is `blocked`

#### Scenario: Ordinary dependency waits
- **WHEN** a todo is blocked only by `blocked_by` or `waiting_on`, or a swarm member has status `blocked`
- **THEN** that condition alone does not cause state `blocked`

### Requirement: Working while owned work runs

The Jcode client in a Herdr pane SHALL report state `working` when it is not blocked and any of these conditions is true: it processes a turn; a swarm member in its own spawn subtree has status `running`, `streaming` or `thinking`; or a background task of the active session runs and was started with wake enabled. Otherwise it SHALL report `idle`. Precedence SHALL be blocked > working > idle.

#### Scenario: Waiting on a swarm worker
- **WHEN** no turn is processing and a worker spawned by this session has status `running`
- **THEN** Herdr receives state `working`
- **AND** Herdr receives state `idle` after the worker reaches a non-running status and no other condition holds

#### Scenario: Foreign swarm members
- **WHEN** a running swarm member shares the swarm but is outside this session's spawn subtree
- **THEN** it does not make this pane `working`

#### Scenario: Waiting on a wake background task
- **WHEN** no turn is processing and a running background task of this session was started with wake enabled
- **THEN** Herdr receives state `working` until that task completes, fails or is cancelled

#### Scenario: Background task without wake
- **WHEN** the only running work is a background task started without wake, such as a development server
- **THEN** Herdr receives state `idle`

### Requirement: Reports are delivered until accepted

The client SHALL treat a state report as delivered only after the Herdr command exits successfully. After a failure or a timeout, it SHALL submit the latest desired report again with a fresh increasing sequence number and bounded backoff. The first retry SHALL wait at least 1 second, and the wait between attempts SHALL be at most 60 seconds. A newer desired report SHALL replace a pending retry. Release on quit SHALL cancel retries. An undelivered desired report SHALL NOT be dropped as a duplicate; a repeated request for the same desired report SHALL NOT start an extra attempt or reset the backoff. Retries SHALL keep the existing clear-before-replace ordering for session changes. Reporting SHALL never block the user interface.

#### Scenario: Herdr temporarily unavailable
- **WHEN** a report times out or fails and the state does not change afterward
- **THEN** the client retries until a report succeeds
- **AND** after success it sends no further reports until the desired report changes

#### Scenario: State changes during backoff
- **WHEN** the desired state changes while a retry is pending
- **THEN** the next attempt sends the newest state, not the stale one

#### Scenario: Quit during backoff
- **WHEN** the user quits while a retry is pending
- **THEN** the client sends the release and no later state report

#### Scenario: Unsupported resume compatibility
- **WHEN** Herdr rejects a report because of its resume command with a usage error
- **THEN** the existing compatibility fallback still applies, and the retried report keeps the session ID

### Requirement: Diagnosable report failures

When a Herdr command fails or times out, the client SHALL log the command kind, the reported state, the sequence number, the exit status or timeout, and Herdr's error output limited to 512 bytes. The log SHALL NOT include environment values other than the pane ID.

#### Scenario: Herdr rejects a report
- **WHEN** Herdr exits with a nonzero status and prints `resume_not_accepted` on its error output
- **THEN** the Jcode log line contains the state, sequence number, exit status and that error text

#### Scenario: Large error output
- **WHEN** Herdr prints more than 512 bytes of error output
- **THEN** the log contains at most 512 bytes of it
