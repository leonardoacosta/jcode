# Config composition

## Purpose

Let users organize Jcode settings into explicit local TOML files while preserving deterministic effective values, safe preference updates and reliable reload behavior.

## ADDED Requirements

### Requirement: Explicit local configuration sources

Jcode SHALL support an optional root `include` array of relative TOML paths. It MUST resolve paths relative to the root configuration directory and load only the listed regular files. It MUST reject absolute paths, parent traversal, expansion syntax, globs, duplicate canonical sources, self-inclusion, nested includes and canonical targets outside that directory. Missing includes SHALL leave single-file behavior supported.

#### Scenario: Split tables load independently of working directory
- **WHEN** a root includes `config/ui.toml` and `config/providers/example.toml` and Jcode runs from another working directory
- **THEN** fully qualified tables in both files contribute to the effective config without scanning other files in the folder

#### Scenario: Unsafe or invalid source selection
- **WHEN** an include is missing, unreadable, malformed, nested, duplicated, a directory, a self-reference or a path escaping containment through traversal or symlink
- **THEN** strict loading fails with the offending source identified and no partial configuration or disk mutation

### Requirement: Deterministic merge and compatibility

Jcode SHALL merge included tables recursively in listed order, then root settings, then apply typed defaults and existing environment overrides. Later scalars and arrays MUST replace earlier values, including empty arrays and arrays of tables. Table/non-table shape conflicts MUST fail with key and source diagnostics. Unknown non-manifest keys MUST remain compatible and preserved on disk.

#### Scenario: Precedence and sibling preservation
- **WHEN** two fragments and the root define the same leaf but different sibling settings
- **THEN** the root leaf wins, otherwise the last fragment wins, and untouched siblings remain effective
- **AND** an existing supported environment override wins at runtime without being saved incidentally

#### Scenario: Model catalogs replace rather than append
- **WHEN** a later source defines a provider model array, including an empty array
- **THEN** that entire array replaces the earlier array without implicit model deduplication or concatenation

#### Scenario: Incompatible document shapes
- **WHEN** one source defines a table and another defines a scalar or array at the same key
- **THEN** loading fails with that key and both source paths instead of silently coercing a value

#### Scenario: Legacy file remains usable
- **WHEN** a valid existing single-file configuration has no manifest and contains unknown future tables
- **THEN** supported effective values retain existing semantics and unknown data is not rejected or deleted

### Requirement: Sparse non-destructive preference persistence

Jcode preference updates MUST write changed settings as explicit root overrides without rewriting fragments or flattening the effective config. Saves MUST preserve unrelated keys, comments, manifest, permissions and unknown data, omit incidental environment overrides and avoid serializing unchanged defaults. The root write MUST use atomic replacement. Detected source conflicts and invalid composition MUST prevent writes.

#### Scenario: A preference originally belongs to a fragment
- **WHEN** a normal preference setter changes a setting loaded from a fragment
- **THEN** only the requested root override is persisted, every fragment remains byte-identical, and the next strict load returns the new file-configured value

#### Scenario: No change and new-file saves
- **WHEN** an unchanged config is saved or a new config differs from defaults at only one leaf
- **THEN** the unchanged save performs no write and the new file persists only the changed leaf

#### Scenario: Unrepresentable removal of inherited value
- **WHEN** removing an optional root value would expose an inherited value and no neutral TOML value represents the requested state
- **THEN** the save fails without mutation and identifies the key and inherited source for manual correction

#### Scenario: Interrupted or conflicting save
- **WHEN** source content changes after the save baseline is read, composition is invalid, or temporary writing or replacement fails
- **THEN** the operation reports the conflict or error, does not replace the original with partial content and does not rewrite fragments

### Requirement: Source-aware reload and trust freshness

Jcode SHALL detect edits, deletion, replacement and restoration of every selected source and changes to manifest membership. Main reload SHALL retain its existing throttle. Auth trust checks MUST observe changed source policy on their next check. Strict load errors MUST identify source paths and available key/line information without echoing values. Non-strict failures MUST retain the existing logged whole-config defaults fallback, never a partially merged config.

#### Scenario: Same-length edit to a fragment
- **WHEN** a selected fragment changes without changing file length while the root is untouched
- **THEN** the next eligible main config check reloads effective settings and notifies existing dependent caches

#### Scenario: Trust is revoked in a fragment
- **WHEN** an auth fragment removes a previously allowed external source
- **THEN** the next trust check observes that removal rather than returning the root-metadata memoized permission

#### Scenario: Broken source and recovery
- **WHEN** a selected file is deleted or becomes invalid and is then repaired
- **THEN** strict loading reports the source failure, non-strict loading logs and uses whole-config defaults, and a later eligible check recovers the complete repaired configuration
- **AND** diagnostics never include configuration values or raw secret-bearing lines

### Requirement: Truthful fragment edit feedback

Config-edit feedback SHALL recognize active fragments and validate the whole composition. It MUST classify effective changes using existing live/restart semantics and distinguish edits masked by higher precedence. Unlisted files MUST NOT be reported as active config.

#### Scenario: Root override masks an edit
- **WHEN** a tool edits an active fragment setting that a root override masks
- **THEN** feedback reports no effective change rather than claiming the edited value is live

#### Scenario: Active and inactive edits
- **WHEN** a tool edits an unmasked active fragment or an unlisted TOML file
- **THEN** the active edit gets effective-change feedback and invalidates config caches, while the unlisted file receives no active-config claim

### Requirement: Documented opt-in adoption

Jcode SHALL document a nonsecret split layout, precedence, array replacement, root override persistence, failure behavior and rollback to an equivalent single file. Installation and normal loading MUST NOT migrate existing user files automatically.

#### Scenario: Equivalent documented layouts
- **WHEN** the documented single-file example and its split equivalent are loaded through the config API in isolated homes
- **THEN** their typed effective settings match and no live user configuration is changed
