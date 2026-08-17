## MODIFIED Requirements

### Requirement: Linked documentation index

The system SHALL provide a static index page that introduces the Jcode command system, links to the six approved command concept pages, links to the System Atlas, and links to the Agent Evaluation Evidence Atlas.

#### Scenario: Reader chooses a concept

- **WHEN** a reader opens the index from a local file or static server
- **THEN** the page presents links for command lifecycle, lane protocol, apply orchestration, model routing, evaluation tournament, and telemetry/results
- **AND** it presents links to `agent-stack.html` and `agent-evaluations.html`
- **AND** each link resolves without a network dependency.

### Requirement: Static System Atlas overview

The microsite SHALL provide `agent-stack.html` as a static brown field-manual translation of `docs/diagrams/agent-stack-recreation.html` with linked destinations for its platform layers, Daily-Driven Ecosystem, and Agent Evaluations.

#### Scenario: Reader explores the agent stack

- **WHEN** a reader opens the System Atlas
- **THEN** it presents surface, orchestration, context, model, tools, runtime, and memory in the source artifact's order
- **AND** every layer card links to its dedicated page
- **AND** the page contains no animation dependency, automatic motion, or remote asset
- **AND** it links to the Daily-Driven Ecosystem and Agent Evaluation Evidence Atlas pages.

#### Scenario: Atlas fidelity is validated

- **WHEN** the source comparison check runs against `docs/diagrams/agent-stack-recreation.html`
- **THEN** the Atlas contains the same seven authoritative layer names in the same order
- **AND** every extracted layer maps to one card, dedicated page, and source record
- **AND** the new Atlas contains no anime.js or remote runtime dependency.

### Requirement: Traceable source inventory

The microsite SHALL include `sources.json` as the definitive implementation source inventory and content matrix, including the Agent Evaluation Evidence Atlas and its manifest projection.

#### Scenario: Documentation evidence is audited

- **WHEN** a page section, control, diagram, chart, illustration, snippet, caveat, status label, claim, chronology event, daily-use example, finding, evaluation record, or evidence link is inspected
- **THEN** `sources.json` identifies its page element, claim text, evidence class, source artifact, source revision, confidence, and implementation status
- **AND** `agent-evals.json` identifies the referenced evaluation entity where applicable
- **AND** the validator fails with a stable requirement ID when required source coverage is absent or stale.
