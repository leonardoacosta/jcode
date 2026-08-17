## ADDED Requirements

### Requirement: Factory documentation index

The repository MUST provide `docs/factory/README.md` as the introduction and navigation entry point for the AI software-factory research and Jcode mapping.

#### Scenario: Reader discovers the factory model
- **WHEN** a reader opens `docs/factory/README.md`
- **THEN** the page defines the factory, shows the lifecycle from intent through feedback, distinguishes agent shell, worker runtime, workflow, and factory control plane, and links to every dedicated factory page

#### Scenario: Claims remain bounded
- **WHEN** the index describes a capability or recommendation
- **THEN** the claim is labeled observed, proposed, external research, or open question and includes a repository or public-source pointer

### Requirement: Dedicated lifecycle and cross-cutting pages

The repository MUST provide dedicated Markdown pages for lifecycle, architecture, artifacts and provenance, workers and orchestration, isolation and execution, gates and approvals, evaluation and regression, observability, governance and risk, feedback and learning, open-harness landscape, Jcode mapping, and sources and limitations.

#### Scenario: Reader follows a lifecycle stage
- **WHEN** a reader selects a lifecycle stage from the index
- **THEN** the linked page explains the stage's purpose, inputs, outputs, controls, evidence, and relationship to adjacent stages

#### Scenario: Reader investigates a cross-cutting concern
- **WHEN** a reader selects a cross-cutting topic such as evaluation, observability, or governance
- **THEN** the dedicated page explains the concern, relevant external findings, current Jcode evidence, proposed direction, and limitations

### Requirement: Repository documentation integration

The repository MUST link the factory index from the existing documentation entry point without changing runtime behavior or altering unrelated active work.

#### Scenario: Documentation entry point exposes the factory
- **WHEN** a reader opens `docs/README.md`
- **THEN** the layout or key-entry-point section links to `docs/factory/README.md`

#### Scenario: Documentation-only boundary is preserved
- **WHEN** the documentation change is reviewed
- **THEN** no runtime source, provider configuration, credentials, private transcripts, or unrelated OpenSpec change is modified

### Requirement: Factory documentation validation

The documentation MUST be validated for navigation, evidence labeling, source references, and documentation integrity.

#### Scenario: Internal navigation is checked
- **WHEN** the documentation validation runs
- **THEN** every factory page link resolves, every referenced Jcode path exists, and `git diff --check` passes

#### Scenario: Reader path is reviewable
- **WHEN** an independent reviewer starts at `docs/README.md`
- **THEN** they can reach the factory index, lifecycle page, and each cross-cutting page without relying on conversation history or private evidence
