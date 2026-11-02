## ADDED Requirements

### Requirement: Typed review verdict

`jcode review --json` SHALL print exactly one verdict object bound to the reviewed revision and checks, and SHALL NOT derive the verdict from free-text matching.

#### Scenario: Passing review
- **WHEN** the reviewer returns a valid verdict with `verdict = pass`
- **THEN** the output includes `revision` and `checks` and the process exits 0

#### Scenario: Findings present
- **WHEN** the reviewer returns `verdict = fail` with findings
- **THEN** each finding has severity and path and the process exits 1

### Requirement: Non-pass outcomes are explicit

Reviews that cannot complete SHALL report `inconclusive`, `cancelled` or `budget_exhausted`, never `pass`.

#### Scenario: Malformed reviewer output
- **WHEN** the reviewer output is not valid verdict JSON
- **THEN** the verdict is `inconclusive` with the parse error and the process exits 2

#### Scenario: Cancelled review
- **WHEN** the review is interrupted
- **THEN** the verdict is `cancelled` and the process exits non-zero
