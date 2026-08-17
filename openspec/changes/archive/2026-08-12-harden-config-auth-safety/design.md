# Design

## Context

The config subsystem has two paths that turn one syntax error into an auth outage:

1. `Config::load()` logs a strict-load error and returns `Config::default()`.
2. The reloadable global cache installs that default and notifies config-dependent consumers as if the reload succeeded.

Every config mutator also follows `Config::load() -> mutate -> Config::save()`. If the source file is malformed, an unrelated writer serializes defaults over it. During the `auto_client_reload` incident, seven reconnecting clients ran the launch-hotkey bake against the malformed file and replaced 6,983 bytes of user configuration with 5,129 bytes of defaults. External-auth approvals and provider choices disappeared, so OpenAI and Copilot changed from available to not configured.

The worktree already contains the uncommitted `display.auto_client_reload` field and deploy-hook work. This change owns the safety work needed before that update can be completed. It must preserve those edits and must not adopt unrelated dirty files.

Repository stamp at authoring: `HEAD 79e846f967b0a9a08d4190f4ff2b6d7016328c2f`; `origin/master 5ae2385748c578ed0096f0f1837a83baca566ebb`.

## Goals / Non-Goals

**Goals:**

- Invalid config changes never replace the last valid live config or emit a successful config-reload notification.
- Read-modify-write helpers never overwrite malformed source bytes.
- Concurrent processes cannot lose each other's valid config changes.
- Successful config writes are durable, atomic, and keep the previous valid file as `config.bak`.
- The deploy-hook installer enables `display.auto_client_reload` idempotently and refuses malformed input.
- Auth approvals and provider readiness are restored from verified evidence or through provider login, then validated.

**Non-Goals:**

- A general configuration service or a new public configuration CLI.
- Automatic repair of arbitrary invalid TOML.
- Automatic trust of external credential files that the user did not previously approve.
- Changes to OAuth URLs, PKCE, callback parsing, token exchange, or refresh protocols.
- Recovery of config values for which no backup or verified evidence exists.

## Decisions

### 1. Hot reload keeps the last known-good value

When the fingerprint changes, the cache calls `Config::load_strict()`. On success it installs the new value, updates the fingerprint, records a successful reload, and notifies consumers. On failure it logs one actionable rejection event, keeps the prior `cache.config`, records the rejected fingerprint to avoid retry/log storms for unchanged invalid bytes, and does not notify auth, model, prompt, or MCP consumers.

**Rejected:** Install defaults after an error. This caused the outage. **Rejected:** Retry parsing on every read. The incident produced 27,953 duplicate-key reports and did not improve recovery.

### 2. All mutators use one strict transaction

Add an internal `Config::update` transaction that:

1. acquires the cross-process config write lock;
2. loads the current file with `load_strict()` while holding the lock;
3. applies one mutation closure;
4. serializes and parses the candidate again;
5. atomically writes the candidate and preserves the prior valid file as `config.bak`;
6. invalidates the cache only after the durable replace succeeds.

All existing `Config::set_*`, allow/revoke external-auth, and launch-hotkey writers delegate to this transaction. Direct `Config::save()` remains available for callers that own a complete new config, but it uses the same locked atomic persistence primitive and validates serialized bytes before replacement.

**Rejected:** Change only the incident writer. Any current setter can trigger the same clobber. **Rejected:** Keep permissive `load()` in mutations with a pre-save validity check. Once defaults replace the source value in memory, intent and source fields are already lost.

### 3. Cross-process locking uses platform file handles

The transaction holds `config.lock` beside `config.toml`. Unix uses a blocking exclusive `flock` on an open file handle. Windows opens the lock file with no sharing and retries boundedly while another process owns the handle. The handle lifetime is the transaction lifetime, so process exit releases ownership and a stale empty lock file is harmless. Tests use the existing test-environment lock plus real concurrent writer threads/processes where supported.

**Rejected:** Lock-directory creation. A crash leaves a stale directory that can block all future writes. **Rejected:** An in-process mutex only. The incident involved several client processes.

### 4. Reuse the existing durable atomic writer

`jcode-storage::write_bytes` already writes a unique temporary file, fsyncs it, keeps the previous inode at `.bak`, atomically renames the new file, and syncs the parent directory. Config persistence delegates to this helper. Before the write, the current source must have parsed successfully, so `config.bak` is always a last known-good file rather than a copy of malformed bytes.

The backup path is `config.bak`, matching the storage helper's extension convention.

### 5. Managed auto-reload setup is parse-aware and idempotent

`scripts/install_deploy_hook.sh` enables the setting through a small checked helper that:

- parses the current TOML before editing;
- changes the existing key when present, inserts it once in `[display]` when absent, and rejects duplicates;
- parses the candidate and asserts the requested boolean;
- writes through a temporary file and atomic replace while preserving a backup;
- produces byte-identical output on a second run.

This script path is operational setup, not a new end-user config API. Runtime code also gains a strict `Config::set_auto_client_reload` setter for typed callers and unit coverage.

### 6. Auth recovery is evidence-bound

The implementation does not infer lost trust approvals. It first checks `config.bak` and verified incident evidence. A path-bound approval is restored only if the approved source and canonical path are present in a valid backup or another trusted local record. Otherwise, the provider is reconnected with its normal login flow. `jcode auth status --json` and focused `jcode auth doctor <provider> --validate --json` provide terminal evidence.

OpenAI and Claude already have direct Jcode credential files after the incident. Copilot remains the known recovery target. Any provider login that needs browser/device confirmation is a terminal human-action gate; it does not weaken the code verification gate.

## Data Flow

### Hot reload

`fingerprint changed -> strict parse -> success: install + notify | failure: retain old config + log rejection`

### Mutation

`acquire config.lock -> strict re-read -> mutate -> serialize -> candidate parse -> atomic write + config.bak -> invalidate cache -> release lock`

### Deploy-hook setup

`strict parse -> locate one [display] key -> update/insert -> strict candidate parse -> atomic write -> rerun is no-op`

## Error Handling

- Parse/read failure: return the full source error; do not write, invalidate, or notify.
- Lock acquisition failure or timeout: return a diagnostic naming `config.lock`; do not write.
- Candidate serialization/parse failure: return the error before touching the primary file.
- Atomic write failure: keep the primary and backup unchanged when the storage helper can do so; do not invalidate the cache.
- Backup recovery: never auto-promote a backup during normal load. Recovery is an explicit operational action so an intentional current edit is not silently reversed.
- Auth recovery failure: keep the provider marked not configured and report the exact login or validation action.

## Testing Strategy

- RED/GREEN: a malformed duplicate-key file plus `set_launch_hotkeys` must stay byte-identical and return an error.
- RED/GREEN: a malformed hot reload must preserve the previous config and auth readiness, emit no successful reload notification, and log once per rejected fingerprint.
- RED/GREEN: two concurrent transactions changing distinct fields must preserve both changes.
- Persistence: successful save creates a valid `config.bak`; primary and backup remain parseable; simulated failure does not publish partial TOML.
- Managed update: key absent, false, true, duplicate, malformed, and second-run byte identity.
- Regression: full config tests, 77 OAuth unit tests, auth login integration, auth-status tests, and the auto-client-reload gate test.
- Runtime: current `config.toml` parses; OpenAI and Claude validate; Copilot validates after normal login or remains at a recorded terminal human-action gate.

## Risks / Trade-offs

- **[Blocking lock delays a config command]** -> Config writes are small; log lock waits and bound Windows retries. Unix lock release is tied to the file descriptor.
- **[A caller bypasses the transaction]** -> Inventory every `Config::load() -> save()` call in `config_file.rs`; add a source guard test that rejects permissive mutation patterns.
- **[Backup exposes sensitive config values]** -> Preserve current file permissions and apply owner-only permissions to primary, backup, and temporary files where the platform supports them.
- **[Concurrent manual editor writes ignore the lock]** -> Atomic Jcode writes prevent torn output; strict re-read prevents Jcode from overwriting an invalid manual edit. External editors cannot be forced to honor the lock.
- **[Operational auth needs user interaction]** -> Run it only after code gates pass and report one focused device/browser action if required.

## Migration Plan

1. Land config safety and regression tests while preserving the uncommitted auto-reload work.
2. Run the managed installer update; it changes the existing setting to `true` without duplication.
3. Inspect valid `config.bak` and other verified local records for prior external-auth approvals.
4. Restore evidence-backed approvals, then run normal login for remaining affected providers.
5. Validate configured providers and the commit-deploy reload path.
6. Archive the OpenSpec change after all non-human gates and required terminal auth gates pass.

Rollback: revert the feature commit and restore `config.toml` from the valid `config.bak` produced before the first new write. Do not restore the malformed incident file.

## Open Questions

None. The approved default is the layered repair above; provider trust is restored only from verified evidence.
