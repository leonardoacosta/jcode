## ADDED Requirements

### Requirement: Linked Agent Evaluations destination

The microsite SHALL provide `agent-evaluations.html` as a linked Agent Evaluation Evidence Atlas.

#### Scenario: Reader opens the evaluation evidence

- **WHEN** a reader follows the evaluation link from the field-manual index or System Atlas
- **THEN** the page presents a decision brief, findings ledger, run explorer, review DAG, telemetry, and evidence map
- **AND** the evaluation tournament, telemetry, and ecosystem pages link back to it
- **AND** all primary content remains available without JavaScript or network access.

### Requirement: Versioned evaluation evidence manifest

The microsite SHALL use `agent-evals.json` as the versioned rendering and validation contract for published evaluation evidence.

#### Scenario: Evaluation evidence is loaded

- **WHEN** the manifest is validated
- **THEN** it identifies its schema version, source revision, evidence digest, tracks, evaluations, runs, candidates, reviewers, findings, dispositions, telemetry records, and evidence sources
- **AND** every entity has a globally unique stable ID
- **AND** every reference resolves to an existing entity or tracked source.

#### Scenario: Manifest evidence changes

- **WHEN** a source, entity, or displayed projection changes
- **THEN** the evidence digest and affected source revision SHALL be refreshed
- **AND** prior digest-bound validation and review evidence SHALL become stale.

### Requirement: Truthful findings ledger

The page SHALL expose material evaluation and review findings with traceable status and disposition.

#### Scenario: Reader inspects a finding

- **WHEN** a finding is displayed
- **THEN** it shows its stable ID, evidence track, severity, claim status, source model/provider or deterministic source, sanitized summary, evidence pointers, disposition, implementation state, verification result, and limitations
- **AND** absent evidence is labeled `unavailable` rather than reconstructed.

#### Scenario: Reader filters findings

- **WHEN** JavaScript is available
- **THEN** the reader can filter findings by track, severity, model/provider, claim status, and disposition
- **AND** filtering does not remove the corresponding records from the no-JavaScript document.

### Requirement: Measured tournament run explorer

The page SHALL present the frozen OAuth smoke evidence without overstating its scope or comparability.

#### Scenario: Reader compares smoke candidates

- **WHEN** the OAuth smoke run is selected
- **THEN** the page identifies the frozen fixture, accepted attempts, Claude Fable 5 candidate, OpenAI GPT-5.5 candidate, deterministic baseline, anonymized outputs, judge receipts, steering evidence, cost, and limitations
- **AND** it reports that both candidates passed, judge preference split, and no production routing mutation was authorized
- **AND** it does not describe the run as a universal model ranking.

#### Scenario: Provider telemetry is compared

- **WHEN** token or timing evidence is displayed across providers
- **THEN** each value retains its provider-native metric name, unit, measurement boundary, and limitation
- **AND** unlike token classes are not presented as normalized equivalents
- **AND** unavailable queue time, provider-internal reasoning tokens, or true provider TTFT are labeled unavailable where applicable.

### Requirement: Cross-provider review DAG evidence

The page SHALL document the approved recommendation-approval DAG and distinguish planned stages from observed executions.

#### Scenario: Reader inspects the review policy

- **WHEN** the review DAG is displayed
- **THEN** it shows one-pass Fable discovery, Sol cross-provider refinement or autoresearch, human approve/reject/defer/modify authority, Luna implementation, and deterministic plus cold verification
- **AND** it states that neither provider immediately refines its own prior iteration.

#### Scenario: Reader inspects an observed remediation finding

- **WHEN** a retained microsite finding is displayed
- **THEN** the record identifies its domain, discovery source, refinement or autoresearch disposition, recorded human decision, implementation evidence when available, and cold-verification outcome
- **AND** any stage not durably observed is labeled unavailable or planned rather than completed.

### Requirement: Separated evaluation authority

The page SHALL keep evaluation output, reviewer recommendation, human disposition, implementation state, verification state, and routing authority separate.

#### Scenario: A candidate receives a higher score

- **WHEN** a candidate or finding has a favorable score or recommendation
- **THEN** the page SHALL NOT render it as an approved production-routing change
- **AND** it SHALL identify the separate human approval and routing-policy change required for promotion.

### Requirement: Safe evidence publication

The manifest and page SHALL publish only sanitized, repository-safe evaluation evidence.

#### Scenario: A source includes sensitive or private content

- **WHEN** a source contains credentials, active tokens, unrelated private prompts, third-party personal data, or an unsafe private URL
- **THEN** the source body SHALL be excluded
- **AND** the manifest MAY retain a sanitized reference and digest
- **AND** required but unpublished fields SHALL be labeled unavailable.

### Requirement: Accessible evidence visualization

The Agent Evaluations page SHALL use the brown technical-field-manual design while keeping its ledger, diagrams, charts, tables, controls, and evidence links accessible.

#### Scenario: Reader uses a narrow viewport or keyboard

- **WHEN** the page is used at 393x852 or with keyboard navigation
- **THEN** controls and links retain visible focus, tables and ledgers introduce no page-level horizontal overflow, and all content remains reachable
- **AND** charts and diagrams provide text equivalents
- **AND** used contrast pairs meet WCAG AA thresholds.

### Requirement: Deterministic evaluation Atlas validation

The repository SHALL validate manifest integrity, evidence provenance, rendered semantics, truthfulness, offline operation, and browser behavior non-interactively.

#### Scenario: Static evaluation validation passes

- **WHEN** `python3 scripts/test-command-system-docs.py` runs
- **THEN** it verifies manifest schema, unique IDs, references, source freshness, digests, claim mapping, HTML equivalence, provider-native telemetry labels, authority boundaries, internal links, and offline assets
- **AND** evaluation-specific failures use `DOCS-EVALS` with the affected page and entity or source key.

#### Scenario: Evaluation defect classes are injected

- **WHEN** `python3 scripts/test-command-system-docs.py --self-test` runs
- **THEN** it observes stable failures for duplicate IDs, dangling references, missing limitations, reconstructed unavailable evidence, false token normalization, unsupported winner claims, automatic-routing language, stale digests, unsafe source references, and HTML/manifest drift.

#### Scenario: Real reader journeys are exercised

- **WHEN** `python3 scripts/test-command-system-docs-browser.py --site docs/diagrams/jcode-command-system` runs
- **THEN** it visits the field-manual index, System Atlas, Agent Evaluations, tournament, telemetry, and ecosystem routes at desktop and 393x852 with JavaScript enabled and disabled
- **AND** it exercises enabled filters and disclosures, keyboard focus, evidence links, local-only assets, fallbacks, overflow, console errors, and network failures.

### Requirement: WS publication and notification

The accepted evaluation Atlas SHALL be integrated into the WS documentation portal and delivered through the established deployment workflow.

#### Scenario: Accepted Atlas is published

- **WHEN** all Jcode acceptance checks and fresh review pass
- **THEN** the settled files SHALL be copied into the WS documentation source without unrelated changes
- **AND** the real docs portal build SHALL pass
- **AND** the exact Azure pipeline run SHALL reach a successful terminal state
- **AND** the Entra-gated live evaluation URL and deployment receipt SHALL be recorded before the URL is sent through ntfy.
