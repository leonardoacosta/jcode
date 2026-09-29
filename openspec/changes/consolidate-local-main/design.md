# Consolidate local main

## Approved outcome and scope

Use the existing local main as the sole integration branch. Preserve its history and as much completed user-authored custom work as possible, using v0.88 as the target code baseline. Do not create another integration branch. This is one repository-recovery change, not authorization to finish independent half-built features.

## Observed starting state

These are planning observations, not a current execution snapshot. Recheck them before mutation.

| Repository family | Worktree | Observed state |
| --- | --- | --- |
| v0.77 | `/home/nyaptor/.jcode` | `codex/systemone-config-routing`, 155 porcelain status entries, two stashes |
| v0.77 | `/tmp/jcode-systemone-main` | Existing clean local main at `3ddd14521` |
| v0.88 | `/home/nyaptor/dev/jcode-unified-history-20260924` | Clean base at `d8de03c2a`, no local main |
| v0.88 | `/home/nyaptor/dev/jcode-worktrees/systemone-config-088` | Deployment source at `d8de03c2a`, 11 dirty files |
| v0.88 | `/tmp/jcode-systemone-config-088` | Clean route commit `b4f04fb86` |

The observed repositories have distinct history roots. The v0.77 current branch and main diverged by 15 and 7 commits respectively. Enumerate all branches, not only these five worktrees. Counts do not establish authorship or completeness.

## Blockers and resolutions

| Blocker | Resolution and required evidence |
| --- | --- |
| Separate histories and versions | Verify graph ancestry and object availability. Import local history without checking out a new branch. Reconcile histories on existing main, with an explicitly reviewed unrelated-history merge if no common ancestor exists. Preserve both parents and review the resulting tree, not merely ancestry. |
| Overlapping route patches and ports | Compare patch equivalence and actual behavior. Integrate once, retain attribution and source-to-result mapping. Do not merge a duplicate implementation simply to check off a branch. |
| Dirty source, untracked files, two stashes | Capture binary patches and protected untracked-file archives separately from Git bundles. Stash objects and indexes require explicit backup. Verify recovery into scratch storage before changing main. Never blanket-add the runtime home directory or credential files. |
| Uncertain ownership or completeness | Inventory author and provenance per item. Completed user work enters main by default. Half-finished work gets a recoverable reference and remaining tasks. Unknown provenance or product decisions pause only the affected batch. |
| Active agents or changing source | Coordinate a mutation pause for affected repositories, record exact starting refs and fingerprints, and stop if they drift. Do not overwrite concurrent edits. |
| Old v0.77 APIs versus v0.88 | Resolve compatibility feature by feature against v0.88. No blanket ours/theirs selection. Retain tests that demonstrate original custom behavior. |
| Prior base-suite failures | Reproduce against pinned inputs, attribute each failure, and fix regressions. No silently skipped failure or broad green-suite claim. Any remaining release-blocking failure stops promotion. |
| Earlier dirty-build provenance | Previous helper wrote source metadata after building, so metadata alone does not prove source equality across the build. Replace that evidence with clean-main pre/post source identity, official build/publish guards, and artifact/runtime hashes. Never manufacture a sidecar to satisfy a guard. |
| Stale test activation metadata | Inspect pending `test-reload-hash` activation ownership and active sessions. Preserve a copy and use supported cleanup only once proven stale. Do not activate a test artifact. |
| Premature worktree deletion | Require item accounting, protected backups, clean disposable worktrees and no active process dependencies. Keep the common Git directory while registered worktrees depend on it. Unique commits alone do not require a worktree once reachable and recoverable. |

## Execution sequence

1. Capture a protected recovery snapshot outside tracked source, including refs, bundles, stashes, binary patches, untracked files and previous deployed executable/channel identities. Verify object integrity and restore representative dirty and untracked content. Record restoration commands.
2. Create `inventory.md` in this change during execution. Each row records source repository/ref/path, author, purpose, completeness, dependencies, classification, destination commit or duplicate proof, test evidence, and backup location. Classifications are complete, already included, superseded, or unfinished. Explicitly explain every non-inclusion.
3. Prepare the existing main worktree without resetting it. Protect any new dirt first. Reconcile the v0.88 history and then custom features in dependency order determined by the inventory. Keep prior main history reachable. Preserve commit authorship where merging and record original hashes when adapting source.
4. Integrate one coherent batch at a time. Run relevant checks before committing each batch. Stop at a failing batch, retain diagnostics, and abort only the operation whose starting state was recorded. Never reset unrelated work.
5. Verify all inventory rows and run final combined-tree gates. Commit resulting source and planning/evidence artifacts. Build from clean main and prove pre/post source equality. Test that artifact against an isolated socket without touching the shared daemon.
6. Record previous deployment channels, publish and promote using supported identity checks, gracefully reload, verify the actual process executable/hash and socket readiness, and exercise selected custom behavior. If startup or behavior fails, restore the previous version through supported promotion/reload and verify recovery.
7. Remove only redundant unused worktrees. Preserve required branches, backups and deferred work until their retention decision is explicit. Do not prune stashes or unique branch refs merely because the worktree is gone.

## Acceptance evidence

| Requirement | Check and pass condition |
| --- | --- |
| Preserve main history | Original main tip remains an ancestor of final main. Imported v0.88 lineage is represented by reviewed merge ancestry. |
| Preserve completed custom work | Every inventory row maps to a main commit with behavioral evidence, or verified duplicate/supersession. Unfinished rows have tested recovery references and remaining tasks. |
| Avoid branch proliferation | Before/after branch inventory shows no newly created integration branch. Backup bundles are recovery artifacts, not working branches. |
| Validate combined source | Focused route, memory, browser, voice, automation, AgentMail, wake/session tests as applicable to inventoried changes; relevant base/app-core/CLI suites and integration tests run on final main. Every failure has a recorded disposition, and unresolved release blockers prevent promotion. |
| Preserve routing semantics | Verify configured 9Router default, explicit Jcode subscription selection, direct provider selection, missing-key failure without account fallback, unknown-route rejection and entitlement checks using public callers and local HTTP fixtures. Do not rely on tests of obsolete test-only resolvers. |
| Build provenance | Clean committed main before build, unchanged commit/tree after build, supported publisher validation, identical tested/published artifact hashes. Evidence-only commits after build must be identified rather than claiming the artifact embeds that later commit. |
| Runtime result | Isolated socket accepts a real client, relevant custom smoke checks pass, graceful deployment succeeds, actual daemon executable matches promoted bytes, client handshake and session continuity succeed. Version text or symlinks alone are insufficient. |
| Cleanup safety | All removed worktrees have complete accounting and no unique unpreserved files, artifacts or active process dependencies. Backups and unfinished work remain recoverable. |

## Alternatives considered

History-preserving reconciliation is approved. Selective porting without joining histories risks hidden omissions. Replacing main with v0.88 risks losing local lineage and is rejected. Blind unrelated-history merging or blanket conflict resolution is not the approved strategy.

## Boundaries

No production API calls or paid account use solely for validation without necessity and authorization. No remote push. No unrelated refactoring. No secrets in tracked inventories or logs. Permission to consolidate does not authorize deleting unaccounted work. Written-plan approval is the next gate, then `apply consolidate-local-main` owns execution.

## Scoped continuation: evaluate and MCP images (2026-09-29)

This continuation does not reopen history reconciliation or authorize the remaining repository cleanup. Fresh inspection finds local main at `12020af75` and the dirty working checkout on `codex/systemone-config-routing`. The user approved the first six-file batch in conversation; the user approved this written continuation with “Approve all” on 2026-09-29.

### Alternatives and selected approach

1. **Selected: port the saved narrow runtime patch onto current main.** This preserves main's newer dispatcher and custom-tool collision handling while making the deployed fixes ordinary source commits. Recheck applicability against the actual main tip.
2. Merge the divergent old branch. Rejected: unrelated changes and old architecture would greatly expand scope.
3. Replace main files with deployed archive files. Rejected: whole-file replacement could silently remove newer code. The archive is comparison evidence, not an authoritative replacement tree.

### Source and boundaries

Baseline: `scratch/completion-20260929/safe-runtime.patch`, with acceptance/provenance recorded beside the installed `12020af75-systemone-images2` binary and in `scratch/completion-20260929/final-acceptance.json`. Verify hashes before use. Exactly these production files are in scope:

- `crates/jcode-base/src/systemone.rs`
- `crates/jcode-system-one/src/resolver.rs`
- `crates/jcode-app-core/src/system_one.rs`
- `crates/jcode-base/src/mcp/tool.rs`
- `crates/jcode-base/src/mcp/mod.rs`
- `crates/jcode-app-core/src/tool/mcp.rs`

Focused tests and these canonical workflow artifacts are allowed. No ChatGPT browser runtime work, unrelated refactor, provider substitution, purchase, remote push, other-repository merge, or orphan retirement belongs to this batch.

### Behavior and failure handling

System One's configured full Omni endpoint must resolve the appropriate credential profile by parsed origin and API-path boundary, not loose string prefix. Cover scheme, host, effective port and path mismatches, trailing slash behavior, and alias compatibility. Credentials must not be sent to an unrelated origin or sibling path. Keep explicit TypeSafe/OpenRouter selection, without fallback.

Eager MCP tools and deferred `McpCallTool` must use the same validated content conversion. Valid supported images become actual image output, with text preserved. Invalid base64, unsupported MIME and oversized content must not become trusted image output or panic. Preserve main's dispatcher, collision, custom-tool, and error semantics.

### Execution, verification and promotion

Preserve the current dirty checkout and refs. Use the existing main branch in an isolated clean worktree if needed, without creating another integration branch. Record the pre-integration main tip and reject concurrent unexpected changes. Never reset or stash unrelated work. Independently review the applied delta and regression tests before committing only scoped files.

Run focused resolver/credential and eager/deferred image regressions, relevant workspace checks, and a normal main-based build. Record command outcomes without treating previous runtime acceptance as proof of the new build. Use an isolated daemon socket and verify executable identity before runtime acceptance. Exercise real evaluate `noul`, `choice`, and `score` via the explicitly configured TypeSafe route and real deferred image output, plus eager-path regression coverage. Do not claim blind visual recognition from a known image fixture. A rejected OpenRouter call due to credits is an external limitation, not a successful paid-path acceptance.

Only promote after the build, scoped tests and isolated runtime acceptance pass. Retain the working binary and channel targets for rollback. Gracefully activate the validated build, confirm the actual shared daemon identity, then repeat evaluate and inline-image acceptance through the live public tool surface. If a gate fails, keep the known-working deployment, record the failure, and fix or stop without claiming completion. Broader consolidation tasks remain unchanged.
