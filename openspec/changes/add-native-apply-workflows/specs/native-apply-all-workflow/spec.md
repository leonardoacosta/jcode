## ADDED Requirements

### Requirement: Native explicit-queue invocation
Jcode SHALL expose queue execution as a native skill named `apply:all` and SHALL require an explicitly selected ordered queue.

#### Scenario: User selects a queue
- **WHEN** a user invokes `/apply:all feature-a feature-b feature-c`
- **THEN** Jcode schedules only those selected features
- **AND** does not add other open or ready work implicitly

### Requirement: Canonical queue schedule
Apply all SHALL reject missing, stale, invalid, unsupported, ambiguous, cyclic, or legacy scheduling inputs before mutation.

#### Scenario: Queue contains a dependency cycle
- **WHEN** selected features form a hard dependency cycle
- **THEN** apply all rejects the queue
- **AND** reports the cycle without dispatching work

### Requirement: Dependency- and conflict-safe waves
Apply all SHALL serialize dependencies and mutable-resource conflicts and SHALL run only proven-independent features concurrently.

#### Scenario: Features touch a shared mutable resource
- **WHEN** two selected features overlap a path, claim, repository, schema, deployment target, external system, or other mutable resource
- **THEN** they are assigned to separate ordered waves

### Requirement: Partial-progress failure handling
A failed feature SHALL pause its transitive dependents while unrelated valid branches continue.

#### Scenario: One branch fails
- **WHEN** feature B fails, feature C depends on B, and feature D is independent
- **THEN** C is paused with B as its blocker
- **AND** D remains eligible to execute
- **AND** the queue reports a partial outcome

### Requirement: Per-feature lifecycle preservation
Every queued feature SHALL complete the same implementation, verification, review, persistence, and settlement contract as native apply.

#### Scenario: A wave completes
- **WHEN** every feature attempt in a wave reaches a terminal state
- **THEN** Jcode settles each feature independently
- **AND** recomputes the ready frontier from current durable evidence

### Requirement: Queue integration gates
Apply all SHALL run queue-level integration gates only after all required branches settle and SHALL not infer success from isolated feature checks.

#### Scenario: Required branch remains paused
- **WHEN** a queue integration gate depends on a paused branch
- **THEN** the gate remains blocked
- **AND** completed unrelated features retain their truthful individual outcomes

### Requirement: Recoverable queue execution
Apply all SHALL reconstruct queue state from the frozen schedule, feature authorities, Git state, Jcode checkpoints, runtime receipts, and verification evidence.

#### Scenario: Queue resumes after interruption
- **WHEN** a queue run resumes
- **THEN** completed current features are not rerun
- **AND** uncertain attempts are reconciled before a new Dispatch is created
- **AND** stale inputs invalidate only the affected frontier and dependents
