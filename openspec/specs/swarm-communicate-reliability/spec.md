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
