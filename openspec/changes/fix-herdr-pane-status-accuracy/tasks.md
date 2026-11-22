# Tasks

Prerequisites for every Jcode task: this checkout, `scripts/dev_cargo.sh`, no credentials. Runtime checks (7.x) also need Herdr 0.9.5 or later on `PATH` and must use a disposable `herdr --session <name>` session, a private `JCODE_HOME` with `JCODE_TEST_SESSION=1`, and a private `--socket`. Never use the user's default Herdr session or daemon. Group 6 edits other repositories and needs their clean status checked first.

## 1. Protect the pane from tool commands

- [ ] 1.1 Add a bash-tool environment helper that applies the client terminal environment and removes `HERDR_ENV`, and use it at the foreground, background and detached spawn sites in `crates/jcode-app-core/src/tool/bash.rs`. Verify: extend `bash_tests/terminal_env.rs` so all three paths show `HERDR_ENV` removed while `HERDR_PANE_ID`, `HERDR_SOCKET_PATH` and `JCODE_CLIENT_HERDR_ENV` stay set; and show that a client snapshot without Herdr keys leaves the command environment as it is today. `scripts/dev_cargo.sh test -p jcode-app-core terminal_env` passes.
- [ ] 1.2 Verify hooks and terminal launch keep `HERDR_ENV`: existing `jcode-base` hook tests and the `jcode-terminal-launch` Herdr routing tests pass unchanged (`scripts/dev_cargo.sh test -p jcode-terminal-launch herdr` and `scripts/dev_cargo.sh test -p jcode-base hook`).

## 2. Carry the background wake flag to the client

- [ ] 2.1 Pass `wake` to `publish_task_started_activity` in `crates/jcode-base/src/background.rs` and add the fixed wake line to the started notification body only for wake tasks, with the header line unchanged. Verify: a unit test formats both variants and asserts the header is byte-identical.
- [ ] 2.2 Extend `ParsedBackgroundTaskStartedNotification` with `wake` and store it on `BackgroundTaskRow` through `upsert_running_background_task_started`. Verify: parser tests for the wake line, its absence and a legacy body; the existing test `test_background_task_started_activity_creates_running_row_without_card` still passes and a new case asserts `wake` on the row.

## 3. Derive blocked and working state

- [ ] 3.1 Add `AgentState::Blocked` (`"blocked"`) and an optional sanitized message (control characters removed, at most 200 bytes at a character boundary) to `StateReport`, and make `report_args` emit `--message`. Verify: `herdr.rs` unit tests for argument order with and without a message, with a session and resume command, and for truncation of a multibyte reason.
- [ ] 3.2 Add the pure state function described in design.md decision 3 and call it from `sync_herdr_agent_state`. Verify: a table test covers each spec scenario: login todo gives blocked; blocked while processing; `blocked_by`/`waiting_on` only and swarm `blocked` give idle; owned `running`/`streaming`/`thinking` worker gives working; foreign-subtree worker gives idle; running wake task gives working; non-wake task gives idle; completed wake task gives idle; nothing gives idle.

## 4. Deliver reports until accepted

- [ ] 4.1 Split `last_submitted` into desired and delivered state and add worker retry with 1 s doubling to a 60 s cap, replacement by newer reports, and cancellation by release, keeping clear-before-replace ordering. Verify: tests with the existing `jcode-herdr-fake` shell fixture in `herdr.rs` tests that (a) fail the first N calls and then succeed, asserting retries stop after success; (b) change the desired state during backoff and assert the stale state is never sent after the change; (c) release during backoff and assert no report follows the release; (d) repeat an identical `sync` during backoff and assert no extra attempt; (e) fail the session-clear step and assert the new session is not reported before the clear succeeds. Use an injectable backoff base so tests finish in under 5 s.
- [ ] 4.2 Verify the resume compatibility fallback still works with retry: the existing tests `report_falls_back_to_state_only_when_herdr_rejects_resume` and `report_keeps_resume_after_non_usage_failure` pass, plus a new case where a timeout precedes the usage error.

## 5. Diagnosable failures

- [ ] 5.1 Capture Herdr stderr (at most 512 bytes) in `run_herdr` and log command kind, state, sequence, status or timeout, and the captured text. Verify: the fake Herdr fixture prints `resume_not_accepted` and exits 1, and the test asserts the formatted log line; a second fixture prints 4 KiB and the test asserts the captured length is at most 512 bytes and the 2 s timeout still applies.

## 6. Documentation and skills

- [ ] 6.1 Rewrite `docs/HERDR.md`: the native reporting that ships (idle, working, blocked with message, retry), the conditions for each state, the bash tool environment rule with the `JCODE_CLIENT_HERDR_ENV` fallback, the observed Herdr 0.9.5 foreign-claim lock-out and its operator fallback, and remove the superseded "recommended first integration" hook plan. Verify: every command and path in the document exists (`rg` for each referenced symbol and path) and no text says Jcode never reports `blocked`.
- [ ] 6.2 Update the `HERDR_ENV` gates in `~/.agents/skills/herdr-handoff`, `herdr-open-git`, `herdr-open-markdown` (repository `~/.agents`) and `~/dev/personal/.agents/skills/herdr`, `herdr-jcode` (repository `~/dev/personal`) to accept `JCODE_CLIENT_HERDR_ENV=1` as well. Do not edit Herdr-generated integration files. Verify: in a shell with only `JCODE_CLIENT_HERDR_ENV=1` each documented gate command exits 0, and with neither variable set it exits 1. Requires: clean status of the touched skill paths in both repositories before editing; commit each repository separately.

## 7. Integration acceptance

- [ ] 7.1 Build with `selfdev build` and run the change's tests: `scripts/dev_cargo.sh test -p jcode-app-core terminal_env`, `scripts/dev_cargo.sh test -p jcode-base background`, `scripts/dev_cargo.sh test -p jcode-tui herdr`, `scripts/dev_cargo.sh test -p jcode-tui terminal_title`, plus `cargo fmt --check` and `cargo clippy` for the touched crates. Record the commands and results in this task's completion note.
- [ ] 7.2 Runtime acceptance in a disposable Herdr session with the built binary on a private socket and `JCODE_HOME`: (a) a bash tool command that runs a fake integration script gated on `HERDR_ENV=1` does not report, and `herdr pane get` still shows `agent: jcode`; (b) a `requires_user_auth` todo makes the pane `blocked`, and completing it returns `idle`; (c) a background `sleep 20` with `wake=true` keeps the pane `working` after the turn ends, then `idle`; (d) stopping the Herdr session for 10 s and starting it again leads to a delivered report. Record observations with timestamps. Delete the disposable Herdr session afterward.
- [ ] 7.3 `openspec validate fix-herdr-pane-status-accuracy --strict` passes, and the dependency validator passes for this change and `report-agent-lifecycle-to-herdr`.
