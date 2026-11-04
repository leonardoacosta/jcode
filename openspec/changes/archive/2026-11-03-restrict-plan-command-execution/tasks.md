# Tasks

- [x] Add a plan-only flag to the turn request and carry it from `commands_plan.rs` to the server.
- [x] Classify native tools as read-only or mutating and enforce denial in tool dispatch.
- [x] Add the read-only bash allowlist per the resolved open decision.
- [x] Surface an unenforced notice when the server does not acknowledge the flag.
- [x] Add tests: read allowed, edit denied with no file change, restriction cleared after completion and cancel, old-server notice.
- [x] Run `cargo test -p jcode-tui` and `cargo test -p jcode-base` covering the new success and failure scenarios.
- [x] Run `openspec validate restrict-plan-command-execution --strict`.

## Implementation evidence (2026-11-03)

Preflight: selected proposal is strict-valid, `execution.depends_on: []`, both defaults resolved. Implementation remains on existing `main` at `54f920477a1e32e71547f514ecee093d0340199f`; unrelated runtime files and other proposals are preserved. Coordinator owns whole-suite gates, public acceptance, independent review, commits and archive. No worker Git mutations, shared-daemon changes, live config writes or paid-provider calls.

- RED: `plan_mode_denies_mutation_and_restores_dispatch` failed with missing `PlanTurnGuard`/`PlanModeDenied` before implementation. Wire test failed with `Null` versus `true` before adding `plan_only`. Local TUI test failed with missing `next_turn_plan_only`. Autonomous-provider test failed because plan completion was not refused. Hosted-image test first failed with missing scoped policy API.
- GREEN: coordinated test task `937315h37r` exited 0: `cargo test -p jcode-app-core --lib plan_mode_ --no-default-features` (6 passed), `cargo test -p jcode-tui --lib plan_ --no-default-features` (39 passed), `cargo test -p jcode-protocol --lib plan_only_` (1 passed), `cargo test -p jcode-provider-core --lib plan_mode_` (1 passed), and `cargo test -p jcode-provider-openai-runtime --lib plan_mode_ --no-default-features` (1 passed). These are real registry/provider/TUI interfaces with synthetic fixtures, not public-daemon acceptance.
- `git diff --check` and `openspec validate restrict-plan-command-execution --strict` passed after formatting. Full TUI/base tests are intentionally unchecked pending coordinator gates.

## Policy boundary

`plan_only` is transient request metadata, not a prompt heuristic or persisted mode. The server sends `plan_mode { id, enforced: true }` before `ack`; a client receiving only `ack` displays **UNENFORCED**. A turn-owned guard covers retries, completion, errors, panics and cancellation. Unknown/custom/MCP tools are denied. Native read/search and web reads remain available. Browser observation does not auto-update or launch a browser. Executable tool/turn hooks are skipped. Providers that execute autonomous tools outside Jcode dispatch are refused before completion; OpenAI hosted image generation is omitted per request, including inherited provider tasks. Ordinary requests retain their previous behavior.

Bash is not an OS sandbox. It accepts one literal allowlisted command with explicit safe options, then executes trusted platform binaries directly without a shell, aliases, substitutions, redirects, inherited environment or user PATH wrappers. Git disables paging, fsmonitor, hooks, signatures, textconv and external diff; configured filter repositories fail closed. Output capture is bounded. Non-Unix plan bash fails closed with native read/search guidance. File access, network reads, telemetry/session persistence and process-level environment are not filesystem isolation guarantees.

## Public acceptance recipe (coordinator)

Build once without reloading the shared daemon. Start that exact binary against a private socket and isolated `JCODE_HOME`, using a loopback deterministic provider fixture. Submit `/plan` through a PTY or a raw `message` request with `plan_only: true`. The remote TUI carries synthetic plan text in `system_reminder` with empty `content`. Have the fixture return `read`/`agentgrep`, native edit/write/apply_patch, allowed bash, injected/unsafe bash options, MCP and browser mutation attempts. Verify successful reads, transcript plan denials and unchanged sentinel files. Send an ordinary turn and verify its write succeeds. Repeat after cancelling a held plan turn, and after a reconnect/reload input restore. Connect the new TUI to a legacy fixture that acknowledges without `plan_mode` and verify the visible UNENFORCED notice. Record exact binary/socket, commands, fixture requests, transcript and file hashes. No paid provider or shared-daemon mutation is required.


## Final integrated acceptance and independent review (2026-11-04)

Supersedes earlier open package-gate notes without removing their failed evidence. Whole integrated gates used `scripts/dev_cargo.sh test --profile selfdev` with isolated child HOME/provider environment, `CARGO_INCREMENTAL=0`, short owned TMPDIR and serial broad suites. Final TUI: 2,918 passed,20 ignored after one unchanged-source scroll-test failure (isolated1 and full rerun passed); base:2,029 plus isolated proxy1 passed,8 ignored. Other final gates: config, SDK, overnight, provider-core209/1ignored, protocol105, todo-policy, runtime helper9, CLI process, Bash, batch, root all-target check and selected strict Clippy passed. Selected strict OpenSpec/dependencies valid. No failing or zero-test attempt is counted as passing coverage.

Actual candidate `54f920477-dirty-f25806dd8390` was built without shared-daemon reload. Private public-wire/PTY tests passed judge advice/notes/cap/first-DM and distinct-home idempotent human reset including actual nonempty started turns; plan write/shell denial, allowed reads, ordinary write restoration and cancellation; actual new TUI/old server UNENFORCED notice. Real public `App::run` PTY integration verified 413/image recovery retains plan denial and next ordinary write succeeds (no shipped CLI local-mode entry exists). Actual busy human queue preserves user content/reset while synthetic reminder does not refund. Current private version/Ping names actual serving executable and reports dirty/unverified evidence mismatched, never a clean match.

Independent reviewer humpback approved the complete selected two-proposal/P01 technical boundary after full source, scope and test-integrity passes. All seven findings R1-R7 were routed to original developers, repaired with negative tests and re-reviewed. Final reviewed tracked diff SHA256 `a7a7a2ec822f7df416b3c472f2602276a1e6182bbf9302c6663c3301e8ba5211`; new module hashes separately reviewed. Local evidence index: `scratch/p01-final-acceptance/scenario-map.json` (all7 selected scenarios), results/logs and actual artifact roots. No paid inference, shared activation or OS-sandbox claim. Native archive and persistence follow this verified implementation commit.
