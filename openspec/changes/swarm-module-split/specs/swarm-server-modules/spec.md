## ADDED Requirements

### Requirement: Behavior-preserving swarm module split
The server swarm coordinator SHALL be organized as a `server/swarm/` module directory with concern submodules, and SHALL preserve every existing item path, signature, and visibility reachable from outside the former `swarm.rs`, so no caller outside `server/swarm/` changes.

#### Scenario: Callers compile unchanged
- **WHEN** the split is applied and `cargo build -p jcode-app-core` runs
- **THEN** it succeeds with no edits to files outside `crates/jcode-app-core/src/server/swarm/` other than removing `swarm.rs`

#### Scenario: Accidental API widening is rejected
- **WHEN** a moved item is made more visible than before
- **THEN** review fails the change and the item is restored to its original reach

### Requirement: Existing swarm tests pass unchanged
All existing swarm tests SHALL be kept with unchanged bodies and SHALL pass after the split.

#### Scenario: Test suite passes
- **WHEN** `cargo test -p jcode-app-core swarm` runs after the split
- **THEN** the same 31 swarm unit tests plus `swarm_cascade_tests` run and pass

#### Scenario: Test logic modified
- **WHEN** the diff changes a test assertion or body beyond `use` paths
- **THEN** the change is rejected as not behavior-preserving

### Requirement: Cohesive file size
No resulting swarm production submodule SHALL exceed 700 lines.

#### Scenario: Split sizes
- **WHEN** `wc -l crates/jcode-app-core/src/server/swarm/*.rs` runs excluding tests
- **THEN** each file is at most 700 lines

#### Scenario: Oversized module
- **WHEN** a submodule exceeds 700 lines
- **THEN** it is split further along an existing function seam before merge
