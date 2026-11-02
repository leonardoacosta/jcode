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
- [ ] 4.3 Enforce an aggregate provider-call budget across allowed-tool follow-up completions and provider retries, not only council prompt dispatches

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

The isolated workflow initially measured 11 provider calls despite a council counter of 7. Disabling council memory work reduced the measured count to 7. This verifies that workflow, not a universal API-call cap, hence 4.3 remains open. No live-provider smoke, shared-daemon reload or push ran.

Recovery uses persisted council identity, never a task-label guess. Pre-marker legacy seats cannot be safely identified automatically. Failed durable closure is logged. A failed spawn RPC before the seat ID reaches the caller may leave a seat until reload recovery.
