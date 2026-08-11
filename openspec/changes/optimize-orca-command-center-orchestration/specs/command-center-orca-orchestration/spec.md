## ADDED Requirements

### Requirement: Explicit orchestration pattern selection
Command Center SHALL classify each Orca-backed action as exactly one of full handoff, supervised Run/Task/Dispatch, direct terminal action, observation only, or decision gate before invoking runtime mechanics.

#### Scenario: Full ownership transfer
- **WHEN** a user asks Orca to take full ownership of a bounded outcome without Jcode supervising intermediate lifecycle
- **THEN** Command Center selects full handoff and records the durable command envelope, authorization and idempotency context, correlation, and verified final evidence without projecting supervised intermediate lifecycle

#### Scenario: Supervised dependency graph
- **WHEN** Jcode must monitor dependencies, gates, retries, or worker completion
- **THEN** Command Center selects supervised Run/Task/Dispatch rather than full handoff

#### Scenario: No silent downgrade
- **WHEN** the selected orchestration pattern is unavailable
- **THEN** Command Center reports the capability as unavailable and MUST NOT substitute a weaker pattern

### Requirement: Durable and runtime authority separation
Jcode SHALL remain authoritative for initiatives, milestones, schedules, permissions, idempotency, rollback intent, and durable outcomes, while Orca SHALL remain authoritative for canonical executable identity and live runtime state.

#### Scenario: Runtime evidence cannot settle state alone
- **WHEN** Orca reports a worker or terminal state without a verified command receipt satisfying the expected transition
- **THEN** Jcode retains the current durable outcome and exposes the observation as non-authoritative evidence

#### Scenario: Verified settlement
- **WHEN** a correlated Orca receipt satisfies the command preconditions and expected terminal state
- **THEN** Jcode records the durable outcome with the receipt provenance

### Requirement: Canonical identifier preservation
Command Center SHALL preserve separate fields for Jcode initiative and run IDs, Orca canonical repository or project ID, Orca Run ID, Task and Dispatch IDs, worktree and terminal handles, and correlation and idempotency IDs.

#### Scenario: Canonical project lookup
- **WHEN** Command Center projects Orca execution into a Jcode run
- **THEN** the project field is populated from canonical repository or project lookup rather than Orca runtime identity

#### Scenario: Runtime ID rejection
- **WHEN** an Orca runtime ID is supplied where a canonical project ID is required
- **THEN** Command Center rejects or leaves the project association unresolved and MUST NOT persist the runtime ID as project identity

### Requirement: Ordered authorization-scoped lifecycle projection
Command Center SHALL normalize Orca messages, questions, heartbeats, gates, attempts, terminal health, completion, escalation, retention, and release events as ordered evidence scoped to the authorized initiative and runtime.

#### Scenario: Scoped replay after sequence gap
- **WHEN** the client or projection layer detects a missing event sequence
- **THEN** replay is requested within the same authorization, initiative, and runtime scope before later events can settle durable state

#### Scenario: Replay authorization or retention boundary
- **WHEN** authorization changes or the requested cursor predates retained evidence
- **THEN** replay is rejected and Command Center requires a fresh authorized snapshot without exposing events from another principal, initiative, or runtime

#### Scenario: Unknown event
- **WHEN** Orca emits an event type the projection layer does not understand
- **THEN** Command Center preserves the event as visible evidence but MUST NOT mutate durable state from it

#### Scenario: Orca unavailable
- **WHEN** the Orca runtime cannot be reached
- **THEN** Command Center exposes a degraded or unavailable state without fabricating completion, cancellation, or cleanup

### Requirement: Fail-closed mutation capability boundary
Every Command Center mutation SHALL declare its authority owner, preconditions, idempotency behavior, expected success receipt, and unavailable-Orca behavior. Undocumented or unsupported Orca operations MUST return an unsupported-capability result.

#### Scenario: Unsupported retry
- **WHEN** retry is requested but the installed Orca interface exposes no verified retry operation
- **THEN** Command Center returns unsupported capability and leaves the durable run unchanged

#### Scenario: Idempotent duplicate request
- **WHEN** the same mutation is submitted again with the same idempotency ID
- **THEN** Command Center returns the existing result or safe in-progress state without launching a duplicate dispatch

#### Scenario: Crash after durable command recording
- **WHEN** Jcode restarts after recording an idempotency envelope but before storing the Orca receipt
- **THEN** recovery reconciles existing Orca evidence before any new dispatch and MUST NOT duplicate the mutation

#### Scenario: Stale precondition
- **WHEN** cancel or retry is requested against a runtime state that no longer satisfies the command precondition
- **THEN** Command Center rejects the mutation and returns the observed current state

#### Scenario: Partial cleanup failure
- **WHEN** release of a worker, terminal, or worktree only partially succeeds
- **THEN** Command Center records each verified result, marks remaining resources recovery-required, and MUST NOT report cleanup complete

#### Scenario: Runtime capability discovery
- **WHEN** Command Center connects to an Orca runtime
- **THEN** it advertises only mutation capabilities verified from that version-matched runtime and its adapter

### Requirement: Scheduled execution uses the same policy bridge
A Jcode schedule SHALL represent durable eligibility intent and SHALL invoke the same pattern selection, permission, correlation, idempotency, and receipt-settlement path used by interactive Command Center actions.

#### Scenario: Scheduled supervised run
- **WHEN** a schedule makes a supervised initiative action eligible
- **THEN** Command Center creates or resumes the correlated Orca Run/Task/Dispatch lifecycle and records the schedule trigger provenance

#### Scenario: Scheduled retry attempt
- **WHEN** retry policy creates a new dispatch attempt
- **THEN** the original schedule and Jcode run remain durable while the new dispatch receives a distinct attempt identity linked by causality

### Requirement: Three-skill responsibility boundary
The installed skill surface SHALL keep generic Orca runtime mechanics in `orca-cli`, generic supervised coordination in `orchestration`, and Jcode-specific Command Center policy in `jcode-command-center-orchestration`.

#### Scenario: Generic Orca request
- **WHEN** a user asks to manage an Orca worktree, terminal, browser, automation, or full handoff without Jcode Command Center context
- **THEN** the request routes to `orca-cli` without loading Jcode-specific policy

#### Scenario: Generic supervised coordination
- **WHEN** a user asks to coordinate Orca Runs, Tasks, Dispatches, messaging, gates, retention, release, or recovery without Jcode durable-state concerns
- **THEN** the request routes to `orchestration`

#### Scenario: Command Center lifecycle request
- **WHEN** a request involves Jcode initiatives, schedules, Command Center launch, retry, cancel, approval, handoff, or Orca lifecycle projection
- **THEN** `jcode-command-center-orchestration` provides the authority and policy bridge while loading generic Orca mechanics as needed

### Requirement: Obsolete llmtrim guidance removal
The relevant installed Orca orchestration skills SHALL contain no `llmtrim` command, dependency, fallback, or recommendation.

#### Scenario: Skill content audit
- **WHEN** the installed skill files and bundled references are searched case-insensitively for `llmtrim`
- **THEN** no matches are returned

### Requirement: Representative skill acceptance evidence
The focused policy skill SHALL include representative evaluation prompts and deterministic checks covering pattern selection, authority, identifier mapping, scheduling, replay gaps, degraded Orca state, unsupported mutation, and resource cleanup.

#### Scenario: Policy evaluation suite
- **WHEN** the skill evaluation suite runs against the focused skill and its baseline
- **THEN** every required policy assertion is graded with concrete evidence and any regression blocks completion

#### Scenario: Runtime adapter regression
- **WHEN** focused app-core Command Center tests run
- **THEN** they prove canonical project identity is not sourced from Orca runtime ID and unsupported mutations remain fail closed
