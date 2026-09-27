## Execution gate

Full written revision approved by user at 2026-09-26T06:36:28Z. Execution claimed by this worker at 2026-09-26T06:37:43Z. Preserve unrelated dirty files. Acceptance may mutate disposable test profiles only, never personal profiles or the shared daemon.

## 1. Upstream contract and capability proof

- [x] 1.1 (depends: approval) Inspect upstream `jev-ultrafast` and `browser-harness` public source/docs. Evidence: upstream README and `pyproject.toml` fetched 2026-09-26; MIT license, Python >=3.12, `Agent(url, goals)` loop, `browser-harness==0.1.13`, and README statement that owned tabs share the existing Chrome profile. It does not document Jcode-compatible one-action APIs or prove isolated local profile state. This is a capability gap for the accepted isolation requirement.
- [x] 1.2 (depends: written revision review) Pin upstream revisions and verify scoped endpoint support in the actual dependency version. Prove a Jcode-managed local Chrome launcher can use two separate data directories with jev-ultrafast, persist state across restart, and fail closed on endpoint loss. A goal-oriented interface is acceptable; upstream-native profile CRUD and parity with every old action are not prerequisites. Record actual environment blockers without declaring unsupported behavior impossible. Evidence: clean upstream commit `1231850a0bf1a0c0341fe408ef1668dbbfdfac46`, `browser-harness==0.1.13`, Python 3.12.14. `python3 scratch/jev-profile-proof/proof.py` passed real two-directory localStorage isolation and persistence across four Chrome/subprocess launches. `python3 scratch/jev-profile-proof/dead.py` passed dead scoped endpoint failure without local discovery. Both used private `BH_RUNTIME_DIR` and explicit `BU_CDP_WS`. See `evidence.md`.

## 2. Profile contract

- [ ] 2.1 (depends: 1.2) After upstream proof, choose and document the public CLI/config controls, metadata schema, default policy, name rules, directory permissions, session binding, confirmation mechanism, and corruption recovery. Ensure normal Jcode startup makes no setup network call.
- [ ] 2.2 (depends: 2.1) Implement profile metadata and lifecycle with focused tests for valid/invalid/duplicate names, path traversal/symlink boundaries, persistence, malformed metadata, default changes, active-session deletion, cancellation, and restrictive permissions. Deletion tests use temporary roots and prove unrelated files remain untouched.

## 3. Replace browser runtime and remove legacy internals

- [ ] 3.1 (depends: 1.2,2.1) Implement the minimum adapter to the verified upstream interface. Expose agent profile create/list/inspect/select operations and default selection; bind each session to its initial profile. Validate arguments, bound subprocesses if applicable, clean up children on timeout/cancel, and redact diagnostics. Test runtime-missing, incompatible, malformed output, timeout, cancellation, and unsupported operation paths.
- [ ] 3.2 (depends: 3.1) Replace `crates/jcode-app-core/src/tool/browser.rs` schema/dispatch and associated tests with upstream-supported browser behavior plus safe profile selection. Remove unsupported Firefox/provider-specific actions; preserve only evidenced public compatibility requirements.
- [ ] 3.3 (depends: 3.2) Remove Firefox bridge helpers, dependencies, packaging/setup scripts and artifacts, tool registration glue, and browser-command rewriting in `tool/bash.rs`. Verify ordinary shell command execution is unchanged. Preserve user-installed legacy files and historical archived documentation.

## 4. Acceptance

- [ ] 4.1 (depends: 2.2,3.2,3.3) Run focused Rust tests for browser/profile/tool/shell paths and static searches proving no executable Firefox bridge references remain. Classify historical text separately from live code.
- [ ] 4.2 (depends: 4.1) Build with coordinated selfdev workflow. Run the newly built binary on an isolated socket/temp config against the verified upstream runtime. Exercise agent create/list/select/default, fresh empty state, two-profile persistence/isolation across restart, temporary cleanup on close and crash recovery, labeled attachment to a disposable signed-in profile, detach without data deletion, duplicate aliases, locked-profile refusal, endpoint loss without fallback, unknown profile, missing runtime, active deletion refusal, and confirmed managed deletion. Verify no legacy files changed and no shared daemon was restarted.
- [ ] 4.3 (depends: 4.2) Publish requirement-to-evidence results with exact commands, binary/socket/config locations, observations, and blocked external prerequisites. Commit only owned files after tests; never delete user bridge files or browser data.
