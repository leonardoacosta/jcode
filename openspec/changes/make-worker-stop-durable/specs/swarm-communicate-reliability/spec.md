# Swarm communicate reliability delta

## MODIFIED Requirements

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

## ADDED Requirements

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
