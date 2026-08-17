## ADDED Requirements

### Requirement: Native slash invocation
Jcode SHALL expose feature authoring as a native skill named `feature` and SHALL preserve its trailing description.

#### Scenario: User invokes feature directly
- **WHEN** a user enters `/feature add durable workflow authoring`
- **THEN** Jcode activates native `feature` with the trailing description
- **AND** does not activate a Codex- or Claude-owned implementation

### Requirement: Exploration handoff reuse
The workflow SHALL consume a valid native explore handoff and SHALL selectively refresh stale evidence instead of repeating all discovery.

#### Scenario: Handoff is current
- **WHEN** repository identity, revisions, paths, and evidence references remain current
- **THEN** the workflow seeds refinement from the handoff
- **AND** does not repeat completed evidence gathering

#### Scenario: Handoff is stale
- **WHEN** a referenced revision, path, or evidence record changed
- **THEN** the workflow refreshes the affected fields
- **AND** reports the invalidated assumptions

### Requirement: Decision-complete refinement
The workflow SHALL classify and dispose every critical uncertainty before authoring.

#### Scenario: User-only judgment remains
- **WHEN** a material decision cannot be discovered or safely defaulted
- **THEN** Jcode asks one focused question and blocks authoritative authoring until answered

### Requirement: Complete surface and case inventory
The workflow SHALL inventory affected consumers and map each material case to a requirement scenario or explicit exclusion.

#### Scenario: Existing consumers may be affected
- **WHEN** a proposed behavior changes an interface, schema, route, component, integration, or operation
- **THEN** the workflow identifies relevant consumers, compatibility behavior, touched paths, dependencies, and verification

### Requirement: Singular repository authority
The workflow SHALL write the feature contract to exactly one accepted durable authority.

#### Scenario: OpenSpec is initialized
- **WHEN** the repository declares no different authority and OpenSpec is ready
- **THEN** the workflow authors and validates an OpenSpec change

#### Scenario: Setup was declined
- **WHEN** no repository authority exists and the user declined initialization
- **THEN** the workflow may use a durable Jcode initiative plus attached design artifact
- **AND** reports that degraded authority explicitly
- **AND** does not create a duplicate task ledger

### Requirement: Observable completion contract
Every requirement SHALL include acceptance behavior, verification commands, and expected results, and every implementation task SHALL map to one or more requirements.

#### Scenario: Feature artifacts are complete
- **WHEN** authoring reaches review
- **THEN** requirement-to-task and requirement-to-check traceability is complete

### Requirement: Independent review and validation
The workflow SHALL require deterministic authority validation and independent semantic review on unchanged artifact bytes before readiness.

#### Scenario: Review changes an artifact
- **WHEN** a review finding causes any authoritative artifact mutation
- **THEN** all affected validation and semantic review evidence is invalidated and rerun

### Requirement: Harness telemetry and efficient execution
The workflow SHALL check telemetry every invocation and SHALL prefer tokenless native tools and bounded structured execution.

#### Scenario: Telemetry is unavailable
- **WHEN** telemetry cannot be emitted
- **THEN** authoring continues and reports the limitation without weakening review

#### Scenario: Shell is necessary
- **WHEN** no native tool or structured integration covers an operation
- **THEN** the workflow uses bounded batched execution and caps output at the source

### Requirement: Implementation handoff
The workflow SHALL end with one explicit implementation action and authoritative artifact references.

#### Scenario: Feature is ready
- **WHEN** refinement, authoring, validation, and review succeed
- **THEN** the workflow reports authority, artifacts, requirements, tasks, checks, dependencies, gates, and the next implementation action
