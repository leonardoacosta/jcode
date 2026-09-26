## 1. Written review and recovery snapshot
- [x] 1.1 Written plan and execution approved by user; resumed with explicit swarm-managed integration instruction on 2026-09-26.
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
