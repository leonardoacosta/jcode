## ADDED Requirements

### Requirement: Assignment carries scope, base and check
The system SHALL record `base_rev`, `file_scope`, `acceptance_check` and optional `turn_budget` on each assignment, defaulting `base_rev` to current HEAD.

#### Scenario: Base recorded on spawn
- **WHEN** a worker is spawned for a plan item without `base_rev`
- **THEN** the item SHALL store the current HEAD as `base_rev`

#### Scenario: Budget exceeded
- **WHEN** a worker exceeds its `turn_budget`
- **THEN** it SHALL be marked `blocked` with reason `budget` and SHALL NOT be auto-continued

### Requirement: Evidence-bearing report
The system SHALL accept structured report fields for base/head revisions, changed files and acceptance results, and SHALL warn on missing or out-of-scope evidence.

#### Scenario: Complete report
- **WHEN** a report includes revisions, in-scope files and a passing acceptance result
- **THEN** it SHALL be stored and shown without warnings

#### Scenario: Missing check or out-of-scope file
- **WHEN** a report omits the acceptance result or lists a file outside `file_scope`
- **THEN** the coordinator view SHALL show a warning naming the gap
