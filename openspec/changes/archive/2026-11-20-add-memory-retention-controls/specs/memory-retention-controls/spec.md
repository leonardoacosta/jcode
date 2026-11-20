## ADDED Requirements

### Requirement: Memory purge is explicit

`jcode memory purge` SHALL report matching entries without deleting unless `--apply` is given, and SHALL only delete entries matching the given filters. Automatic retention SHALL remain disabled unless the user explicitly enables a bounded policy, and SHALL remove only inactive/superseded entries matching that policy. Recall and deletion SHALL share stored memory identity and safe persistence so a stale save cannot resurrect a purged entry. Secret-pattern rejection SHALL NOT be presented as proof that all confidential content is detected.

#### Scenario: Dry run
- **WHEN** the user runs `jcode memory purge --inactive`
- **THEN** matching entries are listed and none are deleted

#### Scenario: Apply with filter
- **WHEN** the user runs `jcode memory purge --inactive --older-than 30 --apply`
- **THEN** only inactive entries older than 30 days are deleted and active entries remain

#### Scenario: Retention disabled by default
- **WHEN** configuration has no explicitly enabled retention policy
- **THEN** age or supersession alone triggers no automatic deletion

#### Scenario: Concurrent recall and purge
- **WHEN** a scoped memory is purged while recall or another save holds an older snapshot
- **THEN** no later save resurrects the deleted identity and the next recall excludes it

### Requirement: Secrets are not stored in memory

Memory storage SHALL reject the entire request when content matches a known secret pattern, SHALL persist nothing from that request, and SHALL report a safe reason without echoing sensitive content.

#### Scenario: Secret in memory content
- **WHEN** a remember request contains an API-key-shaped token
- **THEN** no content from the request SHALL be stored and the tool result SHALL explain rejection without repeating the token

#### Scenario: Ordinary content
- **WHEN** a remember request contains no secret pattern
- **THEN** content is stored unchanged

#### Scenario: Sanitized resubmission
- **WHEN** the caller resubmits deliberately sanitized content after a secret rejection
- **THEN** only the new request SHALL be evaluated and no content from the rejected request SHALL have been persisted
