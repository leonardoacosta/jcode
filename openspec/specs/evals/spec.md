# evals Specification

## Purpose

Define the repository's evaluation surfaces for measuring Jcode planning and task-decomposition behavior against historical OpenSpec proposal work.
## Requirements
### Requirement: Task decomposition fixture catalog

The project SHALL provide a task-decomposition eval fixture catalog that records historical OpenSpec proposal situations by category without embedding absolute local source paths.

#### Scenario: Validate fixture metadata offline

- **GIVEN** the checked-in fixture catalog
- **WHEN** the eval validation command is run without repository root mappings
- **THEN** it SHALL validate required fields, unique fixture IDs, commit hash shapes, expected artifact declarations, and category coverage using only repository files and Python standard library modules.

### Requirement: Fixture materialization from local repositories

The project SHALL support materializing a fixture checkout at the recorded base commit from an operator-supplied local repository root.

#### Scenario: Materialize a base checkout

- **GIVEN** a fixture ID, an output directory that does not already exist, and a `project=/path/to/repo` mapping for the fixture project
- **WHEN** the materialization command is run
- **THEN** it SHALL clone from the supplied repository with shared objects, check out the fixture base commit, and write fixture metadata into the checkout.

### Requirement: Gold artifact scoring

The project SHALL support deterministic scoring of candidate OpenSpec artifacts against the fixture's gold proposal commit.

#### Scenario: Score candidate artifacts

- **GIVEN** a fixture ID, a candidate OpenSpec change directory, and a `project=/path/to/repo` mapping for the fixture project
- **WHEN** the scoring command is run
- **THEN** it SHALL compare required artifact presence and token overlap against the gold change artifacts and emit a JSON score report.

### Requirement: Task decomposition prompt metadata

The project SHALL provide prompt metadata for task-decomposition eval fixtures separately from fixture repository metadata.

#### Scenario: Validate prompt metadata offline

- **GIVEN** the checked-in prompt catalog and fixture catalog
- **WHEN** the prompt validation command is run
- **THEN** it SHALL validate prompt fixture IDs, prompt kind, confidence, source, prompt text, notes, and duplicate coverage using only repository files and Python standard library modules.

### Requirement: Task decomposition run preparation

The project SHALL support validating a task-decomposition eval run plan without materializing a checkout or invoking a model.

#### Scenario: Prepare a dry-run plan

- **GIVEN** a fixture ID, baseline mode, output directory, prompt metadata, and local `project=/path/to/repo` mapping
- **WHEN** the run preparation command is run
- **THEN** it SHALL validate the fixture, prompt, local repository, base commit, gold proposal commit, baseline mode, and output path
- **AND** it SHALL emit JSON that explicitly indicates no materialization or model execution occurred.

### Requirement: Task decomposition semantic rubric records

The project SHALL support validating human-authored rubric score JSON for candidate task-decomposition outputs.

#### Scenario: Validate rubric score JSON

- **GIVEN** a rubric score JSON file for a known fixture and baseline mode
- **WHEN** the rubric validation command is run
- **THEN** it SHALL validate required semantic dimensions, score ranges, reviewer metadata, notes, and emit the computed average score.

