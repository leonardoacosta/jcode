## 1. Runtime

- [x] 1.1 Verify swarm spawn/message/await/stop entry points in `crates/jcode-app-core/src/tool/communicate.rs` and dependency changes landed
- [x] 1.2 Carry persisted council-seat identity through swarm spawn; install the fixed allowlist before execution and resume
- [x] 1.3 Add `council.rs` with registry, caps, barrier, vote parsing, early stop

## 2. Tool surface

- [x] 2.1 Implement `open`/`round`/`say`/`log`/`close`/`run` and register tool
- [x] 2.2 Verdict call returning PASS|PARTIAL, tally, dissent; optional `council` model slot
- [x] 2.3 Persist transcript JSON per round under session dir

## 3. Lifecycle

- [x] 3.1 Stop seats on close/abort/error via drop guard
- [x] 3.2 Stop owner-less council seats in reload recovery

## 4. Verification

- [x] 4.1 Tests: seat-count rejection, call cap, timeout naming, early stop, split-vote verdict, edit rejected
- [x] 4.2 Runtime check: isolated real-server socket with deterministic provider, 3 seats, unchanged fixture `git status`, exactly 7 provider calls, no live seats after, opening cancellation also removes seats
- [x] 4.3 Enforce an aggregate provider-call budget across allowed-tool follow-up completions and provider retries, not only council prompt dispatches

### Verification evidence (2026-11-02)

All Cargo checks ran through `selfdev test` under the compile lock, with real exit status preserved. Final request `selfdev-build-437ad78a11414a4da78ffe6f2b635d03` exited 0:

- `cargo test -p jcode-app-core council --lib`: 14 passed. Includes real executor denials for mutation, batch, MCP and custom-tool escapes, prepared visible/headless policy, session resume, durable orphan recovery, isolated socket run and cancellation.
- `cargo test -p jcode-app-core server::comm_session:: --lib`: 37 passed.
- `cargo test -p jcode-app-core server::swarm_persistence:: --lib`: 25 passed.
- `cargo test -p jcode-app-core agent::tests::agent_ --lib`: 2 passed.
- `cargo test -p jcode-protocol --lib`: 98 passed.
- `cargo test -p jcode-base session --lib`: 183 passed, 2 ignored.
- `cargo check -p jcode --bin jcode`: passed in locked request `selfdev-build-32570becb46c4b61ad921d8ad586988a`, no daemon reload.

Final-diff recheck `selfdev-build-aa40426c3df045eda1591c77739e5c89` also exited 0: all 14 council tests passed, then `cargo check -p jcode --bin jcode` passed. `openspec validate add-council-tool --strict --json` and `git diff --check` passed.

The additional broad `cargo test -p jcode-app-core agent::tests:: --lib` check had 95 passed, 1 failed, 1 ignored. The failing existing `skill_installed_mid_session_keeps_system_prompt_stable_and_is_announced_once` assertion observed 2 announcements instead of 1. An exact-test rerun also failed. A separate tracked-source `git archive ec007f60f` fixture, tested with an explicit `--manifest-path` under the same compile lock (`selfdev-build-c15da21971d74ecea8dda564d8b8e693`), reproduced the same failure at its original line 3216. This failure predates the council diff and remains outside this scoped fix.

The isolated workflow initially measured 11 provider calls despite a council counter of 7. Disabling council memory work reduced the measured count to 7. That earlier fixture verified simple prompts only. Task 4.3 evidence below covers aggregate follow-ups and retries. No live-provider smoke, shared-daemon reload or push ran.

Recovery uses persisted council identity, never a task-label guess. Pre-marker legacy seats cannot be safely identified automatically. Failed durable closure is logged. A failed spawn RPC before the seat ID reaches the caller may leave a seat until reload recovery.

### Task 4.3 evidence (2026-11-02)

Preflight: approved task 4.3 only, existing proposal/design/specs inspected alongside `1666d6386`. Exact shared regions were coordinated with the role-slots-internals worker. Existing read-only policy, concurrent MCP work and branch were preserved. No new protocol/session budget fields were needed: seats and judge reuse persisted `council_id`.

Regression-first check `cargo test -p jcode-app-core council_provider_budget_covers_allowed_tool_followups_and_judge --lib -- --nocapture` failed before implementation with `allowed-tool follow-ups exceeded the shared provider-call budget` (`selfdev-build-1be41f80cab14a31a12cbd54fa595562`, exit 101). Earlier fixture/parallel-integration compile failures were fixed before this behavioral red result.

Fresh final Rust gates ran through `selfdev test`, under the compile lock, using `set -e` without an output pipe. Request `selfdev-build-0246386567794681b55fd684dc1d968a` exited 0:

| Command | Result |
| --- | --- |
| `cargo test -p jcode-provider-core --features tokio-tungstenite/handshake provider_call_budget --lib` | 5 passed, including six concurrent child processes sharing eight permits, retry/follow-up sharing, missing/corrupt state and failure/cancellation without refunds |
| `cargo test -p jcode-app-core council --lib` | 14 passed, including actual allowed-read execution until exhaustion, a later judge denied by the same budget, unchanged evidence, and the isolated three-seat socket/cleanup test |
| `cargo test -p jcode-provider-openrouter-runtime --lib` | 197 passed, 1 ignored live smoke requiring provider credentials. Real loopback HTTP 503 retries sent exactly two requests under a two-call cap, then blocked retry and later judge completion |
| `cargo test -p jcode-provider-openai-runtime council_provider_budget --lib` | 1 passed: exhausted websocket inference retries cannot send another frame, Ping/Pong control frames remain uncharged |
| `cargo test -p jcode-provider-core --features tokio-tungstenite/handshake transport:: --lib` | 6 passed |
| `cargo check -p jcode` | Passed, including all changed provider integrations |

`openspec validate add-council-tool --strict` and `git diff --check` passed. The explicit handshake feature is needed by existing standalone provider-core websocket tests; the first command without it failed on pre-existing handshake-gated test imports. No Cargo feature/dependency changes were made for that test setup.

The cap now uses atomic, non-refundable filesystem permits across processes, seat/judge turns, allowed-tool completions, automatic compaction and provider retries. Consumed permits are conservatively reported as `calls`, including reservations lost to preparation failures/cancellation. They are not exact successful-request telemetry. Missing state fails closed. Cursor CLI, ChatGPT browser and Bedrock SDK council routes fail closed because hidden calls/retries cannot be metered at this boundary. Idle council prewarming is disabled. No live-provider request, daemon reload, stash/reset or push ran.

This records the initial task 4.3 implementation, not final closure. Independent review of `da30d346b` returned NOT READY and reopened 4.3. The proposal is not archived. The previously recorded skill-announcement baseline failure and recovery limitations above remain outside this task.

### Independent-review follow-up (2026-11-02)

The reviewer identified a background-compaction budget escape through `runtime_activity::ActivityGate::spawn`, acceptance of `max_calls` above the specified formula, and suppressed transcript persistence errors. The repairs preserve the budget in the shared admission/spawn helper, reject formula-exceeding caps before spawn, and propagate all persistence errors while cleanup guards remain active.

Regression-first request `selfdev-build-f1099438222943078ecdcbe2efffff4e` recorded exit 101 for the upper-cap and transcript-directory regressions. Its compaction test first needed a fixture-only Debug-bound correction. Corrected request `selfdev-build-fd880027063349abb04d7278bbe2d204` then failed behaviorally with `native background compaction escaped budget (automatic=false)`, exit 101. First repaired request `selfdev-build-3d9ddc7b73814ae5ac464bb872ea2048` passed background compaction, runtime admission and all 15 council tests.

The background regression exercises manual and automatic compaction with exhausted valid three-call shared budgets, native/text/opaque routes, and a retry denied after spending the last remaining permit. The isolated server regression obstructs the transcript with a directory and an unwritable file, expects an operation error, and verifies both seats close, no members remain and the worktree fixture stays unchanged.

A broader parallel compaction run (`selfdev-build-64e63b157104438ba3c501c9f57f574e`) found existing shared-config test interference: the max-context test temporarily sets a 50,000-token cap while two older estimate tests do not take its environment lock. Result: 45 passed, 2 failed, 2 ignored. No unrelated estimator/config/test assertions were changed. The new regression takes the existing environment lock; the full compaction gate is rerun with `--test-threads=1` to avoid that global-config race. Fresh final repair request `selfdev-build-6ba758b5f0e443e58c3c38fc64546338` exited 0 under the `selfdev test` compile lock, with `set -e` and no output pipe. The cancellation fixture now delays only the cancellation probe, so persistence cases do not wait for artificial 30-second replies.

| Final repair command | Result |
| --- | --- |
| `cargo test -p jcode-base compaction --lib -- --test-threads=1` | 47 passed, 2 existing compatibility-fixture helpers ignored |
| `cargo test -p jcode-base runtime_activity --lib` | 3 passed, admission semantics preserved |
| `cargo test -p jcode-app-core council --lib` | 15 passed, including formula upper-bound refusal and directory/unwritable transcript failure cleanup |
| `cargo test -p jcode-provider-core --features tokio-tungstenite/handshake provider_call_budget --lib` | 5 passed |
| `cargo test -p jcode-provider-openrouter-runtime --lib` | 197 passed, 1 credential-dependent live smoke ignored |
| `cargo test -p jcode-provider-openai-runtime council_provider_budget --lib` | 1 passed |
| `cargo check -p jcode` | Passed |

Strict OpenSpec validation, scoped formatting checks and `git diff --check` passed. Independent reviewer bonehound returned final READY after confirming unchanged hashes for all six repair files and inspecting final gates `8225939xcm`, as confirmed by coordinator pig at 2026-11-02T17:50:44.896Z. Reviewed base: `798b12e5813a9a2cb6306ec5be776744d2823938`; pre-closeout six-file diff SHA-256: `ceeb7c717c9f01df42e51cbdfcfc6102480e1631282cd44fd814048595deffae`. Only this task completion/review evidence changed after approval. Task 4.3 is complete. Actual-commit review and proposal archival remain coordinator-owned and pending. No archive, push, shared-daemon reload, stash or reset was performed.
