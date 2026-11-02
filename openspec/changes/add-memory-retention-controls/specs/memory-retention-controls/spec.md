## ADDED Requirements

### Requirement: Memory purge is explicit

`jcode memory purge` SHALL report matching entries without deleting unless `--apply` is given, and SHALL only delete entries matching the given filters.

#### Scenario: Dry run
- **WHEN** the user runs `jcode memory purge --inactive`
- **THEN** matching entries are listed and none are deleted

#### Scenario: Apply with filter
- **WHEN** the user runs `jcode memory purge --inactive --older-than 30 --apply`
- **THEN** only inactive entries older than 30 days are deleted and active entries remain

### Requirement: Secrets are not stored in memory

Memory storage SHALL redact content matching known secret patterns before persisting and SHALL report the redaction.

#### Scenario: Secret in memory content
- **WHEN** a remember request contains an API-key-shaped token
- **THEN** the stored content has the token redacted and the tool result states that redaction occurred

#### Scenario: Ordinary content
- **WHEN** a remember request contains no secret pattern
- **THEN** content is stored unchanged
