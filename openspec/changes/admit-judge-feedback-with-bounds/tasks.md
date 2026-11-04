# Tasks

- [x] Add a judge DM parser for the prefix and single-message rule.
- [x] Add the per-turn continuation counter and config cap.
- [x] Mark admitted feedback as advisory in the injected message.
- [x] Ensure `/review` results never auto-continue.
- [x] Add tests: valid continue, stop, cap reached, malformed, duplicate.
- [x] Run `cargo test -p jcode-tui` and `cargo test -p jcode-base` covering the new success and failure scenarios.
- [x] Run `openspec validate admit-judge-feedback-with-bounds --strict`.

## Implementation verification (2026-11-03)

- Test-first RED observed for missing config default, missing admission API, absent advisory wrapper in the actual DM handler, missing genuine-user reset, missing user-interleave reset, unbounded ledger, server-wake budget refund, missing recorded TUI feedback role, and visible-worker parent identity.
- GREEN: `cargo test -p jcode-base --lib judge_feedback_` (5 tests), `cargo test -p jcode-app-core --lib judge_feedback_` (3 tests), `cargo test -p jcode-app-core --lib server_initiated_turns_always_use_account_router` (1 test), `cargo test -p jcode-app-core --lib prepare_visible_` (7 tests), and `cargo test -p jcode-tui --lib judge_feedback_` (2 tests). Commands used coordinated `selfdev test` with `JCODE_REMOTE_CARGO=0`. These are offline regression tests, not paid/live-provider acceptance.
- Full `cargo test -p jcode-base`: 2027 passed, 2 failed, 8 ignored. The unrelated MCP cross-origin and provider-catalog localhost tests both passed when rerun individually. Full `cargo test -p jcode-tui` showed failures and stalled on shared render-state tests. Its background task was cancelled after preserving logs. The package-gate task remains unchecked pending coordinator isolation/revalidation.
- Strict selected-change validation passed. Integration gates, independent review, commit, archive, and public-wire acceptance remain coordinator-owned. No Git operations, shared-daemon changes, or paid-provider checks were performed by this worker.


## Final integrated acceptance and independent review (2026-11-04)

Supersedes earlier open package-gate notes without removing their failed evidence. Whole integrated gates used `scripts/dev_cargo.sh test --profile selfdev` with isolated child HOME/provider environment, `CARGO_INCREMENTAL=0`, short owned TMPDIR and serial broad suites. Final TUI: 2,918 passed,20 ignored after one unchanged-source scroll-test failure (isolated1 and full rerun passed); base:2,029 plus isolated proxy1 passed,8 ignored. Other final gates: config, SDK, overnight, provider-core209/1ignored, protocol105, todo-policy, runtime helper9, CLI process, Bash, batch, root all-target check and selected strict Clippy passed. Selected strict OpenSpec/dependencies valid. No failing or zero-test attempt is counted as passing coverage.

Actual candidate `54f920477-dirty-f25806dd8390` was built without shared-daemon reload. Private public-wire/PTY tests passed judge advice/notes/cap/first-DM and distinct-home idempotent human reset including actual nonempty started turns; plan write/shell denial, allowed reads, ordinary write restoration and cancellation; actual new TUI/old server UNENFORCED notice. Real public `App::run` PTY integration verified 413/image recovery retains plan denial and next ordinary write succeeds (no shipped CLI local-mode entry exists). Actual busy human queue preserves user content/reset while synthetic reminder does not refund. Current private version/Ping names actual serving executable and reports dirty/unverified evidence mismatched, never a clean match.

Independent reviewer humpback approved the complete selected two-proposal/P01 technical boundary after full source, scope and test-integrity passes. All seven findings R1-R7 were routed to original developers, repaired with negative tests and re-reviewed. Final reviewed tracked diff SHA256 `a7a7a2ec822f7df416b3c472f2602276a1e6182bbf9302c6663c3301e8ba5211`; new module hashes separately reviewed. Local evidence index: `scratch/p01-final-acceptance/scenario-map.json` (all7 selected scenarios), results/logs and actual artifact roots. No paid inference, shared activation or OS-sandbox claim. Native archive and persistence follow this verified implementation commit.
