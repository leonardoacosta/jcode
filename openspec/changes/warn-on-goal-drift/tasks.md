# Tasks

Feature authoring only. No implementation task is completed here. No live transcript is authorized for remote judging by this proposal. Tasks use isolated `JCODE_HOME`, local MCP fixtures and no network unless explicitly stated.

## 1. Goal and monitoring contract

- [ ] 1.1 Trace every normal/streaming/headless/capture completed-turn path, goal attachment/update path and MCP policy gate; record exact symbols and verify none can bypass checks or duplicate a turn evaluation.
- [ ] 1.2 Add opt-in config, exact classifier identity and session-bound minimal state using existing goal/config/session types; add tests for defaults, missing goal, explicit opt-in, exclusions, changed goal and cancellation. Verify `JCODE_REMOTE_CARGO=0 scripts/dev_cargo.sh test --profile selfdev -p jcode-base goal` plus new focused config/session tests. Document data disclosure and no-enforcement semantics in the same phase.

## 2. Bounded classifier and advisory state

- [ ] 2.1 Reuse native MCP transport behind a strict Jev classify adapter with explicit permitted tool/schema, minimized 2,000-character context and item bounds, secret handling, full response validation and no discovery/fallback. Verify a local fake MCP server receives the intended bounded payload and denied tools never receive a call; test malformed/truncated/ambiguous responses and sentinel non-disclosure.
- [ ] 2.2 Integrate single-flight 2-second completed-turn checks without agent-lock/stream stalls or monitor recursion. Add tests covering all inventoried turn paths, timeouts, shutdown, stale goal hashes and no unbounded queue; verify `JCODE_REMOTE_CARGO=0 scripts/dev_cargo.sh test --profile selfdev -p jcode-app-core goal_drift`.
- [ ] 2.3 Implement comparable-streak, uncertainty reset and deduplicated advisory logic with configured confidence/margin; verify fixtures for legitimate phase changes, repeated drift, explicit new user direction, supporting work, missing evidence and prompt injection. Report fixture outcomes, not calibrated model accuracy. Document status and limitations.

## 3. User and agent feedback

- [ ] 3.1 Reuse goal controls and protocol/bus events for advisory display, refocus, dismiss and explicit update. Add functional keyboard tests, headless output tests, same-warning agent-next-turn tests, resume/dismissal tests and no-auto-turn assertions; verify warnings never change continuation, scheduling or permission decisions.
- [ ] 3.2 Verify TUI frames with `debug_socket` at narrow widths, Unicode and monochrome, then run `JCODE_REMOTE_CARGO=0 scripts/dev_cargo.sh test --profile selfdev -p jcode-tui goal_drift`; document exact supported actions and make every shipped control work. No shared-daemon restart without scoped activation authority.

## 4. Fresh verification and closeout

- [ ] 4.1 Run final-diff config/goal/session/MCP and feature tests, `cargo fmt --all -- --check`, `git diff --check`, `openspec validate warn-on-goal-drift --strict`, and canonical dependency validation. Record exact commands, results and diff binding. An optional authorized synthetic live Jev smoke is separate from required offline gates; absent access must be reported, not replaced with a private transcript submission.
- [ ] 4.2 Obtain independent final implementation review after fresh gates; repair actionable findings and rerun affected gates and review. Verify acceptance scenarios map to observed tests, including noninterference and all turn paths.
- [ ] 4.3 Update verified task evidence, commit scoped implementation only after resolving current branch policy, archive natively after review/gates/commit, and verify archive/promotion/history and any required authorized persistence. No external publication, provider changes or live config activation included by default.

## Prerequisites and handoff

Local implementation/tests require existing Rust/Cargo/OpenSpec and native MCP fixture support. No credentials for offline checks. Live activation requires the user-selected trusted MCP server/tool, an available permitted connection and explicit disclosure opt-in; discovery alone proves none of these. Independent review and actual feature acceptance remain future gates. Root `AGENTS.md`, checkout `.agents/AGENTS.md` and shared preserve-checkout rules must be reconciled before Git mutations: current `main` differs from local `origin/HEAD -> dev`. No branch switch or worktree is authorized here.

Acceptance mapping: explicit goal/consent → 1.1–1.2/2.1; bounded evaluation → 2.1–2.2; sustained advisory → 2.3; controls → 3.1–3.2; minimal evidence/noninterference → 2.1–3.1/4.1. `tasks.md` remains the execution ledger. Recheck source/artifact hashes and route/tool contract before Apply.
