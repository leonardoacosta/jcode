## ADDED Requirements

### Requirement: Linked documentation index

The system SHALL provide a static index page that introduces the Jcode command system and links to the six approved concept pages.

#### Scenario: Reader chooses a concept

- **WHEN** a reader opens the index from a local file or static server
- **THEN** the page presents links for command lifecycle, lane protocol, apply orchestration, model routing, evaluation tournament, and telemetry/results
- **AND** each link resolves without a network dependency

### Requirement: Consistent concept-page structure

Each concept page SHALL provide a chapter introduction, illustration, explanatory prose, at least one diagram with a text fallback, an appropriate code or data snippet, failure or boundary notes, an evidence map, and previous/next navigation.

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
- **THEN** it exits successfully only if all seven pages and shared assets satisfy the contract
- **AND** failures identify the affected page and stable requirement ID

### Requirement: Traceable source inventory

The microsite SHALL include `sources.json` as the definitive implementation source inventory and content matrix.

#### Scenario: Documentation evidence is audited

- **WHEN** a page section, diagram, illustration, snippet, caveat, status label, or evidence link is inspected
- **THEN** `sources.json` identifies its source artifact or this change as its approved authority
- **AND** the validator fails with a requirement ID when required source coverage is absent or stale
