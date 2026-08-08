## MODIFIED Requirements

### Requirement: Task decomposition fixture catalog

The project SHALL provide a task-decomposition eval fixture catalog that records historical OpenSpec proposal situations by category without embedding absolute local source paths, and each runnable fixture SHALL include an intent contract for semantic planning-quality evaluation.

#### Scenario: Validate fixture metadata offline

- **GIVEN** the checked-in fixture catalog
- **WHEN** the eval validation command is run without repository root mappings
- **THEN** it SHALL validate required fields, unique fixture IDs, commit hash shapes, expected artifact declarations, category coverage, and intent-contract structure using only repository files and Python standard library modules.

#### Scenario: Validate fixture intent contract

- **GIVEN** a fixture intended for task-decomposition evaluation
- **WHEN** the fixture catalog validation command is run
- **THEN** it SHALL require user intent, scope boundaries, expected blast-radius surfaces, non-goals, ambiguity traps, and reference notes
- **AND** those fields SHALL be usable without treating the historical OpenSpec proposal as a reproduction target.

### Requirement: Gold artifact scoring

The project SHALL support deterministic evidence extraction from the fixture's historical reference commit and candidate artifacts while treating artifact overlap as supporting evidence rather than the primary task-decomposition score.

#### Scenario: Score candidate artifacts

- **GIVEN** a fixture ID, a candidate OpenSpec change directory, and a `project=/path/to/repo` mapping for the fixture project
- **WHEN** the scoring command is run
- **THEN** it SHALL compare required artifact presence and token overlap against the historical reference change artifacts and emit a JSON support-evidence report
- **AND** it SHALL label the overlap result as non-semantic evidence that does not by itself determine planning quality.

#### Scenario: Extract blast-radius support evidence

- **GIVEN** a fixture with an intent contract and historical reference commit
- **WHEN** deterministic evidence extraction is run for a candidate plan
- **THEN** it SHALL report candidate mentions and omissions for expected blast-radius surfaces, non-goal violations, and ambiguity traps
- **AND** it SHALL report relevant changed-path surfaces from the historical reference commit for reviewer context.

### Requirement: Task decomposition semantic rubric records

The project SHALL support validating human-authored rubric score JSON for candidate task-decomposition outputs using planning-quality dimensions rather than historical artifact reproduction.

#### Scenario: Validate rubric score JSON

- **GIVEN** a rubric score JSON file for a known fixture and baseline mode
- **WHEN** the rubric validation command is run
- **THEN** it SHALL validate required semantic dimensions for fidelity, scope lock, blast-radius discovery, risk/dependency ordering, and verification executability
- **AND** it SHALL validate score ranges, reviewer metadata, notes, and emit the computed average score.
