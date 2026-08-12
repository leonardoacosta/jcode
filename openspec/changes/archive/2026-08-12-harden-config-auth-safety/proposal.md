# Harden Config and Auth Safety

## Why

A malformed `config.toml` currently replaces the live configuration with defaults, invalidates authentication state, and can then be overwritten by unrelated config writers. The `auto_client_reload` rollout exposed this shared failure mode: one duplicate key removed provider and external-auth settings from active clients and allowed concurrent startup writers to persist the default configuration over the user's file.

## What Changes

- Keep the last known-good live configuration when a changed config file cannot be parsed; report the error without notifying config-dependent consumers of a successful reload.
- Make every read-modify-write config mutation load the source file strictly and fail without changing disk when it is malformed.
- Serialize config mutations, re-read while holding the write lock, preserve a recoverable backup of the last valid file, and atomically replace the destination.
- Provide an idempotent, TOML-aware update path for `display.auto_client_reload` so setup changes an existing key instead of inserting a duplicate.
- Restore affected external-auth approvals only from verified local evidence and require provider re-login where prior approvals cannot be recovered.
- Add incident regression coverage for malformed reloads, safe mutations, concurrent writers, atomic backup behavior, auth-state preservation, and idempotent auto-reload updates.

## Preconditions

- Preserve the current uncommitted auto-client-reload and deploy-hook edits. Do not revert or replace unrelated worktree changes.
- Use the current repository revision recorded in Impact as the implementation base.
- Treat the active config and provider credential files as operational state. Change them only after the code and test gates pass.
- Restore an external-auth approval only when valid local evidence identifies both its source and canonical path.

## Decisions

- A failed hot reload keeps the last known-good config and records the rejected file fingerprint.
- All config setters use one strict, locked read-modify-write transaction.
- The transaction uses a platform file-handle lock and the existing durable atomic storage writer.
- The managed installer parses TOML, changes one key, and is byte-idempotent.
- Provider recovery uses verified backup evidence or the normal provider login flow.

## Done Means

- Invalid config bytes cannot replace the live config or be overwritten by a setter.
- Concurrent setters preserve distinct changes, and each successful update keeps a valid previous-file backup.
- Managed setup enables exactly one `display.auto_client_reload = true` key and makes no change on a second run.
- Config, OAuth, auth-login, TUI gate, and installer regressions pass.
- OpenAI, Claude, and Copilot pass live status and provider validation after evidence-bound recovery or normal login.

## Testing

- Use RED/GREEN tests for malformed mutation, last-known-good reload, concurrent writers, and backup behavior.
- Run focused config tests, OAuth unit tests, auth-login integration tests, the TUI opt-in gate, and installer cases.
- Run formatting and strict OpenSpec validation before completion.
- Validate each recovered provider with the installed Jcode binary after all code gates pass.

## Capabilities

### New Capabilities

- `config-auth-safety`: Defines fail-closed configuration reload and mutation behavior, recoverable persistence, idempotent managed updates, and preservation of authentication state during invalid config changes.

### Modified Capabilities

None.

## Impact

- Base-commit: jcode@79e846f967b0a9a08d4190f4ff2b6d7016328c2f
- touches: `crates/jcode-base/src/config.rs`, `crates/jcode-base/src/config/config_file.rs`, `crates/jcode-base/src/config/default_file.rs`, `crates/jcode-base/src/config/display_summary.rs`, `crates/jcode-base/src/config/env_overrides.rs`, `crates/jcode-base/src/config_tests.rs`, `crates/jcode-config-types/src/display.rs`, `crates/jcode-tui/src/tui/app/remote/server_events.rs`, `crates/jcode-tui/src/tui/app/remote_tests.rs`, `crates/jcode-tui/src/tui/app/tui_lifecycle_runtime.rs`, `scripts/install_deploy_hook.sh`, `scripts/post_commit_deploy.sh`, `scripts/test_install_deploy_hook.py (new)`
- Affected code: `crates/jcode-base/src/config.rs`, `crates/jcode-base/src/config/config_file.rs`, config tests, auth-status integration tests, and the auto-client-reload setup surface.
- Operational state: `~/.jcode/config.toml` and its recoverable backup; external-auth approvals and provider readiness are validated after repair.
- Compatibility: valid configurations keep their current behavior. Invalid edits stop applying and remain available for correction instead of becoming defaults or being overwritten.
- Dependencies: uses the existing config cache, `Config::load_strict`, platform storage helpers, and test environment lock. No new external crate is required unless the implementation proves the existing lock primitives cannot provide cross-process exclusion.
