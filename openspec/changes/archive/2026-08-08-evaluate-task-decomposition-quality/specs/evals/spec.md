# evals Specification

## ADDED Requirements

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
