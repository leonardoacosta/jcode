# Design

## Context

`Config::load_from_file_strict` reads and deserializes one document. `Config::load` logs errors and falls back to defaults. `Config::save` serializes a complete typed config, losing comments and unmodeled keys. Main reload and trusted-external-auth caches key off root metadata. Existing setters deliberately omit environment overrides during read-modify-write. Keep that boundary.

## Goals / Non-Goals

Make files easy to find and compose without changing table names or existing environment semantics. Make preference saves sparse and comment-preserving. No automatic split, new config framework or multi-file write transaction.

## Decisions

### 1. Explicit manifest, one include level

Only the root document accepts top-level `include`, an array of nonempty strings. Missing or empty means single-file behavior. Paths are resolved relative to the root's parent, never process cwd. Require `.toml` regular files within that canonical directory. Reject absolute paths, `..` components, expansion syntax, globs, duplicate canonical files, the root itself, nested `include` keys and symlinks escaping the root directory. Allow contained symlinks for reading. Treat paths as local data, never executable instructions. Do not scan `config/` automatically.

An explicit manifest beats automatic `config.d` discovery: order is visible, backups cannot activate accidentally, and only files the user selected load. One-level includes avoid cycles and an inheritance language.

### 2. Merge raw values, then deserialize once

Read/parse all documents before exposing a config. Remove manifest metadata before typed deserialization. Merge fragments in listed order, then root. Recursively merge tables. Later scalar or array values replace earlier values, including arrays of provider models and empty arrays. Reject table/non-table conflicts with dotted key and both source paths. Existing typed validation handles final scalar types. Apply defaults only after merge and environment overrides only after successful typed load. Preserve legacy display normalization.

Keep source paths and winning source per key available internally for diagnostics and edit notices. Unknown settings remain accepted and preserved, retaining forward compatibility and current `[tasks]`/`[worktrees]` data. Broad unknown-key validation is a separate contract change.

### 3. Root-only override persistence

This deliberately replaces the assessment's tentative owner-file writes. Editing the winning fragment would require multi-file transactions for `Config::save`, complicate overrides and modify reusable fragments unexpectedly. Instead, compare the requested typed config with a fresh file-only effective snapshot, calculate changed leaves, and patch just those leaves into the root via existing `toml_edit`. Arrays are indivisible leaves. Preserve comments, manifest, unknown keys, order where possible and untouched tables. New keys go in their existing fully qualified tables. No changes means no write.

This applies to both legacy single-file and composed saves. A new file receives only values differing from defaults. Existing explicit settings remain explicit. A root override can be manually removed to inherit again. When a removed optional key would reveal an inherited non-default value and TOML has no representable neutral value, refuse that operation without writing, naming the key and fragment the user can edit. Representable empty/default values remain explicit overrides. Do not introduce a TOML null/deletion extension.

Audit setter call sites and direct `save` callers: setters must compute their intended change against a fresh file-only baseline, not an environment-applied snapshot. Do not persist effective environment values as incidental changes. Keep the public save API compatible where possible and document its changed sparse-write behavior.

Write one root file atomically using a same-directory temporary file and replacement, retain permissions, and check that all loaded source content is unchanged immediately before replacement. Refuse detected concurrent changes rather than merge a stale snapshot. No claim of lock-free cross-process compare-and-swap: a narrow race against arbitrary external editors remains. Resolve a symlinked root to its existing contained target, preserving the symlink itself; reject unsafe target changes. On parse/read/path/conflict/write failure do not truncate the original, flatten config or leave an active partial file. Clean up only the owned temporary file.

Root-only legacy migrations must consult effective values and use this path rather than assume fragment values live in the root. They must not silently mutate fragment data.

### 4. Complete source freshness and consistent failure policy

Use a source-set/content fingerprint shared by the main reload and trusted-auth memoization. Capture root manifest and all selected files, including missing/unreadable states. Detect additions/removals through manifest changes and equal-length fragment edits. Keep the existing main reload throttle. Auth trust checks must observe changed fragment policy on their next check, without waiting for that throttle. Reuse reload listeners, generation and provider context-limit refresh.

For compatibility, strict loads return errors and non-strict loads retain the existing logged whole-config defaults fallback. Never load only the valid subset of a broken composition or silently retain old auth trust. Do not introduce last-known-good behavior in this feature. Diagnostics identify source path and key/line when available, not configuration values, inline secrets or file excerpts. Repairing a missing/broken source must permit recovery on the next eligible check.

### 5. Config-edit feedback uses effective changes

Extend the existing edit watch from one file to the selected source set. A change to an active fragment invalidates caches and validates the whole composition. Compare effective before/after settings for the existing live/restart classifications. If a root override masks a fragment edit, report that it has no effective change, not that the new value is live. Unlisted files are ordinary files. If an edit introduces an invalid manifest or fragment, report the load failure and actual default fallback without echoing raw values.

### 6. Documentation, compatibility and rollback

Provide nonsecret examples with root `include`, `config/ui.toml`, `config/agents.toml`, `config/tools.toml`, `config/runtime.toml`, and `config/providers/example.toml`. No existing install is converted. Show identical effective settings for single-file and split examples, sparse root writes, array replacement and override removal. Downgrading to an old binary requires manually flattening an equivalent config first because old binaries ignore `include`; do not promise old binaries understand fragments.

## Risks / Trade-offs

- Root overrides can grow over time. This is visible and safer than implicit fragment writes. No automatic compaction or defaults deletion.
- Fallback to defaults on malformed configuration is existing behavior, not ideal error recovery. Make it explicit and test it, including auth revocation paths.
- Content fingerprints add bounded local reads. Preserve throttling and measure representative config reload work before claiming no performance regression.
- Shared config surfaces overlap other active proposals. Revalidate callers and serialize implementation edits. Do not add false semantic prerequisites.
- No network, provider credentials, paid model calls or live user config is required for acceptance. Disposable `JCODE_HOME` fixtures exercise public Rust config APIs and real edit-watch code.
