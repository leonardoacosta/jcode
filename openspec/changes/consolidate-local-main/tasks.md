## 1. Written review and recovery snapshot
- [ ] 1.1 Obtain user review of this complete written plan. Design sections are approved, but execution has not started.
- [ ] 1.2 Re-inventory both repositories, all branches/worktrees/stashes, active sessions and source drift. Record exact refs, current deployed process/artifact and protected backup destination. Coordinate affected writers. Depends on 1.1.
- [ ] 1.3 Back up refs, stash objects, dirty/index patches and untracked content separately; verify bundle integrity and restore checks without exposing secrets. Record batch-abort and deployment rollback commands. Depends on 1.2.

## 2. Item accounting and merge order
- [ ] 2.1 Create inventory.md in this change with one accountable row per work item, including dependencies, authorship, completeness and destination/evidence fields. Inspect every branch and both stashes, not only checked-out tips. Depends on 1.3.
- [ ] 2.2 Classify complete, duplicate, superseded and unfinished work. Document recoverable deferrals and verify patch/behavior equivalence for duplicates. Isolate unresolved product/provenance questions rather than dropping work. Depends on 2.1.
- [ ] 2.3 Verify graph roots/common-base availability and choose exact history reconciliation and feature batch order. Capture original main tip and confirm no new integration branch is needed. Depends on 2.2.

## 3. Existing-main reconciliation
- [ ] 3.1 Protect any new main worktree dirt, import required local objects without new integration branches, reconcile v0.88 history with explicit per-conflict review, and verify retained main ancestry. Do not reset main or blanket-select either tree. Depends on 2.3.
- [ ] 3.2 Integrate complete custom batches in the inventoried dependency order, run focused behavioral checks before each commit, preserve attribution and populate source-to-commit mappings. Stop the affected batch on failure or drift. Depends on 3.1.
- [ ] 3.3 Verify every inventory row has a destination commit, proven duplicate/supersession or verified unfinished-work recovery reference. Include this proposal and other local documentation in accounting. Depends on 3.2.

## 4. Final-main verification and build
- [ ] 4.1 Run applicable base/app-core/CLI suites and focused custom-feature integration tests on combined main. Reproduce and resolve relevant failures, including previously reported base-suite failures; record exact commands, counts and exceptions. No unresolved release-blocking failure permits promotion. Depends on 3.3.
- [ ] 4.2 Verify public System One callers, selected-provider credentials, no fallback, entitlement checks and local transport fixtures. Include browser/voice/memory and applicable automation/AgentMail/session/wake coverage from inventory. Depends on 4.1.
- [ ] 4.3 Commit validated combined source, verify clean main, capture source identity, build through supported coordinated workflow, verify unchanged source identity and artifact hash. Do not stamp post-hoc provenance metadata. Depends on 4.2.
- [ ] 4.4 Run the exact artifact on an isolated socket and perform real-client handshake and required custom smoke checks. Stop only the owned test daemon. Record evidence outside tracked source during build validation. Depends on 4.3.

## 5. Deployment and cleanup
- [ ] 5.1 Verify prior deployed artifact is retained and stale activation metadata ownership is understood. Publish/promote validated artifact with supported guards, gracefully reload, verify actual daemon executable hash, socket readiness and session continuity. Roll back on failure. Depends on 4.4.
- [ ] 5.2 Audit worktree process usage, artifacts and item accounting. Remove only proven redundant inactive worktrees, retaining common Git directory dependencies, recovery backups and deferred work. Do not automatically delete stashes or branch refs. Depends on 5.1.
- [ ] 5.3 Record final main commit, deployed source commit/hash, complete item mapping, test outcomes, deferred items and removed/retained worktrees. Distinguish any subsequent evidence-only commit from the built source commit. Close only when each acceptance row in design.md has observed evidence. Depends on 5.2.

## 7. Scoped continuation: six-file evaluate/image batch (2026-09-29)

These tasks are independent of unfinished broad cleanup. Execution starts only after review of the scoped written continuation in design.md.

- [x] 7.1 Review and approve the written scoped continuation. Preserve earlier approval history without treating it as blanket approval of other batches.
- [x] 7.2 Recheck current main/dirty checkout/worktrees, verify patch provenance and protected recovery evidence, and isolate existing main without creating an integration branch. Depends on 7.1.
- [x] 7.3 Port only the six production-file patch onto current main, preserving newer dispatcher/collision/custom-tool behavior. Add focused credential boundary and eager/deferred image regression coverage. Depends on 7.2.
- [x] 7.4 Obtain independent swarm review, fix findings, run focused tests and relevant workspace checks, and commit only scoped changes. Record exact commands/results and source identity. Depends on 7.3.
- [x] 7.5 Build normal committed main and verify evaluate primitives plus image transport against an isolated socket using the new executable. Retain existing runtime unchanged on failure. Depends on 7.4.
- [x] 7.6 Gracefully activate only the validated build with rollback retained. Verify daemon identity and repeat real evaluate/image acceptance. Record source-to-binary evidence and defer all other batches explicitly. Depends on 7.5.

Phase 7 execution evidence: existing main worktree `/home/nyaptor/dev/jcode-worktrees/main-runtime-convergence` starts at `12020af75`. Original dirty checkout remains untouched. Saved patch SHA256 `c8e65166a77ecaf5cc1e455d0b3bce75f0b6ef1a612f383ffe449efc901b91db` matches `final-acceptance.json`. Root session owns integration/promotion, assigned swarm worker owns the six-file patch, independent reviewer owns review. Existing protected recovery archives remain under `scratch/worktree-integration-audit/`.

### Phase 7 verification checkpoint (2026-09-29)

- Source integrated on existing main in `afbbd12f5`. Six production files plus this change's three artifacts only. Independent swarm review found no production blocker. Original dirty checkout preserved.
- Fresh commands passed: `scripts/dev_cargo.sh test -p jcode-base --lib systemone::tests` (8), `scripts/dev_cargo.sh test -p jcode-base --lib mcp::tool::tests` (3), `scripts/dev_cargo.sh test -p jcode-app-core --lib tool::mcp::tests` (13). Deferred regression invokes actual `McpCallTool.execute` through a Python stdio fixture. Earlier test-only mutex deadlock and malformed fixture were corrected before commit.
- Clean committed-main build passed: `scripts/dev_cargo.sh build --profile selfdev -p jcode --bin jcode`. Executable reports `jcode v0.88.140-dev (afbbd12f5)`.
- Isolated daemon requires its own `XDG_RUNTIME_DIR`, not merely another socket filename. Started exact new executable with `JCODE_REPO_DIR` and `JCODE_DEBUG_CONTROL=1`, session `session_owl_1790702761700_d94218193125774a`.
- Real evaluate `noul`, `choice`, `score` all passed on isolated build, model `jev-1.13.0`. Owned image MCP fixture connected successfully.
- Real image-provider acceptance remains BLOCKED: explicitly configured `omni:gpt-6-luna` selected successfully, but actual model message returned `[400]: The requested model is not supported.` No silent provider substitution performed. Debug tool responses omit images and cannot substitute for this gate.
- No promotion or activation performed. Known-working `12020af75-systemone-images2` remains deployed. Tasks 7.5 and 7.6 remain incomplete until real image acceptance succeeds. Broader consolidation remains out of scope.

### Phase 7 completed acceptance

The earlier400 was resolved by using the already-configured qualified identifier `omni:cx/gpt-6-luna` for the same model, without provider substitution. Real isolated image invocation succeeded. Final clean build `a65876997` passed isolated evaluate and image checks, then official guarded debug reload published it. Installed/build binary SHA256 both `61980104737a46a5441794a20303f31507ab92a2510cccb50fda6c27d787d2f3`. Graceful shared reload completed, both existing daemon executable links resolve to `builds/versions/a65876997/jcode`. Current/shared channels match. Rollback binary `12020af75-systemone-images2` retained. Live current-session evaluate passed all three primitives and real deferred `mcp_call` delivered the inline512px image showing white8142 on blue. This proves image transport, not blind recognition. No remote push or other consolidation batch performed.
