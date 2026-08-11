## 1. Canonical Skill Boundaries

- [x] 1.1 Update `/home/nyaptor/dev/agents/skills/orca-cli/SKILL.md` to retain version-matched runtime mechanics and explicitly defer Jcode initiative, schedule, and durable-state policy to `jcode-command-center-orchestration`; verified by live Jcode skill loading and the focused deterministic contract check.
- [x] 1.2 Update `/home/nyaptor/dev/agents/skills/orchestration/SKILL.md` to retain generic supervised Run/Task/Dispatch coordination, clarify the full-handoff boundary, and remove every `llmtrim` reference; verified with `! grep -Rni --exclude-dir=.git 'llmtrim' skills/orca-cli skills/orchestration`.
- [ ] 1.3 Run canonical skill projection self-tests before live installation.
  - Run `bash scripts/reconcile-skill-projections.sh --self-test && bash scripts/verify-skill-projections.sh --self-test` from `/home/nyaptor/dev/agents`.
  - Expected result: both self-tests exit 0 without changing the live installed projections.

## 2. Focused Command Center Policy Skill

- [x] 2.1 Create `/home/nyaptor/dev/agents/skills/jcode-command-center-orchestration/SKILL.md` with a pushy but bounded trigger description for Jcode Command Center, initiatives, schedules, launch, retry, cancel, approval, handoff, and Orca lifecycle projection.
- [x] 2.2 Add progressive-disclosure references covering the five orchestration patterns, authority matrix, identifier envelope, lifecycle projection and replay, scheduling correlation, mutation capability table, degraded states, and acceptance evidence.
- [x] 2.3 Add `evals/evals.json` with realistic prompts and objective assertions for full handoff, supervised DAG work, observation-only projection, approval gates, scheduled retry, replay gaps, canonical identity ambiguity, Orca unavailability, unsupported mutation, and resource cleanup. Independent behavioral evaluation observed a 100% pass rate across all eight committed cases.
- [ ] 2.4 Register the new skill in `/home/nyaptor/dev/agents/skill-projections.json` and run `bash scripts/reconcile-skill-projections.sh --self-test && bash scripts/verify-skill-projections.sh --self-test`. Expected: both self-tests exit 0 and the manifest includes the focused skill without mutating live projections.
- [x] 2.5 Add `/home/nyaptor/dev/agents/scripts/test-jcode-command-center-orchestration-skill.sh` to validate skill structure, required references, trigger boundaries, pattern coverage, authority language, identifier fields, replay-gap assertions, unsupported-capability behavior, and the empty `llmtrim` audit. The committed script, shellcheck, and post-commit rerun all exit 0.

## 3. Runtime Identity Correction

- [ ] 3.1 Add or update focused tests in `/home/nyaptor/dev/jcode/source/jcode/crates/jcode-app-core/src/command_center.rs` proving Orca runtime ID cannot populate canonical project ID and unresolved canonical identity fails closed.
- [ ] 3.2 Change the Orca observation adapter to run the resolved version-matched `ORCA repo list --json`, match the current absolute repository path to exactly one canonical repository/project ID, and preserve runtime ID only as runtime metadata. If the command fails, its schema is unsupported, or zero/multiple matches remain, stop with unresolved canonical identity; do not add undocumented start, retry, or cancel commands.
- [ ] 3.3 Extend the Command Center identifier envelope and focused tests to preserve distinct Task, Dispatch, worktree, terminal, correlation, and idempotency identifiers in addition to Jcode and Orca run/project IDs.
- [ ] 3.4 Implement scoped replay invalidation, crash-safe idempotency reconciliation, verified capability projection, and partial-cleanup recovery states in the Command Center service and adapter surfaces.
- [ ] 3.5 Ensure scheduled triggers enter the same pattern-selection, permission, correlation, idempotency, and receipt-settlement path as interactive commands, with distinct causal dispatch attempts for retries.
- [ ] 3.6 Verify the focused Rust integration.
  - Run `cargo fmt -p jcode-app-core -- --check && cargo check -p jcode-app-core -p jcode-command-center && cargo test -p jcode-app-core command_center --lib && cargo test -p jcode-command-center --lib` from `/home/nyaptor/dev/jcode/source/jcode`.
  - Expected result: formatting and checks exit 0 and both focused test commands report zero failures.

## 4. Documentation and Initiative Projection

- [ ] 4.1 Update `docs/COMMAND_CENTER.md` and `docs/COMMAND_CENTER_MIGRATION_LEDGER.md` with the approved layered projection bridge, three-skill boundary, orchestration selection model, identifier rules, and migration status.
- [ ] 4.2 Add `scripts/test-command-center-architecture.sh` that serves `docs/diagrams/` on an isolated loopback port, launches Chromium at 393x852, asserts `scrollWidth == innerWidth`, asserts no fixed element intersects a section heading, captures `artifacts/command-center-architecture-mobile.png`, and shuts the server down. Run it and expect exit 0 plus the retained screenshot.
- [ ] 4.3 Replace the linked OpenSpec initiative's TBD sections with accepted requirements, approved decisions, affected repositories, dependencies, risks, and coordinated task status; verify the pre-implementation state with `openspec initiative show command-center-orchestration --store jcode`.

## 5. Acceptance and Persistence

- [ ] 5.1 Run and grade the focused skill evaluation iteration.
  - Run paired new-skill and no-policy baseline prompts into `/home/nyaptor/dev/agents/skills/jcode-command-center-orchestration-workspace/iteration-1`, save `eval_metadata.json`, `timing.json`, and `grading.json` per run, then run `python -m scripts.aggregate_benchmark /home/nyaptor/dev/agents/skills/jcode-command-center-orchestration-workspace/iteration-1 --skill-name jcode-command-center-orchestration` from `/home/nyaptor/.agents/skills/skill-creator`.
  - Expected result: all required policy assertions pass, `benchmark.json` and `benchmark.md` are generated, and the static review artifact is retained under the workspace.
- [ ] 5.2 Verify the final OpenSpec artifact set.
  - Run `openspec validate optimize-orca-command-center-orchestration --strict --no-interactive && bash /home/nyaptor/dev/codex/scripts/verify-codex-feature-artifacts.sh --root "$PWD" --change optimize-orca-command-center-orchestration --phase final` from `/home/nyaptor/dev/jcode/source/jcode`.
  - Expected result: strict validation exits 0 and every required deterministic verifier row reports `PASS`.
- [x] 5.3 Verify repository hygiene and obsolete-guidance removal for the implemented skill slice.
  - Observed: the agents commit diff check passed, the obsolete-guidance grep returned no matches, and only the eight intended skill/script paths were committed.
- [ ] 5.4 Commit only the canonical skill repository paths in `/home/nyaptor/dev/agents` and only the linked OpenSpec, Command Center documentation, diagram, and app-core paths in `/home/nyaptor/dev/jcode/source/jcode`; record both commit IDs in the initiative evidence.
- [ ] 5.5 Install the committed skill projection release.
  - Run `release=$(git -C /home/nyaptor/dev/agents rev-parse HEAD) && bash /home/nyaptor/dev/agents/scripts/reconcile-skill-projections.sh --write --verified-release "$release" --audited-release "$release" && bash /home/nyaptor/dev/agents/scripts/verify-skill-projections.sh`, then load `jcode-command-center-orchestration` through Jcode's skill interface.
  - Expected result: reconciliation and verification exit 0, the installed focused skill resolves to the committed canonical source, and Jcode reads its metadata and body.
- [ ] 5.6 Update the initiative task evidence with both containing commit IDs, rerun `openspec initiative show command-center-orchestration --store jcode`, and verify the final output names both repositories, both commits, completed verification, and any remaining follow-up work.
