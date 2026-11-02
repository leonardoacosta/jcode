## ADDED Requirements

### Requirement: Retention is opt-in
Session deletion SHALL occur only when `sessions.retention.enabled` is `true` AND the user passes `--apply` or sets `sessions.retention.auto_gc = true`.

#### Scenario: Apply with retention disabled
- **WHEN** the user runs `jcode sessions gc --apply` with `enabled = false`
- **THEN** no session is deleted and the command exits non-zero naming `sessions.retention.enabled`

#### Scenario: Apply with retention enabled
- **WHEN** `enabled = true` and the user runs `jcode sessions gc --apply`
- **THEN** only classified candidates are removed and the command reports count and bytes freed

### Requirement: Dry-run by default
`jcode sessions gc` without `--apply` SHALL report candidates and SHALL NOT modify any session file.

#### Scenario: Default invocation
- **WHEN** the user runs `jcode sessions gc`
- **THEN** the output lists candidate counts per reason and total bytes, and the session directory is unchanged

### Requirement: Saved sessions are never deleted
A session with `saved = true` SHALL never be a retention candidate, regardless of debug flag, size, or age.

#### Scenario: Saved debug tiny old session
- **WHEN** a session is saved, debug, has 1 message, and is 90 days old
- **THEN** it is kept with reason `saved`

#### Scenario: Session bookmarked during GC
- **WHEN** a candidate becomes saved on disk before its deletion step
- **THEN** the re-check keeps it

### Requirement: Debug and tiny classification
Unsaved, non-live sessions SHALL be candidates when debug and older than `debug_max_age_days`, or when message count <= `tiny_max_messages` and older than `tiny_max_age_days`.

#### Scenario: Old debug session
- **WHEN** an unsaved debug session is older than `debug_max_age_days`
- **THEN** it is a candidate with reason `debug_expired`

#### Scenario: Recent debug session
- **WHEN** an unsaved debug session is newer than `debug_max_age_days`
- **THEN** it is kept

#### Scenario: Unparseable or live session
- **WHEN** a session file fails to parse or is held by a running server
- **THEN** it is kept and reported, never deleted

### Requirement: Search index excludes debug sessions
The recent session search index SHALL exclude debug sessions unless the query requests `debug: true`, and SHALL drop ids of deleted sessions.

#### Scenario: Default search
- **WHEN** `session_search` runs without a `debug` filter
- **THEN** debug sessions do not appear in results

#### Scenario: Explicit debug search
- **WHEN** `session_search` runs with `debug: true`
- **THEN** debug sessions are searchable

#### Scenario: Deleted session
- **WHEN** GC deletes a session
- **THEN** subsequent searches return no hit for its id
