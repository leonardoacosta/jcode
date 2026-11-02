## 1. Existing config and caller inventory

- [ ] 1.1 Trace global/environment/project config precedence, all native task admission entry points and all SDK/native worktree creation callers. Record which project declarations are actually supported.
- [ ] 1.2 Define typed task/worktree policy, compatibility defaults, validation and provenance without changing the user's live config. Keep unattended dispatch disabled and worktree creation explicit by default.

## 2. Deterministic task admission

- [ ] 2.1 Reuse existing scope, dependency, cancellation, budget and attempt ownership data to produce ready/waiting/blocked/needs-human with reasons; do not create a second planning ledger.
- [ ] 2.2 Revalidate before dispatch/restart and on relevant policy drift; preserve narrower persisted authorization and prevent duplicate live attempts. Provide a read-only effective-policy/admission view.

## 3. Worktree policy integration

- [ ] 3.1 Reuse the existing allocator behind one destination/authority resolver; validate repository identity, configured creator identity, sanitized names, uniqueness, symlinks and root containment.
- [ ] 3.2 Enforce safe reuse checks and manual cleanup; preserve dirty/owned worktrees. Project declarations may narrow policy but cannot grant host setup or publication effects.

## 4. Verification and closeout

- [ ] 4.1 Run `cargo test --profile selfdev -p jcode-base config` and `cargo test --profile selfdev -p jcode-config-types`; test old config, invalid limits, effective provenance and project attempts to widen restrictions.
- [ ] 4.2 Run `cargo test --profile selfdev -p jcode-sdk worktrees` with disposable Git fixtures only; test traversal/symlink escape, missing creator, matching reuse, dirty work and live-owner conflicts without touching user worktrees.
- [ ] 4.3 Run focused admission/restart tests in `jcode-app-core`: outside clock window, cancelled tasks, exhausted limits, dependency waiting, policy drift and duplicate owner. Record exact new test names and results.
- [ ] 4.4 Run `openspec validate configure-task-and-worktree-policy --strict` and dependency validation; independently review the final implementation after fresh gates, repair findings, commit and archive through the normal workflow. Report blocked live integration separately.
