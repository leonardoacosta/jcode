## 1. Config
- [ ] 1.1 Add `workspaces` rules to config types with validation and tests
## 2. Detection
- [ ] 2.1 Origin lookup, credential stripping, glob matching; tests incl. no-origin
## 3. Identity
- [ ] 3.1 Cached `user.email` check keyed by repo root + mtimes; tests for hit, miss, unset
## 4. TUI
- [ ] 4.1 Header badge with cell-width truncation; render tests at narrow widths
## 5. Prompt
- [ ] 5.1 Bounded hint block injection; snapshot test
## 6. Verify
- [ ] 6.1 Runtime check via isolated socket build
## 7. Workspace hooks and swarm (depends on add-lifecycle-hooks, add-role-model-slots)
- [ ] 7.1 Per-workspace `hooks` merged after global `[hooks]`; tests for override and append
- [ ] 7.2 Per-workspace `swarm_model` and role-slot overrides; precedence explicit spawn `model` > workspace > `agents.swarm_model`
- [ ] 7.3 Pass resolved workspace to spawned workers in `comm_session.rs`; test worker inherits coordinator workspace
- [ ] 7.4 `openspec validate workspace-aware-context --strict`
