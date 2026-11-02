# swarm-communicate-reliability Specification

## Purpose
Defines how the swarm `communicate` tool bounds request deadlines, reports timeouts versus server errors, and keeps advertised actions aligned with dispatch.

## Requirements

### Requirement: Per-action request deadlines

The communicate tool SHALL apply a socket deadline chosen per action, using a longer deadline for mutating actions (`spawn`, `launch`, `retry`, `resume`) than for read actions.

#### Scenario: Slow spawn completes within extended deadline

- **WHEN** a `spawn` request takes 45 seconds to receive its response
- **THEN** the tool returns the new session id instead of a timeout error

#### Scenario: Mutating action exceeds its deadline

- **WHEN** a `spawn` request receives no response before its deadline
- **THEN** the error states the spawn may have succeeded and instructs checking `status` or `list` before retrying

### Requirement: Classified request errors

The communicate tool SHALL distinguish client timeouts, closed connections and server-reported errors in the messages it returns.

#### Scenario: Server rejects request

- **WHEN** the server returns an `Error` event for a request
- **THEN** the message contains the server error text and does not mention a timeout

#### Scenario: Socket times out

- **WHEN** no terminal response arrives before the deadline
- **THEN** the message identifies a client-side timeout and the action name

### Requirement: Unknown-action message lists advertised actions

The unknown-action error SHALL list exactly the actions in `ADVERTISED_SWARM_ACTIONS`.

#### Scenario: Agent sends unknown action

- **WHEN** the action is `frobnicate`
- **THEN** the error lists `launch`, `list_models` and `list_swarms` and omits internal-only actions

#### Scenario: Advertised action dispatches

- **WHEN** any action in `ADVERTISED_SWARM_ACTIONS` is dispatched
- **THEN** it does not produce the unknown-action error

### Requirement: await_members honors background flag

`await_members` SHALL run a blocking wait with a deadline of `timeout_secs + 30` when `background=false`, and a background wait otherwise.

#### Scenario: Blocking wait requested

- **WHEN** `await_members` is called with `background=false` and members finish
- **THEN** the tool returns the member results inline

#### Scenario: Blocking wait times out

- **WHEN** `background=false` and members do not finish within `timeout_minutes`
- **THEN** the tool returns a timeout result naming the pending members

### Requirement: Queue messages for busy workers
The system SHALL queue non-interrupt messages addressed to a busy worker and deliver them at that worker's turn end.

#### Scenario: DM to busy worker
- **WHEN** a coordinator sends a DM to a worker mid-turn
- **THEN** the sender SHALL receive a `queued` acknowledgement and the worker SHALL receive the message after its turn ends

#### Scenario: Queue full
- **WHEN** the queue exceeds its bound
- **THEN** the oldest message SHALL be dropped and its sender notified

### Requirement: Full worker reports returned to coordinator
The system SHALL return a worker's complete final report from `summary` and `await_members`, without per-line truncation below the size cap.

#### Scenario: Summary after report
- **WHEN** a worker has submitted a report and the coordinator calls `summary` for it
- **THEN** the output SHALL contain the full report body before any tool-call listing

### Requirement: Report-based completion
The system SHALL treat a member as done only when it submitted a report, and SHALL keep it done after it stops.

#### Scenario: Ready without report
- **WHEN** a member reaches `ready` without submitting a report
- **THEN** `await_members` SHALL list it as `incomplete`

#### Scenario: Reported then stopped
- **WHEN** a member submitted a report and then stopped
- **THEN** it SHALL count as done and SHALL NOT trigger a timeout wake-up

### Requirement: Resolved worker model shown
The system SHALL show each worker's resolved model and its source (`explicit`, `config`, `env`, `inherit`, `slot`) in spawn output and `swarm list`.

#### Scenario: Config default applied
- **WHEN** a worker is spawned without `model` and `agents.swarm_model` is set
- **THEN** spawn output SHALL show that model with source `config`

#### Scenario: Explicit model
- **WHEN** a worker is spawned with `model`
- **THEN** `swarm list` SHALL show that model with source `explicit`
