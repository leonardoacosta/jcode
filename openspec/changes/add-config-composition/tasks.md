# Tasks

All tasks are implementation work, not completed by authoring this proposal. Execute groups in order. `tasks.md` is the only execution ledger. Recheck the source revision, shared-file ownership and execution authority before Apply. No task edits the live `~/.jcode/config.toml` or restarts the shared daemon.

## 1. Source resolution and deterministic merge

- [x] 1.1 Inventory config loads, saves, setters, migrations, fingerprints and edit-watch callers in `jcode-base` and `jcode-app-core`; verify every root-only assumption has an implementation owner in this task list and record exact caller/test paths in research.
- [x] 1.2 Add root manifest parsing, canonical contained path resolution and raw TOML merge before typed defaults/environment application. Add `config_composition` tests for explicit order, cwd independence, sibling preservation, arrays/empty arrays, final type validation, conflicts, unknown keys, absent manifest, missing/unreadable sources, nested/self/duplicate includes and traversal/symlink escape; verify with `JCODE_REMOTE_CARGO=0 scripts/dev_cargo.sh test --profile selfdev -p jcode-base config_composition` using local-only execution.
- [x] 1.3 Document supported syntax, precedence and error behavior alongside the existing generated/default config documentation without changing existing installs; verify every documented source-selection and merge example against the config API in isolated `JCODE_HOME` fixtures.

## 2. Sparse preference saves

- [x] 2.1 Reuse `toml_edit` for file-only changed-leaf root overrides. Integrate setters, direct `Config::save` callers and root-only migrations. Add `config_composition` tests for legacy/composed saves, comments/unknown tables, provider catalogs, manifest preservation, no-op/new-file sparse writes, environment isolation and unrepresentable optional removal; verify focused tests pass and fragments remain byte-identical.
- [x] 2.2 Implement atomic same-directory root replacement, existing permission/symlink preservation and source-drift checks. Add fault/conflict fixtures covering invalid baseline, detected concurrent fragment/root edits, write/replace failure and owned-temp cleanup; verify original bytes and permissions survive each failed operation through focused config tests.
- [x] 2.3 Document root overrides, manual override removal and the narrow arbitrary-editor race limitation; verify docs examples with real setters and reloads in fixtures, including a round trip retaining currently unknown `[tasks]` and `[worktrees]` tables.

## 3. Reload, auth and tool feedback integration

- [x] 3.1 Make main reload and trusted-auth memoization observe the full source set/content and missing/error states. Preserve existing throttle, listeners, generation, legacy normalization and context-limit refresh. Add focused tests for equal-length fragment edits, root membership changes, deletion/restoration, immediate auth revocation and strict denial of auth/hook/unattended policy actions on invalid composition despite non-strict UI fallback; verify via `JCODE_REMOTE_CARGO=0 scripts/dev_cargo.sh test --profile selfdev -p jcode-base config` with local-only execution.
- [x] 3.2 Extend `ConfigEditWatch` and config-edit notices to selected fragments and effective before/after changes. Add `config_edit_notice` tests for active/inactive files, masked edits, manifest edits, invalid fragments and multi-file tool writes. Verify with `JCODE_REMOTE_CARGO=0 scripts/dev_cargo.sh test --profile selfdev -p jcode-app-core config_edit_notice`; ensure notices describe actual fallback and never echo secret values.
- [x] 3.3 Add nonsecret split-layout and single-file equivalence fixtures plus opt-in adoption/downgrade documentation. Verify identical typed results, unchanged user files and no automatic directory discovery through focused config tests. Document restart-only settings without claiming they reload live.

> Review-fix note (2026-11-19): independent review M1-M5 and minors m4/m7/m9/m13 were repaired with tests; evidence in research.md "Review-fix evidence". Known limits kept checked-honest: `jcode-setup-hints` and `jcode-sdk` launch links still read only the root file (documented in `docs/CONFIG_COMPOSITION.md`), so 1.1's "every root-only assumption has an owner" is satisfied by routing `router_transfer` and recording those three as documented limits.

## 4. Integrated verification and finalization

- [x] 4.1 Run fresh final-diff gates: `cargo fmt --all -- --check`, local-only `JCODE_REMOTE_CARGO=0 scripts/dev_cargo.sh test --profile selfdev -p jcode-base`, local-only `JCODE_REMOTE_CARGO=0 scripts/dev_cargo.sh test --profile selfdev -p jcode-app-core config_edit_notice`, `git diff --check` and `openspec validate add-config-composition --strict`. Record commands/results and the final-diff identity in research. Exercise every spec scenario, not compilation alone, and verify errors do not leak sentinel secrets.
- [ ] 4.2 Obtain an independent final review after fresh gates against requirements, caller inventory and final diff. Repair actionable findings and rerun affected gates and review. Verify review coverage includes auth freshness, environment persistence, atomic-save failures, and source/edit-watch integration.
- [ ] 4.3 Update task evidence only from observed checks, commit verified scoped implementation using configured identity, then archive through native OpenSpec and verify promoted specs, archive and commit history. Confirm any required authorized persistence. Reconcile checkout-policy conflicts before Git mutations; remote publication, live migration and shared-daemon restart are not included.

## Execution prerequisites and safe verification

| Work | Required tools and target | Access and dependencies | Stop boundary |
| --- | --- | --- | --- |
| 1.1–3.3 | Existing Rust toolchain, Cargo, `scripts/dev_cargo.sh`, `toml_edit`; this repository, disposable homes/files only | No credentials/network; groups ordered above; serialize overlapping config edits | Missing toolchain/resources blocks affected checks. Do not switch providers or use remote compilation as a substitute. |
| 4.1 | Same local tools, installed `openspec`, Git | Set `JCODE_REMOTE_CARGO=0` and verify wrapper selects local execution before running tests; use storage test-env lock for environment-mutating fixtures | Local test route is documented but not yet run for new code. Never test against live user config. |
| 4.2 | Separate native reviewer context with access to final diff and artifacts | Independent review availability must be observed at execution; current authoring is not an independent review | No valid review means implementation cannot complete/archive. |
| 4.3 | Git configured identity and installed native OpenSpec archive command | Current branch policy mismatch must be resolved before scoped Git mutations; no tracker binding, remote publication or destructive retirement required | Do not change branches, force-push, publish or migrate user files to clear a gate. |

A future Apply must refresh these prerequisites and final-diff gates. Feature authoring grants no implementation or live-config authority. No implementation task is checked here.

- Coordinator gate 2026-11-19 (4.1): `cargo fmt --all -- --check` clean on owned files; `git diff --check` clean; app-core `--lib -- --test-threads=1` 2260 passed/0 failed (includes `config_edit_notice`, `registry_execute_`); `jcode-config-types` 36 passed; jcode-base `--lib` run: failures only in `prompt::prompt_tests::{mermaid_prompt_module_follows_capability,project_system_prompt_file_replaces_default_base_prompt,test_skill_prompt_integration}` and `usage::*` which also fail on the pristine HEAD export (`scratch/applyall/base`), plus timing-flaky `mobile_sync`/`jev` tests that vary run to run on both trees. Rhino's `config` 259 / `hooks` 22 / `router_transfer` 7 filters pass. `openspec validate add-config-composition --strict` valid. Workspace clippy `-D warnings` exit 0.
