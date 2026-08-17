## ADDED Requirements

### Requirement: Shared canonical scheduling engine
Native apply and apply all SHALL compile their selected work into the same versioned schedule and wave-plan contracts.

#### Scenario: Commands schedule equivalent work
- **WHEN** apply and apply all each schedule the same single feature at the same repository revision
- **THEN** dependency, conflict, provenance, revision, lineage, risk, and verification fields have identical semantics

### Requirement: Observable orchestration selection
The scheduler SHALL select direct, reviewed, light-swarm, deep-DAG, or durable-initiative execution from observable risk and topology and SHALL report the rationale before mutation.

#### Scenario: User requests lower rigor than policy
- **WHEN** a user requests an orchestration level below the computed minimum
- **THEN** execution requires an explicit authorized approval gate

### Requirement: Authority-preserving Orca integration
Jcode SHALL own durable intent and outcomes while Orca SHALL own only the supported runtime identities and lifecycle resources it controls.

#### Scenario: Runtime observation reports completion
- **WHEN** Orca or a worker reports a terminal runtime state
- **THEN** Jcode treats it as evidence
- **AND** advances durable outcome only after correlated acceptance receipts satisfy declared preconditions

### Requirement: Independent review evidence
Review findings SHALL identify evidence, severity, affected requirement, and a reproducible check, and SHALL be invalidated by relevant mutation.

#### Scenario: Reviewers disagree
- **WHEN** independent reviewers produce conflicting findings
- **THEN** a synthesis gate evaluates the evidence
- **AND** does not decide by simple majority vote

### Requirement: Bounded user-visible projection
The workflow SHALL keep terminal output compact and event-driven and SHALL project bounded execution state and durable evidence links in the side pane.

#### Scenario: Queue produces many runtime events
- **WHEN** event volume exceeds the bounded projection size
- **THEN** the side pane retains summarized state and stable evidence links
- **AND** model context is not filled with the complete event transcript

### Requirement: Harness telemetry and token-efficient execution
Every invocation SHALL check harness telemetry and SHALL prefer typed native tools and bounded structured execution.

#### Scenario: Telemetry is unavailable
- **WHEN** telemetry cannot be emitted
- **THEN** execution reports the limitation
- **AND** scheduling, review, verification, and settlement policy remain unchanged

#### Scenario: Shell execution is necessary
- **WHEN** no typed tool or structured integration covers an operation
- **THEN** execution uses direct bounded commands, machine-readable output, timeouts, batching, and source-side caps
