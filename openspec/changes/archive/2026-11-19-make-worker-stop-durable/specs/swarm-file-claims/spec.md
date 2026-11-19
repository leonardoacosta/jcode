# Swarm file claims delta

## MODIFIED Requirements

### Requirement: Claim release
The swarm server SHALL release a worker generation's claims only after that generation's turns and tracked owned mutations have settled. Successful work completion, confirmed stop or reconciled orphan recovery SHALL trigger release after that condition holds. A stop request, membership removal, cancellation signal, async caller cancellation or orphan label alone SHALL NOT release claims or allow automatic ownership transfer. An older generation's cleanup SHALL NOT release a newer generation's claims.

#### Scenario: Stop releases
- **WHEN** worker A's generation has durably stopped and all its tracked mutation work has settled
- **THEN** none of that generation's claims SHALL remain and worker B can claim those paths

#### Scenario: Orphan releases
- **WHEN** worker A is detected as orphaned after reload and its predecessor execution and mutations are proved settled
- **THEN** its claims SHALL be released

#### Scenario: In-flight mutation outlives cancellation
- **WHEN** stop cancels an async caller while its owned filesystem mutation is still pending
- **THEN** its claims SHALL remain protected until the mutation settles and no terminal stop success SHALL be reported early

#### Scenario: Membership removed but execution unresolved
- **WHEN** a worker loses membership while owned mutation work remains live or uncertain
- **THEN** its claims SHALL NOT become free solely because it is no longer a registered worker

#### Scenario: Stale cleanup after explicit resume
- **WHEN** an old generation's cleanup runs after a newer generation has acquired a claim
- **THEN** that cleanup SHALL NOT remove the newer generation's ownership

## ADDED Requirements

### Requirement: Revoked native writes cannot bypass claim checks
Native mutation admission SHALL check the execution generation's revocation independently of swarm worker membership or conflict mode. Existing `off`, `warn` and `block` conflict behavior SHALL remain unchanged for non-revoked workers. A retained claim SHALL NOT be advertised as universal fencing for arbitrary shell or external tools.

#### Scenario: Removed worker issues another native edit
- **WHEN** a revoked worker attempts edit, write or either patch format after worker unregistration
- **THEN** the tool SHALL refuse the mutation before file bytes change in every claim mode

#### Scenario: Existing warning and block behavior
- **WHEN** an admitted non-revoked worker conflicts with another admitted worker's path claim
- **THEN** existing warning or block behavior SHALL remain configured as before, without granting a revoked generation any bypass
