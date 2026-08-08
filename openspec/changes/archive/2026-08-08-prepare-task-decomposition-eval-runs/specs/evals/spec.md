# evals Specification

## ADDED Requirements

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
