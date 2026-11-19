## ADDED Requirements

### Requirement: Workspace detection from origin
The system SHALL resolve the active workspace by matching the working directory's git `origin` URL, with credentials stripped, against ordered user-configured rules; the first match wins.

#### Scenario: Origin matches a rule
- **WHEN** origin is `https://dev.azure.com/brown/proj/_git/repo` and a rule maps `dev.azure.com/brown/**` to `brown`
- **THEN** the active workspace is `brown`

#### Scenario: No origin or no match
- **WHEN** the directory has no repo, no origin, or no matching rule
- **THEN** the workspace is `unknown` and no hints are injected

#### Scenario: Credentials in origin
- **WHEN** origin contains `user:token@`
- **THEN** the token is never matched, logged, displayed, or injected

### Requirement: Workspace status badge
The TUI header SHALL show the workspace as a single-row badge that never wraps, truncating with an ellipsis by display-cell width.

#### Scenario: Narrow terminal
- **WHEN** available width is smaller than the badge
- **THEN** the badge truncates with `…` and occupies one row

#### Scenario: Unknown workspace
- **WHEN** the workspace is `unknown`
- **THEN** no badge is shown

### Requirement: Per-workspace hints
The system SHALL inject the matched workspace's tracker, branch template, and skill hints into the system prompt, bounded in size.

#### Scenario: Hints present
- **WHEN** workspace `brown` has `tracker = "ado"`
- **THEN** the prompt states ADO is the configured tracker

#### Scenario: Invalid hint config
- **WHEN** a rule has an unknown tracker value
- **THEN** config load warns naming the rule and that rule's hints are skipped

### Requirement: Cached Git identity check
The system SHALL compare effective `user.email` with the workspace `expected_email`, cache the result per repo root and config mtimes, and never modify Git config.

#### Scenario: Mismatch
- **WHEN** effective email differs from `expected_email`
- **THEN** the badge shows a mismatch marker and the prompt notes it

#### Scenario: Cache reuse
- **WHEN** neither git config file changed since the last check
- **THEN** no git process is spawned

#### Scenario: Identity unset
- **WHEN** `user.email` is unset
- **THEN** the system reports missing identity and does not invent one
