## ADDED Requirements

### Requirement: Explicit recurring schedule definition

The existing `schedule` tool SHALL create recurring schedules with a stable schedule ID, a valid cron expression, an explicit IANA time-zone identifier, a task payload compatible with existing one-shot schedules, a missed-run policy, and optional limits. It SHALL reject invalid expressions, unknown time zones, unsupported fields, and non-positive limits without changing persisted state. A schedule SHALL be evaluated by local calendar time in its configured zone, not by assuming a fixed UTC interval.

#### Scenario: Create recurring schedule

- **WHEN** a caller creates a schedule with a valid cron expression, IANA zone, task, and explicit missed-run policy
- **THEN** the existing ambient schedule store SHALL persist the definition and its next valid occurrence
- **AND** the existing ambient runner SHALL be nudged to recalculate its next wake

#### Scenario: Invalid definition is rejected

- **WHEN** a caller submits an invalid cron expression, an unknown IANA zone, or a zero/negative configured limit
- **THEN** the tool SHALL return a validation error
- **AND** the schedule store SHALL remain unchanged

### Requirement: Deterministic daylight-saving behavior

Cron occurrences SHALL be derived from local calendar timestamps and resolved using the stored IANA zone. A nonexistent local time in a daylight-saving gap SHALL be skipped. An ambiguous local time in a daylight-saving overlap SHALL resolve to the earlier UTC instant and SHALL run at most once for that local timestamp. The schedule SHALL retain its configured zone and SHALL NOT silently rewrite it to UTC or the host zone.

#### Scenario: Spring-forward gap

- **WHEN** a cron expression matches a local wall time that does not exist because clocks move forward
- **THEN** that local occurrence SHALL be skipped
- **AND** the next valid local occurrence SHALL be computed without shifting the missing occurrence to another wall time

#### Scenario: Fall-back overlap

- **WHEN** a cron expression matches an ambiguous local wall time during the backward clock transition
- **THEN** exactly one occurrence SHALL be created at the earlier UTC instant
- **AND** the second rendering of that local time SHALL NOT create another occurrence

#### Scenario: Zone rules change

- **WHEN** the system time-zone database changes while a schedule is persisted
- **THEN** future, unclaimed occurrences SHALL be recalculated using the current rules for the stored zone
- **AND** persisted occurrence keys and terminal outcomes SHALL remain unchanged

### Requirement: Occurrence identity and clock movement

Each occurrence SHALL have a stable idempotency key derived from the schedule ID and its resolved local calendar timestamp and zone. The key SHALL be persisted before dispatch and reused on restart or retry of the same occurrence. The scheduler SHALL persist the last evaluated occurrence boundary and SHALL NOT create a second occurrence key for a previously recorded local occurrence. A forward clock jump SHALL apply the schedule's missed-run policy to all newly due occurrences. A backward clock jump SHALL NOT re-enqueue occurrences already recorded as pending, dispatched, terminal, or skipped.

#### Scenario: Forward clock jump

- **WHEN** the wall clock advances past several scheduled instants before the next scheduler pass
- **THEN** every newly missed occurrence SHALL be resolved under the schedule's missed-run policy
- **AND** the system SHALL persist the decision before making any occurrence runnable

#### Scenario: Backward clock jump

- **WHEN** the wall clock moves backward across an already-recorded occurrence
- **THEN** the scheduler SHALL NOT enqueue that occurrence again
- **AND** future occurrences SHALL continue in cron order

#### Scenario: Restart at an occurrence boundary

- **WHEN** the server restarts after recording an occurrence but before or during delivery
- **THEN** it SHALL retain the same occurrence idempotency key and recovery state
- **AND** it SHALL NOT create a duplicate occurrence for that boundary

### Requirement: Missed-run policies

Each recurring schedule SHALL select exactly one policy: `skip`, `run-once`, or bounded `catch-up`. `skip` SHALL record all missed occurrences as skipped and enqueue none. `run-once` SHALL coalesce all missed occurrences into the latest missed occurrence and enqueue it once while recording the earlier occurrences as coalesced. `catch-up` SHALL enqueue the oldest missed occurrences up to the configured positive per-schedule catch-up limit and record the remainder as skipped. The policy SHALL be applied after startup downtime and clock jumps, and its decisions SHALL be persisted before dispatch.

#### Scenario: Skip missed occurrences

- **WHEN** a schedule with policy `skip` has multiple occurrences become due while the server is unavailable
- **THEN** the system SHALL record each occurrence as skipped
- **AND** it SHALL enqueue no missed run

#### Scenario: Run once after downtime

- **WHEN** a schedule with policy `run-once` has multiple occurrences become due while the server is unavailable
- **THEN** the system SHALL enqueue only the latest missed occurrence
- **AND** it SHALL record earlier missed occurrences as coalesced without replaying them

#### Scenario: Bounded catch-up after downtime

- **WHEN** more occurrences are missed than the configured catch-up limit
- **THEN** the system SHALL enqueue the oldest missed occurrences up to the limit
- **AND** it SHALL record all remaining missed occurrences as skipped

### Requirement: Bounded dispatch and concurrency

A recurring schedule SHALL support a positive per-schedule maximum active-run count, and the scheduler SHALL enforce a configured global maximum active scheduled-run count across all recurring schedules through the existing ambient runner. A full limit SHALL leave due work durably queued, preserve occurrence identity, and make no new execution attempt until capacity is available. Paused schedules SHALL admit no new occurrences for dispatch; already-running occurrences SHALL follow the existing cancellation contract and SHALL NOT be represented as stopped merely because the schedule was paused.

#### Scenario: Per-schedule concurrency is full

- **WHEN** a due occurrence is ready and that schedule already has its maximum number of active runs
- **THEN** the occurrence SHALL remain durably pending
- **AND** no additional run for that schedule SHALL start until capacity is released

#### Scenario: Global concurrency is full

- **WHEN** the global scheduled-run limit is reached
- **THEN** due occurrences SHALL remain durably pending and retain their keys
- **AND** the runner SHALL admit them when an active run settles

#### Scenario: Pause with an active run

- **WHEN** a user pauses a schedule that has an active occurrence
- **THEN** the schedule SHALL stop admitting future occurrences
- **AND** the active occurrence SHALL retain its existing running/cancellation state until it settles

### Requirement: Schedule controls and preview

The existing schedule tool surface SHALL support listing schedules and recent occurrence outcomes, pausing and resuming by stable schedule ID, and previewing the next N occurrences without mutating schedule or queue state. List output SHALL show schedule ID, cron expression, time zone, next occurrence, pause state, missed-run policy, concurrency and budget limits, and recent occurrence outcomes. Resume SHALL calculate the next eligible occurrence using persisted occurrence history and the configured missed-run policy. A preview SHALL use the same cron and time-zone calculation as dispatch.

#### Scenario: Pause and resume

- **WHEN** the user pauses and later resumes a schedule by its ID
- **THEN** the pause state SHALL survive restart
- **AND** resume SHALL apply the selected missed-run policy to occurrences due during the pause without duplicating recorded occurrences

#### Scenario: Preview without mutation

- **WHEN** the user requests a preview for a valid schedule
- **THEN** the tool SHALL return the requested upcoming local times, zones, and resolved UTC instants
- **AND** the persisted schedule and occurrence queue SHALL remain unchanged

#### Scenario: List after restart

- **WHEN** the server restarts with saved recurring schedules and terminal occurrence history
- **THEN** `schedule list` SHALL show the same schedule IDs, control state, next-run information, and recorded outcomes

### Requirement: Durable occurrence delivery and crash recovery

Recurring occurrences SHALL use the durable claim, acknowledgment, uncertain-delivery, and history contract defined by `scheduled-run-recovery`. A schedule occurrence SHALL remain associated with its occurrence key through queue delivery and terminal history. A crash after dispatch without acknowledgment SHALL produce the recovery contract's uncertain state and SHALL NOT cause automatic redelivery. An acknowledged delivery SHALL NOT be presented as proof that the task completed successfully.

#### Scenario: Crash mid-run after delivery acknowledgment

- **WHEN** a recurring occurrence is delivered and acknowledged, then the server crashes before the task reports completion
- **THEN** the occurrence SHALL retain its acknowledged delivery identity and existing execution-recovery state
- **AND** the scheduler SHALL NOT create or automatically deliver another occurrence with the same key

#### Scenario: Crash before delivery acknowledgment

- **WHEN** the server restarts with a dispatched recurring occurrence lacking acknowledgment
- **THEN** the occurrence SHALL be marked uncertain under `scheduled-run-recovery`
- **AND** it SHALL require the existing explicit retry path rather than automatic redelivery

### Requirement: Per-schedule resource and spend budgets

A schedule MAY set token and monetary spend limits, and SHALL persist the limits with its definition. Scheduled execution SHALL use the existing overnight run ownership, cancellation, usage reconciliation, and authoritative budget enforcement mechanisms where available. Before admitting a run, the system SHALL reserve or otherwise enforce the remaining configured budget using authoritative provider/usage data. When the budget is exhausted or cannot be authoritatively enforced for the selected provider, the system SHALL not start another run and SHALL expose a `budget_exhausted` or `budget_unavailable` outcome. Estimated or post-request-only usage SHALL NOT be presented as a hard monetary cap. Budget use and reservation settlement SHALL be attributed to the schedule occurrence key and deduplicated across restart and retry.

#### Scenario: Budget exhausted before dispatch

- **WHEN** a due occurrence is ready but the schedule's remaining enforceable budget is zero
- **THEN** the occurrence SHALL not start
- **AND** the schedule list SHALL show `budget_exhausted` with the remaining amount and limit

#### Scenario: Provider cannot enforce configured spend cap

- **WHEN** a schedule has a monetary cap but the selected provider has no authoritative reservation or enforcement path
- **THEN** the occurrence SHALL not start
- **AND** the system SHALL show `budget_unavailable` rather than silently treating an estimate as a hard cap

#### Scenario: Budget exhausted during an active run

- **WHEN** authoritative usage reaches the configured token or monetary limit during an active occurrence
- **THEN** the existing run-limit and cancellation mechanisms SHALL stop further chargeable work
- **AND** the occurrence SHALL record the limit outcome without claiming settlement until owned work is confirmed stopped

#### Scenario: Crash during budgeted run

- **WHEN** the server crashes after a budget reservation but before recording run settlement
- **THEN** restart reconciliation SHALL associate the reservation with the same occurrence key
- **AND** it SHALL neither double-reserve nor release spend that may already have occurred

#### Scenario: Duplicate usage event

- **WHEN** a provider usage event is replayed for the same schedule occurrence after retry or restart
- **THEN** budget accounting SHALL count the event once by its stable ownership/event identity
- **AND** the remaining schedule budget SHALL remain consistent with the deduplicated usage
