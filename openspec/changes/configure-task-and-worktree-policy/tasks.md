## 1. Existing config and caller inventory

- [x] 1.1 Trace global/environment/project config precedence, all native task admission entry points and all SDK/native worktree creation callers. Record which project declarations are actually supported.
- [x] 1.2 Define typed task/worktree policy, compatibility defaults, validation and provenance without changing the user's live config. Keep unattended dispatch disabled and worktree creation explicit by default.

## 2. Deterministic task admission

- [ ] 2.1 Reuse existing scope, dependency, cancellation, budget and attempt ownership data to produce ready/waiting/blocked/needs-human with reasons; do not create a second planning ledger.
- [ ] 2.2 Revalidate before dispatch/restart and on relevant policy drift; preserve narrower persisted authorization and prevent duplicate live attempts. Provide a read-only effective-policy/admission view.

## 3. Worktree policy integration

- [x] 3.1 Reuse the existing allocator behind one destination/authority resolver; validate repository identity, configured creator identity, sanitized names, uniqueness, symlinks and root containment.
- [x] 3.2 Enforce safe reuse checks and manual cleanup; preserve dirty/owned worktrees. Project declarations may narrow policy but cannot grant host setup or publication effects.

## 4. Verification and closeout

- [x] 4.1 Run `cargo test --profile selfdev -p jcode-base config` and `cargo test --profile selfdev -p jcode-config-types`; test old config, invalid limits, effective provenance and project attempts to widen restrictions.
- [x] 4.2 Run `cargo test --profile selfdev -p jcode-sdk worktrees` with disposable Git fixtures only; test traversal/symlink escape, missing creator, matching reuse, dirty work and live-owner conflicts without touching user worktrees.
- [ ] 4.3 Run focused admission/restart tests in `jcode-app-core`: outside clock window, cancelled tasks, exhausted limits, dependency waiting, policy drift and duplicate owner. Record exact new test names and results.
- [ ] 4.4 Run `openspec validate configure-task-and-worktree-policy --strict` and dependency validation; independently review the final implementation after fresh gates, repair findings, commit and archive through the normal workflow. Report blocked live integration separately.

## Verification evidence and incomplete slices (2026-11-03 UTC)

Inventory/design and requirement mapping: [design.md](design.md). All Cargo commands below used `scripts/dev_cargo.sh`, preserving host-wide compile serialization. Disposable fixtures only. No live TOML, user worktrees, providers, activation, Git or archive operations.

- 1.1/1.2: existing global file then supported environment overrides, no project policy loader. New policy has no environment override layer. Strict parsing rejects invalid policy. Provenance distinguishes defaults from explicit global fields. Unsupported project task/worktree/setup declarations reject rather than widen authority.
- 2.1 remains incomplete: typed `TasksConfig::admit` implements deterministic ready/waiting/blocked/needs-human with original-bound intersection, exact scope, cancellation/dependencies/workspace/budget/owner reasons. Existing overnight manifest now has optional legacy-compatible snapshot and native revalidation. Missing atomic reservation, authoritative spend accounting and task/effects/dependency binding prevent a complete actual admission path. No second ledger added.
- 2.2 remains incomplete: `jcode policy [--admission <JSON>]` provides read-only effective field provenance and explicitly diagnostic reasons. Supervisor checks current strict policy and original snapshot before provider preflight and each future snapshot-bearing turn, then fails closed because authoritative attempt/spend reservation is absent. Manual `/overnight` remains unchanged. There is no existing automatic restart entrypoint, and no automatic restart is invented here. Existing live-owner snapshot returns waiting but is not an atomic duplicate-attempt lock.
- 3.1/3.2: one SDK `create_with_policy` resolver enforces authority, creator/repository/base, root/layout, all-ancestor symlink rejection, owner metadata and clean exact-base reuse. Legacy ungoverned `create` refuses with an actionable error. No cleanup/reset/clean/stash/switch/delete added. Git checkout hooks explicitly disabled. Every SDK Git invocation, including the safe config probe, passes `-c core.fsmonitor=false`. Real disposable reuse sentinel failed before the override, then passed without executing the configured fsmonitor executable. Every SDK Git operation first probes effective config and refuses any configured filter driver or unverifiable filter policy, including status/reuse/list. A disposable `.gitattributes` sentinel regression first failed because smudge checkout was accepted, then passed for smudge/clean/process refusal without executing their commands. Caller supplies trusted consumer checks; no runtime consumer source exists in this SDK-only allocator boundary.

Fresh gates:

| Command | Observed result |
| --- | --- |
| `scripts/dev_cargo.sh test --profile selfdev -p jcode-base config` | 183 unit tests + 2 matching integration tests passed |
| `scripts/dev_cargo.sh test --profile selfdev -p jcode-config-types` | 28 passed |
| `scripts/dev_cargo.sh test --profile selfdev -p jcode-sdk worktrees` | 14 passed after configured-filter refusal and fsmonitor suppression |
| `scripts/dev_cargo.sh test --profile selfdev -p jcode-app-core --lib task_admission` | 2 passed |
| `scripts/dev_cargo.sh test --profile selfdev -p jcode-overnight-core` | 8 passed |
| `scripts/dev_cargo.sh test --profile selfdev -p jcode --test cli_exit_status` | 7 passed, including real diagnostic snapshot and byte-identical fixture config preservation |
| `openspec validate configure-task-and-worktree-policy --strict` | Passed |
| `/usr/bin/python3 /home/nyaptor/dev/personal/.agents/scripts/openspec/validate_dependencies.py --root <this-worktree> --all --json` | Passed: `valid: true`, no issues, 44 active changes |
| `python3 scripts/check_dependency_boundaries.py` | Passed |
| `scripts/dev_cargo.sh clippy --profile selfdev -p jcode-config-types --all-targets -- -D warnings` | Passed after direct let-chain and fixture struct-initializer fixes, no lint allowances; all 28 type tests rerun passed |
| `scripts/dev_cargo.sh clippy --profile selfdev -p jcode-sdk --all-targets -- -D warnings` | Passed after filter and fsmonitor repairs, no lint allowances |
| `git diff --check` | Passed |

Exact new coverage:
- `config_task_policy_old_config_and_field_provenance`, `config_task_policy_invalid_limits_and_project_overrides_rejected`.
- `task_policy_defaults_fail_closed_and_limits_validate`, `admission_has_no_clock_window_and_never_widens_authorization`, `admission_explains_disabled_unknown_cancelled_dependencies_and_duplicate`, `admission_resource_deadline_setup_and_ownership_edges`.
- `worktrees_policy_requires_authority_identity_and_safe_names`, `worktrees_policy_reuse_checks_dirty_base_and_live_owner`, `worktrees_policy_rejects_ancestor_symlink_escape`, `worktrees_policy_preserves_existing_unowned_destination_and_remote_credentials`, `worktrees_policy_rejects_symlink_owner_metadata`, `worktrees_policy_does_not_execute_repository_checkout_hooks`, `worktrees_policy_refuses_configured_filters_before_creation_and_reuse`, `worktrees_policy_reuse_does_not_execute_fsmonitor`.
- `task_admission_native_manifest_revalidates_cancel_owner_and_policy`, `task_admission_legacy_manifest_has_no_implicit_authority`. These exercise the existing native manifest/revalidation guard, not a nonexistent auto-restart executor. Therefore 4.3 stays open for actual restart coverage.
- `policy_cli_is_read_only_and_defaults_fail_closed`, `policy_cli_diagnostic_file_failure_is_read_only`, `policy_cli_preserves_fixture_config_and_diagnostic_snapshot` (passed).

Red evidence observed before implementation: missing policy types/APIs, missing native revalidation/guard, ownership metadata symlink accepted, repository checkout hook executed without authority, and process-level policy CLI created application state before the startup early-return fix. Shared WIP compilation failures were reported to their file owners and repaired before CLI acceptance ran.

4.4 remains open: parent independent review/commit/archive not performed by worker, explicitly outside worker authority. The canonical dependency validator was supplied by the coordinator and passed with explicit root (command above). This proposal declares `execution.depends_on: []`. `openspec doctor --json` separately reports missing `openspec/config.yaml`/`.yml` in this checkout, but is not the canonical dependency validator and did not justify initialization. Live executor/monetary accounting/restart acceptance remains blocked on its implementation consumers, not provider credentials.
