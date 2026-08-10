## ADDED Requirements

### Requirement: Invalid hot reload preserves live configuration
Jcode SHALL retain the last known-good live configuration when a changed config file cannot be read or parsed. It SHALL NOT report a successful config reload or notify config-dependent consumers for rejected bytes.

#### Scenario: Duplicate key is rejected without auth change
- **WHEN** the active config is valid and a file edit introduces a duplicate TOML key
- **THEN** Jcode SHALL keep the prior live configuration
- **AND** authentication readiness, provider routes, and external-auth approvals SHALL remain unchanged.

#### Scenario: Rejected fingerprint does not create a log storm
- **WHEN** callers read config repeatedly while the same invalid file fingerprint remains on disk
- **THEN** Jcode SHALL report the rejection once for that fingerprint
- **AND** SHALL retry only after the fingerprint changes or an explicit reload is requested.

#### Scenario: Corrected file applies normally
- **WHEN** the invalid file is replaced with valid TOML having a new fingerprint
- **THEN** Jcode SHALL install the corrected configuration
- **AND** SHALL notify config-dependent consumers exactly as for any other successful reload.

### Requirement: Config mutations fail closed
Every Jcode read-modify-write config operation SHALL strictly read the current source while holding the config write lock. If the source is malformed or unreadable, the operation SHALL return an error and preserve the source bytes exactly.

#### Scenario: Unrelated writer cannot erase malformed config
- **WHEN** `config.toml` contains a duplicate key and the launch-hotkey bake or another setter attempts to write
- **THEN** the operation SHALL fail with the parse error
- **AND** the primary file bytes, backup, live config, and auth state SHALL remain unchanged.

#### Scenario: Missing config starts from defaults
- **WHEN** no config file exists and a typed setter runs
- **THEN** the operation SHALL create a valid config using schema defaults plus the requested mutation
- **AND** SHALL NOT treat absence as a malformed-file error.

#### Scenario: Candidate must parse before publication
- **WHEN** mutation output cannot be serialized and parsed as the current Config schema
- **THEN** Jcode SHALL reject the candidate before changing the primary file or cache.

### Requirement: Config transactions preserve concurrent updates
Jcode SHALL serialize config mutations across processes, re-read the source after acquiring the lock, and apply each mutation to the latest valid configuration.

#### Scenario: Distinct concurrent changes are not lost
- **WHEN** two Jcode processes concurrently update different config fields
- **THEN** both successful mutations SHALL be present in the final valid config
- **AND** neither process SHALL publish a partial or default-only file.

#### Scenario: Process exit releases lock ownership
- **WHEN** a writer exits while owning or after opening the config lock file
- **THEN** a later writer SHALL be able to acquire the lock without manual deletion of the lock file.

### Requirement: Config persistence is atomic and recoverable
Successful config persistence SHALL atomically replace `config.toml`, preserve the previous valid primary at `config.bak`, durably flush the new bytes, and preserve owner-only permissions where supported.

#### Scenario: Successful update creates valid backup
- **WHEN** a valid existing config is successfully updated
- **THEN** `config.toml` SHALL contain the new valid configuration
- **AND** `config.bak` SHALL contain the complete previous valid configuration.

#### Scenario: Write failure does not publish partial TOML
- **WHEN** temporary-file writing, flushing, or replacement fails
- **THEN** readers SHALL observe the previous complete primary or the complete new primary, never partial bytes
- **AND** Jcode SHALL NOT invalidate the live config cache for the failed write.

### Requirement: Managed auto-client-reload setup is idempotent
The deploy-hook setup SHALL enable `display.auto_client_reload` by parsing and updating valid TOML, SHALL never create more than one key in `[display]`, and SHALL refuse malformed or duplicate-key input.

#### Scenario: Existing false value changes in place
- **WHEN** valid config contains `auto_client_reload = false`
- **THEN** setup SHALL change that single value to `true`
- **AND** SHALL preserve unrelated config values.

#### Scenario: Missing key is inserted once
- **WHEN** valid config has a `[display]` table without `auto_client_reload`
- **THEN** setup SHALL insert exactly one `auto_client_reload = true` entry in that table.

#### Scenario: Second setup run is byte-identical
- **WHEN** setup runs against a config that already has `auto_client_reload = true`
- **THEN** it SHALL succeed without changing the file bytes.

#### Scenario: Duplicate or malformed input is preserved
- **WHEN** setup receives malformed TOML or more than one `auto_client_reload` key in `[display]`
- **THEN** it SHALL fail with an actionable error
- **AND** SHALL preserve the input bytes exactly.

### Requirement: Authentication recovery uses verified evidence
Jcode operations SHALL restore external-auth approvals only from a valid backup or another verified local record that identifies both the source and canonical credential path. Providers without recoverable approval SHALL use their normal login flow before being reported ready.

#### Scenario: Verified path approval is restored
- **WHEN** a valid pre-incident backup records a path-bound external-auth approval and the canonical credential file still exists
- **THEN** recovery SHALL restore that exact approval
- **AND** provider diagnosis SHALL recognize the source without broadening trust.

#### Scenario: Missing approval requires login
- **WHEN** no valid local evidence identifies a provider's prior approval
- **THEN** recovery SHALL NOT infer or add trust
- **AND** SHALL run or request the provider's normal browser, device-code, or manual-safe login flow.

#### Scenario: Recovered configured providers are validated
- **WHEN** config recovery and any required logins finish
- **THEN** each recovered configured provider SHALL pass `jcode auth status` and its applicable live provider validation
- **AND** failures SHALL remain visible with the exact next action instead of being marked complete.
