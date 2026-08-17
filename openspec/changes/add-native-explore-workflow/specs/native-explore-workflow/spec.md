## ADDED Requirements

### Requirement: Native slash invocation
Jcode SHALL expose exploration as a native skill named `explore` and SHALL preserve trailing prompt text as its topic.

#### Scenario: User invokes explore
- **WHEN** a user enters `/explore investigate command workflows`
- **THEN** Jcode activates native `explore` with the trailing topic
- **AND** does not activate a Codex- or Claude-owned implementation

### Requirement: Native evidence sequence
The workflow SHALL establish intent, plan with `todo`, retrieve relevant context, gather scoped evidence, synthesize alternatives, and select one default route.

#### Scenario: Evidence is discoverable
- **WHEN** repository tools, memory, sessions, initiatives, or Recon can answer a question
- **THEN** the workflow gathers that evidence before asking the user
- **AND** distinguishes facts, assumptions, and external claims

### Requirement: Decision-map mode
The workflow SHALL use a durable initiative when material decisions cannot be resolved in one session.

#### Scenario: Exploration remains ambiguous
- **WHEN** the destination is known but blocking decisions remain
- **THEN** the workflow creates or updates an initiative with decision milestones and checkpoints
- **AND** any side-panel map remains a non-authoritative view

### Requirement: Ranked routing
The workflow SHALL end with an ordered queue and one default route.

#### Scenario: Multiple viable outcomes exist
- **WHEN** exploration produces multiple candidates
- **THEN** the workflow ranks them using evidence and trade-offs
- **AND** identifies one default action

### Requirement: Structured feature handoff
The workflow SHALL provide a structured handoff when native `/feature` is selected.

#### Scenario: Exploration is feature-ready
- **WHEN** critical decisions are resolved
- **THEN** the handoff includes success criteria, provenance, decisions, scope, surfaces, revisions, conflicts, edge cases, done means, and limitations
- **AND** `/feature` can freshness-check it without repeating discovery

### Requirement: Harness telemetry check
The workflow SHALL check telemetry availability every invocation and emit supported events without blocking on telemetry failure.

#### Scenario: Telemetry is available
- **WHEN** the harness exposes workflow telemetry
- **THEN** exploration emits start, phase, route, efficiency, degradation, and completion observations

#### Scenario: Telemetry is unavailable
- **WHEN** telemetry is unavailable
- **THEN** exploration continues and reports that limitation truthfully

### Requirement: Token-efficient execution
The workflow SHALL prefer purpose-built tools and structured bounded output over shell execution.

#### Scenario: Typed tool exists
- **WHEN** a Jcode tool covers an evidence operation
- **THEN** the workflow uses it instead of a shell equivalent

#### Scenario: Shell is required
- **WHEN** no purpose-built tool covers a required operation
- **THEN** the workflow uses bounded execution, batching, structured output, and source-side caps
