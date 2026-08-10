# Tasks

## 1. Lock in the incident regressions

- [x] 1.1 Add RED regressions that write duplicate TOML keys and prove a representative setter returns an error without changing the primary bytes, backup, or live config, while a missing file still starts from schema defaults.
  - touches: `crates/jcode-base/src/config_tests.rs`
  - depends on: none
  - Verify with `cargo test -p jcode-base --lib config::tests::malformed_config_mutation_preserves_source -- --exact` and `cargo test -p jcode-base --lib config::tests::missing_config_setter_uses_defaults -- --exact`; the expected result before implementation is a malformed-source failure that shows the permissive setter overwrites the file, and the expected result after implementation is two passing tests with byte identity preserved for invalid input and a valid default-based file for absent input.

- [x] 1.2 Add a RED hot-reload regression that proves a duplicate-key edit keeps the last known-good config and auth approvals, sends no success notification, reports one rejection per fingerprint, and accepts a later corrected file.
  - touches: `crates/jcode-base/src/config_tests.rs`, `crates/jcode-base/src/config.rs`
  - depends on: none
  - Verify with `cargo test -p jcode-base --lib config::tests::invalid_hot_reload_keeps_last_known_good -- --exact`; the expected result before implementation is a failure because defaults replace the cache, and the expected result after implementation is one passing test with one rejection and one later successful reload.

- [x] 1.3 Add RED persistence regressions for two distinct concurrent mutations, lock release after writer process exit, a valid owner-only previous-file backup, and complete TOML after an injected write failure.
  - touches: `crates/jcode-base/src/config_tests.rs`, `crates/jcode-base/src/config/config_file.rs`
  - depends on: none
  - Verify with `cargo test -p jcode-base --lib config::tests::concurrent_config_updates_preserve_both_changes -- --exact`, `cargo test -p jcode-base --lib config::tests::config_lock_releases_after_process_exit -- --exact`, `cargo test -p jcode-base --lib config::tests::config_update_keeps_valid_backup -- --exact`, and `cargo test -p jcode-base --lib config::tests::config_write_failure_preserves_complete_primary -- --exact`; the expected result before implementation is at least one failure from a lost update, absent lock, missing backup, or non-injectable unsafe write, and the expected result after implementation is four passing tests with valid complete TOML and owner-only permissions where supported.

## 2. Make config reload and persistence fail closed

- [x] 2.1 Change the reloadable config cache to parse strictly, retain its last known-good value on error, remember the rejected fingerprint, and notify consumers only after a successful install.
  - touches: `crates/jcode-base/src/config.rs`, `crates/jcode-base/src/config_tests.rs`
  - depends on: 1.2
  - Verify with `cargo test -p jcode-base --lib config::tests::invalid_hot_reload_keeps_last_known_good -- --exact`; the expected result is one passing test, no successful reload notification for invalid bytes, and normal application after the fingerprint changes to valid bytes.

- [x] 2.2 Add a cross-process config lock and one strict `Config::update` transaction that re-reads under the lock, validates the candidate, writes atomically through `jcode_storage::write_bytes`, preserves owner-only permissions, and invalidates the cache only after success.
  - touches: `crates/jcode-base/src/config/config_file.rs`, `crates/jcode-base/src/config_tests.rs`
  - depends on: 1.1, 1.3
  - Verify with `cargo test -p jcode-base --lib config::tests::malformed_config_mutation_preserves_source -- --exact`, `cargo test -p jcode-base --lib config::tests::concurrent_config_updates_preserve_both_changes -- --exact`, `cargo test -p jcode-base --lib config::tests::config_lock_releases_after_process_exit -- --exact`, `cargo test -p jcode-base --lib config::tests::config_update_keeps_valid_backup -- --exact`, and `cargo test -p jcode-base --lib config::tests::config_write_failure_preserves_complete_primary -- --exact`; the expected result is five passing tests with no malformed overwrite, no lost distinct update, automatic lock release, a valid last-known-good backup, and no partial publication.

- [x] 2.3 Move every config read-modify-write helper, including model, reasoning, display, launch-hotkey, and external-auth approval writers, onto the strict transaction and add a source guard against new permissive mutation patterns.
  - touches: `crates/jcode-base/src/config/config_file.rs`, `crates/jcode-base/src/config_tests.rs`
  - depends on: 2.2
  - Verify with `cargo test -p jcode-base --lib 'config::tests'`; the expected result is that all config tests pass and the source guard finds no setter that uses permissive `Config::load()` before saving.

## 3. Finish the managed auto-client-reload rollout safely

- [x] 3.1 Complete the typed `display.auto_client_reload` schema, default, environment override, summary, and strict setter while preserving the existing uncommitted rollout edits.
  - touches: `crates/jcode-config-types/src/display.rs`, `crates/jcode-base/src/config.rs`, `crates/jcode-base/src/config/config_file.rs`, `crates/jcode-base/src/config/default_file.rs`, `crates/jcode-base/src/config/display_summary.rs`, `crates/jcode-base/src/config/env_overrides.rs`, `crates/jcode-base/src/config_tests.rs`
  - depends on: 2.3
  - Verify with `cargo test -p jcode-base --lib 'config::tests'`; the expected result is that schema/default/override/setter tests pass and a second request to enable the setting does not damage unrelated config.

- [x] 3.2 Make the deploy-hook installer update `display.auto_client_reload` only after strict TOML checks, preserve unrelated bytes and a backup, reject malformed or duplicate input, and do nothing on a second run.
  - touches: `scripts/install_deploy_hook.sh`, `scripts/test_install_deploy_hook.py (new)`
  - depends on: 2.2, 3.1
  - Verify with `python3 scripts/test_install_deploy_hook.py`; the expected result is that absent, false, true, duplicate, malformed, and second-run byte-identity cases all pass.

- [x] 3.3 Keep automatic client replacement limited to opted-in remote sessions and keep the post-commit hook limited to buildable changes.
  - touches: `crates/jcode-tui/src/tui/app/remote/server_events.rs`, `crates/jcode-tui/src/tui/app/remote_tests.rs`, `crates/jcode-tui/src/tui/app/tui_lifecycle_runtime.rs`, `scripts/post_commit_deploy.sh`
  - depends on: 3.1, 3.2
  - Verify with `cargo test -p jcode-tui --lib tui::app::remote::tests::auto_client_reload_gate_requires_remote_and_opt_in -- --exact` and `bash -n scripts/install_deploy_hook.sh scripts/post_commit_deploy.sh`; the expected result is one passing gate test and zero shell syntax errors.

## 4. Verify code and repair operational auth state

- [x] 4.1 Run the focused and full regression matrix after implementation.
  - touches: verification only; no additional tracked paths
  - depends on: 2.3, 3.3
  - Verify owned Rust paths with `rustfmt --edition 2024 --check crates/jcode-base/src/config.rs crates/jcode-base/src/config/config_file.rs crates/jcode-base/src/config/default_file.rs crates/jcode-base/src/config/display_summary.rs crates/jcode-base/src/config/env_overrides.rs crates/jcode-base/src/config_tests.rs crates/jcode-config-types/src/display.rs crates/jcode-tui/src/tui/app/remote/server_events.rs crates/jcode-tui/src/tui/app/remote_tests.rs crates/jcode-tui/src/tui/app/tui_lifecycle_runtime.rs`, then run `cargo test -p jcode-base --lib 'config::tests'`, `cargo test -p jcode-base --lib 'auth::oauth::tests::'`, `cargo test -p jcode --test auth_login_flow`, `cargo test -p jcode-tui --lib tui::app::remote::tests::auto_client_reload_gate_requires_remote_and_opt_in -- --exact`, and `python3 scripts/test_install_deploy_hook.py`; the expected result is zero owned-path format differences and all named test suites passing. Workspace-wide `cargo fmt --all -- --check` has unrelated baseline differences outside this feature's declared paths.

- [x] 4.2 Apply the managed setup to the active config, inspect only valid local backup evidence for exact external-auth approvals, and restore no approval that lacks source-and-path evidence.
  - touches: operational state `/home/nyaptor/dev/jcode/config.toml`, `/home/nyaptor/dev/jcode/config.bak`, and provider credential files only
  - depends on: 4.1
  - Verify with `python3 -c 'import pathlib,tomllib; p=pathlib.Path("/home/nyaptor/dev/jcode/config.toml"); tomllib.loads(p.read_text()); tomllib.loads(p.with_suffix(".bak").read_text())'` and `jcode auth status --json`; the expected result is one valid `auto_client_reload = true` key, a parseable primary and backup, OpenAI and Claude reported available, and no inferred broad external-auth trust.

- [ ] 4.3 Validate the recovered noninteractive providers and prepare the completed change for archive.
  - touches: `openspec/changes/harden-config-auth-safety/`
  - depends on: 4.2
  - Verify with `jcode auth doctor openai --validate --json`, `jcode auth doctor claude --validate --json`, and `openspec validate harden-config-auth-safety --strict --no-interactive`; the expected result is successful live validation for OpenAI and Claude and a zero-exit strict OpenSpec validation. Archive follows only after the terminal Copilot gate also passes.

## User Gate

- [ ] 5.1 [user:post] Complete the normal Copilot browser or device-code login if no valid local approval evidence can restore it.
  - touches: provider-managed credential state only
  - depends on: 4.2
  - Run `jcode auth login copilot` and complete the provider prompt; the expected result is that `jcode auth doctor copilot --validate --json` reports the provider configured and the live request valid without adding unverified trust.
