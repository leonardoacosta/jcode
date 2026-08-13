# command-system-docs Specification

## Purpose
TBD - created by archiving change add-command-system-documentation-microsite. Update Purpose after archive.
## Requirements
### Requirement: Linked documentation index

The system SHALL provide a static index page that introduces the Jcode command system, links to the six approved command concept pages, and links to the System Atlas.

#### Scenario: Reader chooses a concept

- **WHEN** a reader opens the index from a local file or static server
- **THEN** the page presents links for command lifecycle, lane protocol, apply orchestration, model routing, evaluation tournament, and telemetry/results
- **AND** it presents a link to `agent-stack.html`
- **AND** each link resolves without a network dependency

### Requirement: Static System Atlas overview

The microsite SHALL provide `agent-stack.html` as a static brown field-manual translation of `docs/diagrams/agent-stack-recreation.html`.

#### Scenario: Reader explores the agent stack

- **WHEN** a reader opens the System Atlas
- **THEN** it presents surface, orchestration, context, model, tools, runtime, and memory in the source artifact's order
- **AND** every layer card links to its dedicated page
- **AND** the page contains no animation dependency, automatic motion, or remote asset
- **AND** it links to the Daily-Driven Ecosystem page

#### Scenario: Atlas fidelity is validated

- **WHEN** the source comparison check runs against `docs/diagrams/agent-stack-recreation.html`
- **THEN** the Atlas contains the same seven authoritative layer names in the same order
- **AND** every extracted layer maps to one card, dedicated page, and source record
- **AND** the new Atlas contains no anime.js or remote runtime dependency

### Requirement: Linked agent-stack layer pages

The microsite SHALL provide dedicated pages for surface, orchestration, context, model, tools, runtime, and memory.

#### Scenario: Reader opens a layer

- **WHEN** a reader follows a System Atlas card
- **THEN** the page explains what the layer does and its current architecture
- **AND** it includes a dated evidence-backed evolution chronology
- **AND** it explains how the user daily-drives relevant Claude Code, Codex, Pi, Jcode, and cross-agent capabilities
- **AND** it documents interfaces, ownership, failure boundaries, and related layers
- **AND** it provides previous, next, atlas, and cross-layer navigation

### Requirement: Daily-Driven Ecosystem page

The microsite SHALL provide `daily-driven-ecosystem.html` comparing Claude Code, Codex, Pi, Jcode, and cross-provider agents.

#### Scenario: Reader compares harness roles

- **WHEN** a reader opens the ecosystem page
- **THEN** each harness card describes its role, observed usage, strengths, friction, and cooperation boundaries
- **AND** every card links to evidence supporting its material claims
- **AND** the page does not declare a universal harness or model winner

### Requirement: Evidence classification

Every material microsite claim SHALL be classified as `measured`, `documented`, or `inferred` in `sources.json` and visibly distinguished where the class affects interpretation.

#### Scenario: Reader audits a workflow claim

- **WHEN** the claim is measured
- **THEN** it cites a telemetry record or receipt and preserves its observation window and limitations
- **WHEN** the claim is documented
- **THEN** it cites a repository-authoritative artifact and recorded revision
- **WHEN** the claim is inferred
- **THEN** it cites supporting evidence, records confidence, and is not phrased as established fact

### Requirement: Frozen and redacted ecosystem evidence

The microsite SHALL persist historian findings in `ecosystem-evidence.json` rather than depending on mutable live session stores during normal acceptance.

#### Scenario: Historian evidence is frozen

- **WHEN** implementation consolidates Claude Code, Codex, Pi, Jcode, and cross-agent findings
- **THEN** each retained claim records a stable claim ID, harness, observation window, evidence class, confidence, sanitized source reference, and source digest when locally readable
- **AND** raw credentials, access tokens, unrelated private prompts, and third-party personal data are excluded
- **AND** `sources.json` records the frozen snapshot digest

### Requirement: Consistent concept-page structure

Each command concept and agent-stack layer page SHALL provide a chapter introduction, illustration, explanatory prose, at least one diagram with a text fallback, an appropriate code or data snippet, failure or boundary notes, a claim-level evidence map, and previous/next navigation.

#### Scenario: Reader opens any chapter

- **WHEN** a reader follows a chapter link
- **THEN** the chapter uses the shared visual system and navigation
- **AND** its claims identify repository evidence
- **AND** the reader can continue forward, backward, or return to the index

### Requirement: Brown technical-field-manual design

The microsite SHALL use the approved parchment, walnut, umber, espresso, and muted-copper palette with editorial typography, compact monospaced labels, ruled details, and restrained line illustrations.

#### Scenario: Reader compares pages

- **WHEN** the index and concept pages are viewed together
- **THEN** they present one coherent brown field-manual identity
- **AND** text and controls retain accessible contrast and visible focus states

### Requirement: Offline diagrams and illustrations

The microsite SHALL not require remote fonts, scripts, stylesheets, images, Mermaid services, or other network assets.

#### Scenario: Network is unavailable

- **WHEN** the microsite is opened with network access disabled
- **THEN** all pages, navigation, illustrations, code snippets, and diagram explanations remain available
- **AND** every diagram has a readable non-script fallback

### Requirement: Truthful command and orchestration documentation

The command lifecycle, lane protocol, and apply orchestration chapters SHALL distinguish approved designs, implemented behavior, authority boundaries, and unavailable integrations.

#### Scenario: A workflow is not fully shipped

- **WHEN** a chapter describes a proposed or partially implemented command
- **THEN** it labels the current status and source change
- **AND** it does not present the design as production-complete behavior

#### Scenario: Reader reviews the lane protocol

- **WHEN** the lane protocol page describes thread counts, lane labels, or project-name prefixes
- **THEN** it identifies this OpenSpec change as the approved protocol authority
- **AND** it labels the syntax as temporary conversation behavior rather than a shipped Jcode command

### Requirement: Truthful model-routing and evaluation documentation

The model-routing, tournament, and telemetry chapters SHALL preserve safety gates, provider accounting caveats, judge disagreement, and the human routing-approval boundary.

#### Scenario: Reader reviews the OAuth smoke result

- **WHEN** the telemetry/results page presents the 2026-08-12 OAuth smoke run
- **THEN** it identifies that both candidates passed one frozen fixture
- **AND** it presents tokens and timings as provider-native evidence with stated limitations
- **AND** it reports the split judge preference and that production routing was not mutated
- **AND** it states that routing changes require a separate human approval action

### Requirement: Responsive and accessible navigation

The microsite SHALL remain readable and operable on desktop and a 393x852 mobile viewport using keyboard navigation and semantic landmarks.

#### Scenario: Mobile keyboard reader navigates the site

- **WHEN** the viewport is 393x852 and navigation is operated by keyboard
- **THEN** no page introduces horizontal overflow
- **AND** focus remains visible
- **AND** the chapter menu, breadcrumbs, and previous/next links remain reachable
- **AND** normal text meets WCAG AA 4.5:1 contrast while large text and non-text interactive indicators meet 3:1

### Requirement: Deterministic microsite validation

The repository SHALL provide a non-interactive validation command covering page inventory, HTML structure, local assets, internal links, required content blocks, evidence maps, and offline constraints.

#### Scenario: Microsite is ready for review

- **WHEN** the validation command runs
- **THEN** it exits successfully only if all sixteen pages and shared assets satisfy the contract
- **AND** failures identify the affected page and stable requirement ID

#### Scenario: Evidence or rendered semantics drift

- **WHEN** a source revision changes, a claim lacks a mapped page element, telemetry differs from committed JSON, or SVG, Mermaid, and fallback descriptions disagree materially
- **THEN** validation fails with the affected page, element, and stable requirement ID

#### Scenario: Rendered acceptance is exercised

- **WHEN** the acceptance workflow runs in a real browser
- **THEN** it visits every command chapter, System Atlas card, layer page, and ecosystem card
- **AND** it checks desktop and 393x852 rendering, keyboard focus, no-JavaScript behavior, local-only assets, and horizontal overflow
- **AND** it computes every used contrast pair rather than relying on token presence

#### Scenario: Stable validator diagnostics are emitted

- **WHEN** any static, evidence, diagram, telemetry, offline, accessibility, truthfulness, Atlas, layer, ecosystem, or navigation assertion fails
- **THEN** the diagnostic uses one of `DOCS-INDEX`, `DOCS-ATLAS`, `DOCS-LAYER`, `DOCS-ECOSYSTEM`, `DOCS-EVIDENCE`, `DOCS-DIAGRAM`, `DOCS-TELEMETRY`, `DOCS-OFFLINE`, `DOCS-A11Y`, or `DOCS-TRUTH`
- **AND** it identifies the affected page plus element or source key

### Requirement: Traceable source inventory

The microsite SHALL include `sources.json` as the definitive implementation source inventory and content matrix.

#### Scenario: Documentation evidence is audited

- **WHEN** a page section, diagram, illustration, snippet, caveat, status label, claim, chronology event, daily-use example, or evidence link is inspected
- **THEN** `sources.json` identifies its page element, claim text, evidence class, source artifact, source revision, confidence, and implementation status
- **AND** the validator fails with a requirement ID when required source coverage is absent or stale

