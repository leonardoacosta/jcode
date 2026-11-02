## ADDED Requirements

### Requirement: Evidence names the serving binary
The system SHALL report source revision, client hash and serving server hash with a verdict of matched, mismatched or unknown.

#### Scenario: Matching runtime
- **WHEN** source HEAD equals client and server git hashes and the tree is clean
- **THEN** the verdict SHALL be `matched`

#### Scenario: Stale daemon
- **WHEN** the server git hash differs from source HEAD
- **THEN** the verdict SHALL be `mismatched` and SHALL name both hashes

#### Scenario: Old server without hash
- **WHEN** the connected server does not report a git hash
- **THEN** the verdict SHALL be `unknown` and SHALL NOT be `matched`
