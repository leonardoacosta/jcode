# mcp-task-tracking Specification

## Purpose
Durable tracking of long-running MCP tool calls through the 2026-07-28 Tasks extension (`io.modelcontextprotocol/tasks`). Opt-in comes only from global configuration and only for shared servers. A task handle is persisted on the existing background-task record before it is shown, polled with `tasks/get` under bounded backoff, resumed after restart without replaying `tools/call`, and cancelled only through the server. `notifications/tasks` is a wake hint, never state. Saved tasks are bound to the server identity they were created on. Server text is untrusted and bounded.

## Requirements

### Requirement: Negotiate the MCP Tasks extension
The MCP client SHALL advertise support for `io.modelcontextprotocol/tasks` in its per-request client capabilities and SHALL only interpret task handles when the server advertises the extension in `server/discover`. A task handle received without negotiated extension support or for an unsupported request SHALL be treated as a protocol error. A server without task support SHALL continue through the ordinary synchronous `tools/call` path.

#### Scenario: Supported tool returns a task handle
- **WHEN** a server advertises `io.modelcontextprotocol/tasks` and returns a valid `resultType: "task"` for `tools/call`
- **THEN** Jcode SHALL persist the task handle and return control with a visible tracked task
- **AND** Jcode SHALL NOT replay the original `tools/call` to obtain the result

#### Scenario: Server does not support task tracking
- **WHEN** the server does not advertise the Tasks extension
- **THEN** Jcode SHALL issue and handle `tools/call` synchronously
- **AND** Jcode SHALL NOT send task polling or cancellation requests to that server

#### Scenario: Task handle without negotiated support
- **WHEN** the server returns a task handle without advertising the Tasks extension or in response to an unsupported request type
- **THEN** Jcode SHALL reject the response as a protocol error
- **AND** it SHALL not poll, cancel, or retrieve results for that handle

### Requirement: Persist and resume task tracking
Jcode SHALL durably associate each MCP task with its configured server identity, authorization context reference, task ID, originating session and tool call, received task state, creation/update times, expiry, and eventual result or error. Persisted task records SHALL contain no copied credential values. After reconnect or daemon restart, Jcode SHALL resume `tasks/get` for nonterminal, unexpired tasks using the original server and authorization context. Jcode SHALL never reissue the originating `tools/call` as recovery.

#### Scenario: Daemon restarts while task is working
- **WHEN** Jcode restarts with a persisted task that has not expired and is nonterminal
- **THEN** Jcode SHALL reconnect to the same configured server under the same authorization context and query `tasks/get`
- **AND** the task SHALL remain visible with its refreshed remote status

#### Scenario: MCP server restarts and no longer has the task
- **WHEN** a tracked task's server restarts and `tasks/get` reports that the task ID is unknown or expired
- **THEN** Jcode SHALL mark the remote task unavailable while retaining its last known state locally
- **AND** it SHALL not retry the originating `tools/call` or claim that the server resumed the task

#### Scenario: Reconnect targets a different server identity
- **WHEN** the saved server name resolves to a different endpoint or authorization context than the task record
- **THEN** Jcode SHALL stop automatic polling and show the task as unresolved
- **AND** Jcode SHALL NOT send the saved task ID or result request to the different server

### Requirement: Reconcile status with polling
Jcode SHALL treat `notifications/tasks` as optional hints, not authoritative completion evidence. It SHALL refresh status with `tasks/get` after a notification and poll nonterminal tasks when notifications are absent or lost. Polling SHALL respect `pollIntervalMs` when supplied, apply bounded backoff after transport errors, and avoid polling after a terminal state or known expiry.

#### Scenario: Status notification is lost
- **WHEN** a tracked task remains nonterminal and no `notifications/tasks` update arrives
- **THEN** Jcode SHALL poll `tasks/get` no sooner than the suggested interval when supplied
- **AND** it SHALL update the visible state from the returned task

#### Scenario: Notification arrives with stale status
- **WHEN** a task notification conflicts with the latest persisted state or is older than the stored update
- **THEN** Jcode SHALL treat it as a wake hint and query `tasks/get`
- **AND** it SHALL not regress a newer persisted state based only on that notification

#### Scenario: Poll endpoint is temporarily unavailable
- **WHEN** `tasks/get` fails because the connection is interrupted
- **THEN** Jcode SHALL retain the last known task state and schedule a bounded-backoff retry
- **AND** it SHALL show that remote status is unavailable rather than report completion or failure

### Requirement: Retrieve terminal results and expose task state
Jcode SHALL use the current `tasks/get` task state to detect terminal status and retain any inlined completed result or failed error. Where the extension requires a separate result retrieval operation, Jcode SHALL use only the method defined by the negotiated extension revision. The existing background-task UI SHALL identify MCP tasks, show status and available result/error, and distinguish `working`, `input_required`, `completed`, `failed`, `cancelled`, `expired`, and unresolved connection states.

#### Scenario: Task completes
- **WHEN** `tasks/get` reports `completed` with a result
- **THEN** Jcode SHALL persist and display the result under the originating tracked task
- **AND** it SHALL stop status polling

#### Scenario: Task requires input
- **WHEN** `tasks/get` reports `input_required`
- **THEN** Jcode SHALL preserve and surface the outstanding input request in the existing task surface
- **AND** it SHALL not invent or submit a response without the applicable user/model interaction

#### Scenario: Task fails
- **WHEN** `tasks/get` reports `failed` with an error
- **THEN** Jcode SHALL preserve and display that error as the task outcome
- **AND** it SHALL not convert the failure into a successful tool result

### Requirement: Cancel with race-safe outcomes
Jcode SHALL send `tasks/cancel` only for a tracked nonterminal task to its original server and authorization context. It SHALL reconcile the response with `tasks/get` or a subsequent status notification, preserving the server's terminal state. The UI SHALL distinguish cancellation requested from confirmed `cancelled` state.

#### Scenario: Server confirms cancellation
- **WHEN** the server returns a task state of `cancelled` for `tasks/cancel`
- **THEN** Jcode SHALL persist and display `cancelled`
- **AND** it SHALL stop polling

#### Scenario: Completion races with cancellation
- **WHEN** the server reports `completed` or `failed` while a cancellation request is in flight
- **THEN** Jcode SHALL retain the server-reported terminal outcome
- **AND** the UI SHALL not claim that cancellation stopped the operation

#### Scenario: Cancellation request fails
- **WHEN** `tasks/cancel` fails or the response is lost
- **THEN** Jcode SHALL keep cancellation unconfirmed and poll the same task when reachable
- **AND** it SHALL not locally mark the remote task `cancelled`

### Requirement: Expire task records safely
Jcode SHALL track the server-provided `ttlMs` from `createdAt` and stop remote retrieval after expiry. Expiry SHALL be visible and distinct from a remotely reported terminal state. Jcode MAY retain a minimal local history record after expiry, but SHALL not represent an unavailable remote result as retrieved.

#### Scenario: Task expires before result retrieval
- **WHEN** a task reaches its TTL before Jcode retrieves its terminal result
- **THEN** Jcode SHALL mark the remote task expired/unavailable and stop remote polling
- **AND** preserve its last known status and task ID in local history

#### Scenario: Server reports an expired task as not found
- **WHEN** `tasks/get` returns the protocol's not-found/expired error for a tracked task
- **THEN** Jcode SHALL display an expired/unavailable outcome
- **AND** it SHALL not retry indefinitely or treat the error as task failure
