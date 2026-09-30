## 1. Establish recoverable baseline
- [x] 1.1 Recheck main, working tree, active agents and Git operations. Record actual baseline, configured identity, fork tracking, and upstream target.
- [x] 1.2 Create a unique recovery ref. Add or verify upstream remote, fetch master and v0.89.3 without overwriting tags, and verify the pinned release commit. Preserve origin/main tracking.

## 2. Integrate release (depends on 1)
- [x] 2.1 Inventory changes on both sides and identify local capability regression coverage.
- [x] 2.2 Merge pinned v0.89.3 into existing main without committing immediately. Resolve conflicts by reviewing both behaviors and preserving local capabilities. Seek decisions for incompatible behavior.
- [x] 2.3 Review the full staged integration diff and confirm only owned changes are staged. Verify baseline and release ancestry after the merge commit is created.

## 3. Validate and commit (depends on 2)
- [x] 3.1 Run formatting, affected tests, and relevant full test gate through coordinated tooling. Fix synchronization regressions, documenting exact commands and results.
- [x] 3.2 Build the TUI with selfdev tooling. Verify the built version and exercise the built binary on an isolated socket or tester without replacing the shared daemon.
- [x] 3.3 Commit the verified merge and compatibility fixes using configured identity. Confirm both parents' ancestry, clean owned paths, and unchanged origin/main tracking.

## 4. Report and close (depends on 3)
- [x] 4.1 Report commit ID, recovery ref, source version, retained capabilities, test and runtime evidence, and any limitations.
- [x] 4.2 Document a repeatable upstream-release fetch, verify, recovery-ref, merge, and validation procedure in canonical execution evidence. Do not push or deploy automatically.
