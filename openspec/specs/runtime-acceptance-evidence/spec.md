# runtime-acceptance-evidence Specification

## Purpose
Bind verification claims to the actual source, client and serving binary, distinguishing authoritative matches from mismatches and unavailable evidence.

## Requirements

### Requirement: Build cleanliness is bound to immutable compiler inputs
An authoritative clean build assertion SHALL require an enforced immutable source snapshot used by the compiler. Caller environment flags, mutable pre-build observations and compilation alone SHALL NOT establish that assertion.

#### Scenario: Source changes during compilation
- **WHEN** the original checkout changes while an immutable build runs
- **THEN** the compiler SHALL read only the captured source bytes and attempts to mutate those captured bytes SHALL fail

#### Scenario: Unsupported immutable execution
- **WHEN** immutable source enforcement is unavailable or the source inputs cannot be captured safely
- **THEN** the build SHALL refuse the verified path or report cleanliness as unknown without silently falling back to a verified mutable build

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
