# Native Initiative Command Center Specification

## Purpose

Define the documented native Jcode authority, vocabulary, and projection boundary for durable initiatives and the Command Center surfaces.

## ADDED Requirements

### Requirement: One canonical vocabulary is documented

Jcode documentation SHALL state that `initiative` is the canonical user-facing term, that the `Goal*` Rust types are the internal representation of the same entity, and SHALL NOT introduce a third term for it.

#### Scenario: A contributor reads the initiative guide

- **WHEN** a contributor opens the native initiative guide
- **THEN** it SHALL state that `initiative` and `Goal` denote one entity
- **AND** it SHALL name `initiative` as canonical in user-facing text and `Goal` as canonical in Rust type names
- **AND** it SHALL list `Goal`, `GoalStatus`, `GoalMilestone`, `GoalStep`, and `GoalUpdate` as the internal types.

#### Scenario: Documentation asserts a durability property

- **WHEN** documentation states any durability, ordering, or uniqueness guarantee about initiatives
- **THEN** the statement SHALL cite a source file and line that supports it
- **AND** it SHALL NOT assert a guarantee the cited code does not provide.

### Requirement: Persistence authority is documented at its true scope

Documentation SHALL identify app-core goal persistence as authoritative for initiative identity, status, milestones, steps, and updates, and SHALL separately record the non-durable properties.

#### Scenario: Revision semantics are documented

- **WHEN** documentation describes initiative revisions
- **THEN** it SHALL state that a revision is derived from `updated_at` in milliseconds and is not a persisted field
- **AND** it SHALL state that two saves within the same millisecond produce the same revision.

#### Scenario: Idempotency scope is documented

- **WHEN** documentation describes command idempotency
- **THEN** it SHALL state that the idempotency record is held in process memory and is lost when the daemon restarts
- **AND** it SHALL state that a command retried after a restart re-applies
- **AND** it SHALL reference the durable-store work tracked outside this change.

#### Scenario: Checkpoints are documented

- **WHEN** documentation mentions checkpoints
- **THEN** it SHALL state that a checkpoint summary is appended to the initiative's updates list
- **AND** it SHALL NOT describe a checkpoint as an independently addressable entity.

#### Scenario: The derived memory store is documented

- **WHEN** documentation states that persistence has a single authority
- **THEN** it SHALL also record that a derived memory projection is written on each update
- **AND** it SHALL mark that projection as non-authoritative.

### Requirement: The Command Center web UI holds no local persistence

The Command Center web application SHALL NOT use browser storage APIs, and this SHALL be enforced by a repository check rather than asserted in prose alone.

#### Scenario: The invariant is checked

- **WHEN** the no-frontend-persistence check runs against `apps/command-center/src`
- **THEN** it SHALL report no matches for `localStorage`, `sessionStorage`, or `indexedDB`
- **AND** it SHALL exit with status `0`.

#### Scenario: A storage call is introduced

- **WHEN** any file under `apps/command-center/src` adds a call to a browser storage API
- **THEN** the check SHALL exit non-zero
- **AND** it SHALL name the offending file and line.

#### Scenario: A browser submits a mutation

- **WHEN** an authenticated browser submits an initiative mutation
- **THEN** the daemon SHALL apply the existing revision, idempotency, authorization, and CSRF/origin checks
- **AND** the browser SHALL install the snapshot returned by the daemon rather than deriving one locally.

### Requirement: Known concurrency and failure modes are recorded

Documentation SHALL record the observable failure modes of the current implementation rather than omitting them.

#### Scenario: The failure-mode list is present

- **WHEN** a contributor reads the native initiative guide
- **THEN** it SHALL list revision collision within one millisecond, idempotency loss across daemon restart, last-write-wins between concurrent TUI and browser writes, and a memory-sync failure that returns an error after a successful durable write
- **AND** each entry SHALL cite its source location.

#### Scenario: The TUI write path is described

- **WHEN** documentation describes initiative mutation paths
- **THEN** it SHALL state that the TUI writes through the goal module directly rather than the daemon repository
- **AND** it SHALL state that the daemon revision check does not apply on that path.

### Requirement: Extensions preserve authority boundaries

New initiative views and commands SHALL extend existing native DTOs, commands, projections, and security boundaries before introducing new storage or parallel lifecycle models.

#### Scenario: A new browser view is proposed

- **WHEN** a contributor adds a new initiative view
- **THEN** the implementation SHALL identify the existing native read and mutation seams it uses
- **AND** it SHALL document why an existing seam cannot satisfy the requirement before adding a new one.
