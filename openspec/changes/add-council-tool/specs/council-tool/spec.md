## ADDED Requirements

### Requirement: Council lifecycle actions
The system SHALL provide a native `council` tool with actions `open`, `round`, `say`, `log`, `close`, and `run`, implemented over the existing swarm spawn/message/await/stop runtime.

#### Scenario: One-shot run
- **WHEN** `run` is called with a topic, 3 seats, and default rounds
- **THEN** seats give opening statements, one round runs, a verdict is returned, and all seats are stopped

#### Scenario: Invalid seat count
- **WHEN** `open` is called with 1 seat or 5 seats
- **THEN** the call is rejected before any seat is spawned

### Requirement: Bounded rounds and calls
The council SHALL run opening statements plus a fixed number of rounds (default 1, max 3), each with a per-round timeout, and SHALL NOT exceed `seats * (rounds + 1) + 1` model calls unless a lower configured cap applies.

#### Scenario: Call cap respected
- **WHEN** a 3-seat, 1-round council completes
- **THEN** at most 7 model calls were made

#### Scenario: Seat times out
- **WHEN** one seat does not answer before the round timeout
- **THEN** the round completes with the other replies and the failed seat is named in the result

### Requirement: Early stop on agreement
The council SHALL skip remaining rounds when every live seat's latest reply carries the same vote; a missing vote SHALL block early stop.

#### Scenario: Unanimous after openings
- **WHEN** all seats vote the same option in opening statements with rounds=2
- **THEN** no further rounds run and the verdict is produced

### Requirement: Verdict with tally and dissent
A separate verdict call SHALL return `PASS` or `PARTIAL`, a vote tally, and named dissent; unparseable verdict output SHALL be reported as `PARTIAL`.

#### Scenario: Split vote
- **WHEN** two seats vote A and one votes B
- **THEN** the result is `PARTIAL` with tally A=2, B=1 and the B seat listed as dissent

### Requirement: Read-only seats
Seats SHALL be spawned with a Rust-enforced tool allowlist that excludes file writes, edits, patches, shell, and git mutations.

#### Scenario: Seat attempts edit
- **WHEN** a seat calls `edit`
- **THEN** the executor rejects the call and no file changes

#### Scenario: Workspace unchanged
- **WHEN** a council run completes
- **THEN** `git status` output is identical to before the run

### Requirement: No orphan seats and persisted transcript
All seats SHALL be stopped on close, abort, error, or coordinator crash, and the transcript SHALL be persisted after each round.

#### Scenario: Abort mid-round
- **WHEN** the coordinator aborts during a round
- **THEN** every seat is stopped and the partial transcript is returned and saved

#### Scenario: Coordinator crash
- **WHEN** the server reloads with an open council whose owner is gone
- **THEN** its seats are stopped during swarm recovery
