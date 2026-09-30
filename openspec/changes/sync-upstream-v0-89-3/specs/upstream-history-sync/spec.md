## ADDED Requirements

### Requirement: Preserve local and upstream ancestry
The synchronization SHALL merge the verified upstream v0.89.3 release into existing main without rewriting local commits and SHALL retain a recovery ref at the execution baseline.

#### Scenario: Successful synchronization
- **WHEN** integration and validation succeed
- **THEN** the baseline and de65ade33d514b31a43885318179b3622f321170 are ancestors of main and local customizations remain functional

#### Scenario: Unsafe starting state
- **WHEN** unowned edits or another Git operation affect integration
- **THEN** integration waits for coordination without resetting or discarding work

### Requirement: Resolve compatibility explicitly
The synchronization SHALL review both sides of conflicts and preserve local capabilities rather than wholesale-selecting upstream or local files.

#### Scenario: Conflicting behavior cannot coexist
- **WHEN** a conflict requires removing or changing a local capability
- **THEN** the conflict and trade-off are presented for user decision before committing that resolution

### Requirement: Enable repeatable future updates
The repository SHALL have an upstream remote for the authoritative repository and SHALL keep main tracking the user's origin/main.

#### Scenario: Future release update
- **WHEN** the user follows the reported update procedure
- **THEN** it fetches upstream, verifies a selected release, preserves a recovery ref, and performs a history-preserving merge

#### Scenario: Existing upstream remote differs
- **WHEN** upstream already refers to another repository
- **THEN** execution stops before changing that remote and requests a decision

### Requirement: Validate actual merged behavior
The synchronization SHALL run relevant regression gates and build the TUI, testing the new binary in isolation before reporting success.

#### Scenario: Validation fails
- **WHEN** a required check fails
- **THEN** the failure is repaired within scope or reported as blocking, without claiming synchronization complete

#### Scenario: Source integration is complete
- **WHEN** ancestry, regressions, build, and isolated runtime checks pass
- **THEN** the owned changes are committed and evidence is reported without pushing or replacing the shared daemon
