# swarm-communicate-reliability Specification

## Purpose
Defines how the swarm `communicate` tool bounds request deadlines, reports timeouts versus server errors, and keeps advertised actions aligned with dispatch.

## Requirements

### Requirement: Per-action request deadlines
The communicate tool SHALL apply a socket deadline chosen per action, using a longer deadline for mutating actions (`spawn`, `launch`, `retry`, `resume`, `stop`) than for read actions. Stop deadlines SHALL bound observation, not revoke the durable stop barrier or synthesize settlement.

#### Scenario: Slow spawn completes within extended deadline
- **WHEN** a `spawn` request takes 45 seconds to receive its response
- **THEN** the tool returns the new session id instead of a timeout error

#### Scenario: Mutating action exceeds its deadline
- **WHEN** a `spawn` request receives no response before its deadline
- **THEN** the error states the spawn may have succeeded and instructs checking `status` or `list` before retrying

#### Scenario: Stop observation exceeds deadline
- **WHEN** a stop has been accepted but its response or settlement is not observed before the client deadline
- **THEN** the result SHALL identify pending or unresolved stop state, instruct exact-ID inspection before retry, and SHALL NOT say `Stopped agent`

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
The system SHALL bind each accepted work request, completion report and completion wait to a session execution generation. A successful work completion SHALL require a report for the generation being awaited. A report SHALL remain available after that generation stops, but SHALL NOT satisfy a later generation. Stop settlement SHALL be represented separately from successful task completion, and unknown membership SHALL establish neither.

#### Scenario: Ready without report
- **WHEN** a member reaches `ready` without submitting a report for the awaited generation
- **THEN** `await_members` SHALL list it as incomplete for successful work completion

#### Scenario: Reported then stopped
- **WHEN** a member submitted a report for the awaited generation and that generation later settled stopped
- **THEN** its reported work result SHALL remain available for that generation and SHALL NOT trigger a timeout wake-up, without implying any newer assignment completed

#### Scenario: New assignment with old ready report
- **WHEN** a worker has an old completion report and accepts or queues a newer work request
- **THEN** a wait for the new request SHALL require its own generation result and SHALL NOT return the earlier ready report as completion

#### Scenario: Unknown member during stop
- **WHEN** a member disappears while its generation is awaited
- **THEN** the wait SHALL consult retained control state or report unknown/unresolved, never infer completion or quiescence solely from absence

#### Scenario: Settled cancellation without task success
- **WHEN** a wait explicitly targets stopped execution and that generation settles without a successful work report
- **THEN** the wait SHALL return stopped/cancelled with no successful-task claim

### Requirement: Resolved worker model shown
The system SHALL show each worker's resolved model and its source (`explicit`, `config`, `env`, `inherit`, `slot`) in spawn output and `swarm list`.

#### Scenario: Config default applied
- **WHEN** a worker is spawned without `model` and `agents.swarm_model` is set
- **THEN** spawn output SHALL show that model with source `config`

#### Scenario: Explicit model
- **WHEN** a worker is spawned with `model`
- **THEN** `swarm list` SHALL show that model with source `explicit`

### Requirement: Wait replay respects generation identity
Persisted waits, final-response reuse and notifications SHALL identify the requested work generation. Legacy waits without an explicit generation SHALL bind the current accepted generation when the wait begins. A persisted response for another generation SHALL NOT be replayed as fresh. Duplicate delivery SHALL NOT restart revoked work or overwrite newer report state.

#### Scenario: Cached final response from earlier run
- **WHEN** a caller repeats a wait with the same session IDs and target statuses after a new work generation is admitted
- **THEN** the old persisted response SHALL NOT satisfy the newer wait even if the member status text matches

#### Scenario: Notification arrives after stop
- **WHEN** a stale notification or completion report arrives for a revoked generation
- **THEN** it SHALL remain attributable to that generation and SHALL NOT wake it, clear stop intent or satisfy a wait for newer work

#### Scenario: Restart restores pending wait
- **WHEN** the daemon restarts with a pending generation-bound wait
- **THEN** the wait SHALL preserve its generation and cancellation bindings and SHALL NOT rebind to unrelated or newly admitted work
