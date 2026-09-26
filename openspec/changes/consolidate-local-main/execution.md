# Execution evidence

## Coordination correction, 2026-09-26
The user explicitly instructed this session to manage merge conflicts using a swarm. Prior live-process observations did not identify any source files being written. HEAD, index and tracked diffs were unchanged across recorded observations. No claim of cross-harness lease ownership is made. The coordinator owns all main/index/ref changes and builds; workers initially analyze disjoint capability groups read-only. Before each mutation, verify recorded source baselines and stop on actual unexpected drift. Do not create another integration branch.

## Recovery
Protected recovery snapshot: `/home/nyaptor/.jcode/scratch/consolidate-main-recovery-20260926T132806`. Both history bundles passed verification. Primary and deployment dirty/untracked files were restored in detached scratch clones. Both stash objects were independently packed/restored. Source stashes and dirty worktrees remain intact. Snapshot reports record 16 branch tips and 1837 stash comparisons, not 1837 unique custom features.

## Primary-history merge in progress
Target is existing local `main` in `/tmp/jcode-systemone-main`, starting tip `3ddd145211680121c39b6e7dd65a1f39d57e2226`. Merge source `56a302f121726883340738278cb38f198e75be60`. Actual `git merge --no-commit --no-ff` produced only `crates/jcode-base/src/systemone.rs` add/add conflict. Rustfmt-normalized production sources match. Resolution retains main's file and four regression tests. Staged whitespace check passed. Actual merged-worktree tests are running before commit. Merge is not yet committed or validated as complete.

## v0.88 reconciliation
Unrelated-history rehearsal produced 609 conflicting paths. Reports are retained in recovery storage. Workers are analyzing backend, UI/tooling and history provenance. No blanket ours/theirs resolution is authorized. Final acceptance remains open.

## Primary batch verification and repairs
Initial app-core compilation exposed six errors already present in the combined committed feature surface: remote desktop registration referred to absent module files and absent configuration; unused Agent question fields referred to an absent type; poke shadow called a removed service resolver. Removed only disconnected remote-desktop registration and unused question fields, retaining their unfinished source in original worktree/backups. Poke now uses the existing shared System One adapter. No completed remote-desktop implementation was removed from this target.

Fresh checks on actual main worktree: code-intelligence Python 3 passed; base System One 4 passed; schema-dialect 39 passed; build-support 52 passed; app-core automation 42 passed; poke-shadow 22 passed; System One service crate 4 passed. The app-core `evaluate` filter matched zero tests and is NOT claimed as coverage. Compiler warnings remain; final full combined-tree acceptance is still pending.

Swarm assignments were issued for backend, UI/tooling and history analysis. Several workers encountered repeated provider failures; coordinator performed the bounded compile repairs rather than treating worker availability as a merge blocker.

## Historical baseline established and v0.88 merge started
Primary imported root `59588ecc` differs from upstream v0.77 `b7d98c9cb` in exactly seven paths: README, TUI commands/helpers/navigation, command tests, URL rendering and interaction-link tests. Those are source-editor-link and repository-aware release-prompt custom changes, all retained as deltas against the explicit upstream comparison base. `b7d98c9cb` is an ancestor of target `d8de03c2a`; it is a comparison base, NOT a claimed Git ancestor of the imported local root.

`merge-tree --merge-base=b7d98c9cb HEAD d8de03c2a` computes a three-way tree preserving all main custom deltas and upstream evolution, with nine content conflicts instead of 609 add/add conflicts. Started actual unrelated-history merge and populated that computed tree; real merge parents remain main `c9382edfa` and `d8de03c2a`. Reviewed source conflicts retain both additive declarations, preserve browser custom jev_select plus upstream browser lifecycle/handoff, and use Default initializers for new browser fields. Lockfile uses upstream lock as seed and combined manifests to resolve added dependencies. Offline lock regeneration lacked a cached dependency; online cargo check is running. Final target merge is uncommitted pending compilation and tests.

## Combined CLI compilation
`cargo check -p jcode --bin jcode` passed on combined v0.88 tree after fixing a duplicate browser goal field, restoring completed poke-shadow state/initializers, and removing disconnected unfinished question UI references whose state/module only exists in original dirty work. This is compilation evidence, not end-to-end acceptance. The original primary dirty work and stashes were not changed. Broad base/app-core/TUI lib tests are running; v0.88 merge remains uncommitted until verification resolves.

## Broad verification findings
First combined app-core run: 1506 passed, 7 failed, 13 ignored. Fixed actual schema compatibility regressions in evaluate (missing array items/untyped criteria), shortened descriptions to registry limits, and updated explicit open-world tool expectations for evaluate's dynamic question map. Browser goal remains available when handoff is disabled because custom jev_select needs it; regression now asserts that behavior. Registry sweep rerun: 59 passed, 1 ignored.

Second broad run: app-core 1512 passed, one wake timeout, 13 ignored; base/TUI exposed additional failures. Generated artifacts hit /tmp quota and were moved to disk-backed scratch. Initial destination omitted a literal `target` path component, breaking existing test-harness detection and allowing host configuration/auth leakage in tests. Corrected layout to scratch/consolidate-main-build/target; exact browser-suppression harness regression passes. Do not treat the affected broad-run auth/prompt failures as source regressions until clean-layout rerun finishes. A serial full rerun is in progress. No deployment changes made.

## Environment-clean verification
Full app-core now passes 1513 tests, 13 ignored. Clean scratch HOME and a credential-scrubbed child environment reduced the base suite from 10 failures to exactly one concurrent voice-transport fixture failure (1610 passed, 6 ignored). The fixture assumed alternating entitlement/decision requests, but upstream concurrently evaluates batches. Fixed the test server to answer Jcode requests by method and requested question IDs, retaining real entitlement preflight coverage rather than removing the subscription provider. Exact regression now passes. Clean-home full base/TUI run is active, log clean-home-base-tui-final.log.

Pending after the v0.88 lineage merge: integrate separate System One routing commit b4f04fb86 and reviewed deployed subscription delta; final feature accounting; clean-main build/runtime/deploy; cleanup. Do not claim those are included just because the upstream .88 lineage compiles.

## Verified v0.88 lineage merge
Actual combined-tree suites passed: app-core 1513 passed/13 ignored, base 1611 passed/6 ignored, TUI 2413 passed/17 ignored. Base/TUI used scratch HOME with inherited provider credentials removed; no source tests skipped beyond their existing ignored markers. CLI cargo check passed. Source-editor links, local custom additions and both Git parent histories retained. Separate deployed routing delta and final artifact/runtime acceptance remain pending.

## Named System One merge verified
Merged b4f04fb86 routing history with explicit deployed Jcode subscription selector/key/endpoint regression additions. Retained concurrent entitlement transport fixture instead of weakening coverage. Duplicate config field removed while preserving legacy alias and custom model setting. Conflict-resolution script briefly truncated two uncommitted files; immediately recovered full default template and complete test tail from verified HEAD, then reran checks. Fresh Jev/memory tests: 56 passed/1 ignored; public memory tool tests: 5 passed; full CLI check passed. All fixes are in the existing main merge, no new branch.
