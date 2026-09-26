## Purpose

Lets maintainers compare local repository retrieval methods on fixed questions, so they can identify relevance, cost, latency, and freshness trade-offs before changing coding-agent navigation.

## ADDED Requirements

### Requirement: Reproducible local retrieval comparison
The evaluation SHALL run a fixed, version-controlled question set against the same repository revision and report results independently for each configured retrieval strategy.

#### Scenario: Compare configured strategies
- **WHEN** the evaluator runs with required local tools available
- **THEN** it records one result per question and strategy, including strategy name, repository revision, query, ranked paths and line references when available, elapsed time, output size, and command status

#### Scenario: Preserve reproducibility metadata
- **WHEN** an evaluation completes
- **THEN** its summary identifies the source revision, Graft version, evaluator version, strategy configuration, and fixture-set version

### Requirement: Measure relevant retrieval quality
The evaluation SHALL compare retrieved paths and source spans against explicit per-question expected evidence, reporting path-level precision and recall separately from exact-span or symbol-hit measures when those labels exist.

#### Scenario: Score judged expected files
- **WHEN** a question has one or more expected repository paths
- **THEN** each strategy's output is scored for expected-path recall and irrelevant-path precision using a documented top-k cutoff

#### Scenario: Avoid unsupported correctness claims
- **WHEN** a result lacks gold line or symbol labels
- **THEN** the report marks span/symbol scoring unavailable rather than inferring correctness from retrieval rank alone

### Requirement: Report efficiency and operational validity
The evaluation SHALL report retrieval latency, emitted output size, errors, and Graft freshness for each run, without silently dropping failed queries or stale-index evidence.

#### Scenario: A retrieval command fails
- **WHEN** a strategy exits unsuccessfully, times out, or returns malformed output
- **THEN** the result records the failure and the summary excludes it from quality averages while reporting the failed count

#### Scenario: Graft graph is missing or stale
- **WHEN** a Graft strategy requires an absent or stale graph
- **THEN** the evaluator reports that prerequisite and stops or marks that strategy unavailable; it SHALL NOT build or rewrite the graph unless the operator explicitly requests setup

### Requirement: Keep evaluation local and non-mutating by default
The evaluation SHALL run without sending repository contents or queries to hosted services, changing source files, or writing generated graphs/results into tracked paths by default.

#### Scenario: Default local run
- **WHEN** the evaluator runs with defaults
- **THEN** it uses installed local tools only, writes transient outputs to a temporary or ignored evaluation directory, and leaves tracked files and repository configuration unchanged

#### Scenario: Explicit graph preparation
- **WHEN** the operator explicitly requests graph preparation
- **THEN** the runner states the target graph directory and any repository side effects before invoking `graft build`, and supports an isolated graph directory where the CLI allows it
