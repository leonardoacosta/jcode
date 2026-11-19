## 1. Config
- [x] 1.1 Add `workspaces` rules to config types with validation and tests
## 2. Detection
- [x] 2.1 Origin lookup, credential stripping, glob matching; tests incl. no-origin
## 3. Identity
- [x] 3.1 Cached `user.email` check keyed by repo root + mtimes; tests for hit, miss, unset
## 4. TUI
- [x] 4.1 Header badge with cell-width truncation; render tests at narrow widths
## 5. Prompt
- [x] 5.1 Bounded hint block injection; snapshot test
## 6. Verify
- [x] 6.1 Runtime check via isolated socket build
  - Evidence 2026-11-19: private socket + dedicated JCODE_HOME + JCODE_TEST_SESSION=1, see `scratch/applyall/runtime-workspace-aware.log` (command, binary sha, observed lines). Workspace hook marker appeared only after a tool call in the matched repo; `# Workspace` block present for the brown session and absent for the plain one. Header badge not observable headless: covered by the jcode-tui header test.
## 7. Workspace hooks and swarm (depends on add-lifecycle-hooks, add-role-model-slots)
- [x] 7.1 Per-workspace `hooks` merged after global `[hooks]`; tests for override and append
- [x] 7.2 Per-workspace `swarm_model` and role-slot overrides; precedence explicit spawn `model` > workspace > `agents.swarm_model`
- [x] 7.3 Pass coordinator context as provenance and verify worker's actual target repository/workspace before applying it. Test same-repository worktree inheritance, different-workspace rejection, unknown target, changed cwd and config-source freshness. No cross-workspace hook/tracker/identity authority is inherited from the coordinator alone.
- [x] 7.4 `openspec validate workspace-aware-context --strict`
