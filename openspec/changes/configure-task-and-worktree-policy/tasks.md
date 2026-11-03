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
- 2.1/2.2 remain partially open only for positive provider execution: explicit native `start_task_attempt`/`restart_task_attempt` now re-read strict current global policy, validate exact trusted runtime-root/run identity and real manifest/workspace, atomically serialize capacity and claim across processes, persist nonrefundable attempts/ownership/blocked dispatch decision in the existing manifest, preserve original scope/authorization/workspace and counters on restart, and refuse cancellation/dependency/exhaustion/legacy/duplicate owners. Stable lock sidecars are coordination, not a second ledger. Every existing manifest writer now uses locked monotonic merge, preventing stale saves from clearing cancellation, refunding counters or widening original scope/bounds. `jcode policy` remains read-only and diagnostic. Manual `/overnight` unchanged; no scan/auto-activation introduced.
- Exact remaining external dependency: provider transports expose no authoritative per-task pre-send monetary upper bound/reservation covering retries and side calls. `ProviderCallBudget` enforces counts only; session usage is post-request and price/local spend is estimated or aggregate. Every new native claim therefore persists `budget-unverifiable: provider lacks authoritative pre-send monetary bound; no provider/session effects dispatched`. No `ready` result is presented as execution. This is an implementation dependency, not missing credentials, approval or live activation. Positive monetized dispatch, hard spend enforcement and positive effects acceptance remain unverified and blocked.
- 3.1/3.2: one SDK `create_with_policy` resolver enforces authority, creator/repository/base, root/layout, all-ancestor symlink rejection, owner metadata and clean exact-base reuse. Legacy ungoverned `create` refuses with an actionable error. No cleanup/reset/clean/stash/switch/delete added. Git checkout hooks explicitly disabled. Every SDK Git invocation, including the safe config probe, passes `-c core.fsmonitor=false`. Real disposable reuse sentinel failed before the override, then passed without executing the configured fsmonitor executable. Every SDK Git operation first probes effective config and refuses any configured filter driver or unverifiable filter policy, including status/reuse/list. A disposable `.gitattributes` sentinel regression first failed because smudge checkout was accepted, then passed for smudge/clean/process refusal without executing their commands. Caller supplies trusted consumer checks; no runtime consumer source exists in this SDK-only allocator boundary.

Fresh gates:

| Command | Observed result |
| --- | --- |
| `scripts/dev_cargo.sh test --profile selfdev -p jcode-base config` | 183 unit tests + 2 matching integration tests passed |
| `scripts/dev_cargo.sh test --profile selfdev -p jcode-config-types` | 28 passed |
| `scripts/dev_cargo.sh test --profile selfdev -p jcode-sdk worktrees` | 14 passed after configured-filter refusal and fsmonitor suppression |
| `scripts/dev_cargo.sh test --profile selfdev -p jcode-app-core --lib overnight` | 21 passed after strict nonrecovering authority reads, including 13 admission-related tests |
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
- `task_admission_native_manifest_revalidates_cancel_owner_and_policy`, `task_admission_legacy_manifest_has_no_implicit_authority`; atomic native tests `task_admission_atomic_start_restart_preserves_authority_and_denies_unknown_spend`, `task_admission_atomic_race_has_one_live_owner`, `task_admission_atomic_live_owner_blocks_another_process_and_attempts_are_finite`, `task_admission_atomic_child_process_probe`, `task_admission_atomic_cancel_and_stale_save_cannot_refund_or_widen`, `task_admission_atomic_capacity_and_current_policy_narrowing`, `task_admission_atomic_completion_after_cancel_stays_terminal`, `task_admission_atomic_rejects_dependencies_exhaustion_legacy_and_workspace_drift`, `task_admission_atomic_public_boundary_rejects_external_path_and_accepts_sandbox_root`. All passed using real disposable manifest/OS locks including a separate child process. Positive provider dispatch stays blocked on the exact monetary contract above; 4.3 stays open for that acceptance.
- `policy_cli_is_read_only_and_defaults_fail_closed`, `policy_cli_diagnostic_file_failure_is_read_only`, `policy_cli_preserves_fixture_config_and_diagnostic_snapshot` (passed).

- Critical backup rollback repair: generic `storage::read_json` may restore an older `.bak`. All overnight authority/accounting manifest reads now use dedicated `read_manifest_at` (`std::fs::read` + `serde_json::from_slice`) without recovery or writes: public validation, locked current/merge, capacity candidates, supervisor `load_manifest`, `latest_manifest` and cancellation. `task_admission_atomic_corrupt_primary_never_restores_older_backup` and `task_admission_atomic_corrupt_capacity_candidate_is_not_recovered` both failed before repair and passed after it. Current attempt N plus older backup cannot refund counts; restart/cancel/stale save/load/latest refuse corruption and preserve primary+backup bytes. Generic storage behavior unchanged. Fresh exact gate log `native-corrupt-green.log`: 21 passed.

Red evidence observed before implementation: missing policy types/APIs, missing native revalidation/guard, ownership metadata symlink accepted, repository checkout hook executed without authority, and process-level policy CLI created application state before the startup early-return fix. Shared WIP compilation failures were reported to their file owners and repaired before CLI acceptance ran.

4.4 remains open: parent independent review/commit/archive not performed by worker, explicitly outside worker authority. The canonical dependency validator was supplied by the coordinator and passed with explicit root (command above). This proposal declares `execution.depends_on: []`. `openspec doctor --json` separately reports missing `openspec/config.yaml`/`.yml` in this checkout, but is not the canonical dependency validator and did not justify initialization. Live executor/monetary accounting/restart acceptance remains blocked on its implementation consumers, not provider credentials.

## Authorized monetary-contract continuation (2026-11-03 UTC)

This section supersedes the earlier statement that provider-core has no reservation interface. It does not claim paid-provider support or terminal completion. Existing tasks 2.1/2.2/4.3 include this approved dependency expansion. The native coordinator retains their final task status and integration evidence ownership.

Provider worker implemented the agreed monetary capability/reservation interfaces and explicit offline bounded fixture. Default live capabilities stay unsupported. The account uses the existing run namespace, original task/cap binding, durable narrowing, locked synced pre-send reservations, strict corrupt-state rejection, bounded actual settlement, and no refunds. No price estimate or user-supplied quote is authoritative. Synthetic adapter route is not registered or enabled in user config. Native integration is coordinator-owned.

Fresh local gates through `selfdev test` (host-coordinated Cargo):

| Command | Observed result |
| --- | --- |
| `cargo test -p jcode-provider-core --lib --features tokio-tungstenite/handshake` | 207 passed, 1 ignored after final provider changes |
| `cargo test -p jcode-provider-openrouter-runtime --lib monetary_budget_rejects_unsupported_provider_before_http` | 1 passed, zero loopback TCP connections |
| `cargo test -p jcode-provider-openrouter-runtime --lib council_provider_budget_stops_actual_http_retries_and_later_completion` | 1 passed, original two-request retry ceiling retained |
| `cargo clippy -p jcode-provider-core --all-targets --features tokio-tungstenite/handshake -- -D warnings` | Passed after consuming all returned fixture streams, no lint allowances |
| `openspec validate configure-task-and-worktree-policy --strict` | Passed after expanded design/task mapping |
| `python3 scripts/check_dependency_boundaries.py` | Passed |
| `git diff --check` | Passed |

Exact added tests in `provider_monetary_budget::tests`: `offline_completion_reserves_before_send_and_cannot_refund_or_widen`, `retries_children_side_calls_and_bare_spawn_share_one_account`, `request_bounds_and_nested_account_substitution_fail_closed`, `crash_reservations_and_invalid_settlement_never_refund`, `monetary_budget_child_process`, `concurrent_processes_never_overspend_original_account`, `incomplete_corrupt_symlinked_or_overflow_journal_fails_closed`, `unsupported_shared_transport_fails_before_tcp_connect`, `offline_adapter_preserves_existing_council_call_limit`, `aborted_operation_retains_synced_reservation`. Real API negative: `tests::wire_tests::monetary_budget_rejects_unsupported_provider_before_http`.

RED evidence: missing monetary types/API before implementation; explicit council compatibility test failed before adding the existing call-count consumption to offline sends. Real OpenRouter negative initially timed out because existing generic retry heuristics retried the permanent monetary refusal. Adding the early monetary guard before provider auth effects made the same actual-provider negative pass with no TCP connection. Full passing gates: background tasks `3599965qjt` (existing HTTP retry regression), `582000kasf` (final core/negative/lint). A reopened-account effective-ceiling assertion also failed before reporting the persisted narrowed value on `open`; send enforcement already used the durable journal ceiling. Real task abort leaves one unsettled unit and denies the resumed call. Running provider-core alone without its existing websocket handshake feature fails pre-existing websocket-test imports; the explicit feature above enables those existing tests without changing dependency features. Existing unmatched Cargo profile package warnings remain unrelated. No live provider requests, live config, user worktrees, Git/archive/publication, accounts, access or deployments occurred.
