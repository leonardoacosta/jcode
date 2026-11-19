# Worker-stop baseline and acceptance evidence

## Scope and preflight

Tasks 1.2/1.3 only. User authorized this baseline assignment on 2026-11-10. No production control changes, shared-daemon activation, Git mutations, external providers, credentials, historical workers or unrelated process controls were used by this worker. Coordinator owns remaining tasks and authoritative `tasks.md` updates. This record does not claim task 7.1 or proposal completion.

Repository: `/home/nyaptor/.jcode`, origin `https://github.com/leonardoacosta/jcode.git`, source HEAD `b37c875f1f1ddc4ceb34f5453d1ace35534dd973`, dirty. Existing preflight explicitly retains `main` despite local `origin/HEAD` pointing to `origin/dev`. Baseline eligibility: authorized and verified. Production implementation was held until the red result below. Final gates/review/commit/archive remain coordinator-owned and unproved here.

Owned paths: `tests/e2e/worker_stop.rs`, one `mod worker_stop;` registration in `tests/e2e/main.rs`, this evidence record. Assignment named `tests/e2e.rs`, but that file does not exist. Cargo actually discovers `tests/e2e/main.rs`. Creating the named root file would duplicate the target. Registration adjustment was announced to and accepted by the coordinator. No other files were edited by this worker.

`graft map` and `graft ask 'worker stop control swarm cancel binary integration test' --source` preceded source inspection. The installed graph search did not resolve the precise fixture detail, so existing test/protocol/source ranges were inspected. `graft map` reported an automatic one-file index refresh. No graph rebuild was requested. Existing `scratch/sdk-cancel-acceptance.py` supplied the loopback Responses fixture pattern. `tests/e2e/binary_integration.rs` supplied private runtime/home/install and owned spawned-process isolation. No production implementation was copied into the acceptance test.

## Entrypoint

Run through coordinated `selfdev test`, without reload:

```sh
JCODE_WORKER_STOP_BINARY=/home/nyaptor/.jcode/target/debug/jcode cargo test -p jcode --test e2e worker_stop::public_stop_prevents_post_stop_native_write -- --ignored --exact --nocapture
```

The test defaults to `target/selfdev/jcode` when `JCODE_WORKER_STOP_BINARY` is absent. Use the override only for a known actual built binary. A source/binary revision mismatch fails before worker creation. `cargo test` builds the actual debug executable in addition to the test harness, so the recorded command used that fresh executable with coordinator approval, not a mocked server and not the shared daemon. Subsequent final acceptance must use the newly built final-diff binary and fresh hashes.

The original `target/selfdev/jcode` reported `jcode v0.99.208-dev (7abc9a0da, dirty)`, SHA256 `59b438a64912ba746729813e534cfdf2085012b9c5797c5d348b5a1375a4961f`. It did not match source HEAD and was not used to claim the baseline. This provenance discrepancy was reported before execution.

Requirements: Linux user/network namespaces, `/usr/bin/unshare`, `/usr/sbin/ip`, `/usr/bin/timeout`, `/usr/bin/python3`, existing Rust toolchain. No installation is performed. Unsupported namespace creation fails closed, not a skip/pass.

### Frozen workload and oracle

1. Launch the real daemon with `--no-update --no-selfdev --provider openai-api --model gpt-5.4 --socket <owned>/runtime/native.sock serve --temporary-server --owner-pid <fixture-pid> --temp-idle-timeout-secs 60`. Verify `/proc/<owned-pid>/exe`, public Ping `server_runtime.binary_path`, `--version` and source revision.
2. Public `subscribe` creates the coordinator. Public `comm_spawn` creates only a fresh owned headless worker. Synthetic loopback inference sends a partial response and waits on an explicit fixture event. Public `comm_status` must show `activity.is_processing=true` before stop. Marker must not exist.
3. Public `comm_stop` request ID 5 targets that worker. Wait for terminal `done`, not `ack`. Recheck marker absence, then release the provider gate. The supplied response calls the actual native `write` tool, never a Python filesystem write of the marker.
4. Observe marker for 5 seconds. **Acceptance requires absence after terminal stop success.** If marker appears, fail with `BASELINE FAILURE: public CommStop returned Done while busy worker later executed native write`. This failing assertion is retained, not inverted or annotated `should_panic`.

Fixed deadlines: startup 15 s, public frame 10 s, busy provider gate 10 s, provider gate release 20 s, post-stop observation 5 s, owned daemon termination 5 s then kill/reap 5 s, outer process ceiling 90 s then 5 s kill escalation. Generation is recorded as absent/null on baseline, not fabricated. Stop pending/error is not accepted as terminal success by this test. Broader draining/deadline/generation scenarios remain required under tasks 2–7.

### Isolation and owned cleanup

- Outer process clears inherited environment. New user/network namespace has only loopback, verified via `ip -json link show`, no default route, and a failing connection probe to reserved `192.0.2.1:443`. External calls cannot escape that namespace, even if runtime code attempts discovery. No shared namespace fallback exists.
- Fresh temporary root contains private HOME, JCODE_HOME, XDG config/data/cache/runtime, JCODE_RUNTIME_DIR and JCODE_INSTALL_DIR. HOME has no user MCP, skill, login or credential state. Synthetic API token is a literal fixture string, not a credential. Updates, telemetry and memory sidecars are disabled.
- Socket/state targets are not caller arguments. Socket is minted under the owned root; checks reject an existing socket, symlink, or non-owned socket parent before connect. The harness cannot be pointed at a shared socket. Provider binds exactly `127.0.0.1`; only `/v1/models` and at most two `/v1/responses` fixture requests are accepted. No shared debug control is enabled or used.
- `finally` releases the gate, closes readers/socket, terminates and reaps only the subprocess handle it created, shuts down/closes the fixture server and joins its thread. `TemporaryDirectory` removes only its owned root on success or assertion failure. No name-based process killing, shared cleanup or unrelated file deletion occurs. Owner PID and temporary-server idle limit provide additional bounds.

## Observed red baseline

Both runs below compiled and executed through `selfdev test` and failed the intended behavior assertion, not compilation/setup. Each run used real public daemon control and native mutation. Inference alone was synthetic. No live-provider behavior is claimed.

| Observation | First reproduction | Final-safeguard repeat |
| --- | --- | --- |
| Tool task | `2693633uw6` | `469430phq0` |
| Completion UTC | 2026-11-10 14:18:23 | 2026-11-10 14:21:23 |
| Actual binary | `/home/nyaptor/.jcode/target/debug/jcode` | same path, freshly compiled debug profile |
| Version | `jcode v0.99.211-dev (b37c875f1, dirty)` | same version |
| Binary SHA256 | `cf3fddebca04bb6d279a258ee4d7dc3bb087754c6c812fdc34b1f253f3740d44` | `537a28311768cd067d8b19c7687d0540fc54879b12ba7ea6433dbec5248fa768` |
| Worker | `session_maple_1794320302894_c03d1d5b72b9f372` | `session_duckling_1794320482931_0bbec480742c7d2d` |
| Before stop | `status=running`, `activity.is_processing=true`, marker absent | same |
| Stop response | `{"id":5,"type":"done"}` | same |
| After stop | marker exists after 0.010 s | same |
| Marker SHA256 | `000281bd4f4ee0c05a35ccca5c815bf27411a048757d74745f091d3fa25cd769` | same |
| Generation | absent/null | absent/null |
| Provider requests | two `/v1/responses` | same |
| Result | 0 passed, 1 failed, exit 101 | same |

Public Ping in both runs: `binary_path=/home/nyaptor/.jcode/target/debug/jcode`, `git_hash=b37c875f1`, `dirty=true`, `cleanliness_verified=false`. Executable path also matched `/proc/<owned-pid>/exe`. Fresh post-observation Ping returned the same runtime identity. Dirty build identity is explicitly not proof of a clean tree.

Relevant source SHA256 values, identical in both runs:

| Path | SHA256 |
| --- | --- |
| `crates/jcode-app-core/src/server/comm_session.rs` | `8c7661d3f223105dbf08bcd6fe09710e1bd7f22d8e1423b79409d8e1eaf8eddd` |
| `crates/jcode-app-core/src/server/client_lightweight_control.rs` | `b7447b3355f3d0e07234a529fdfbfdbf96305dd46d752f3bbfdb0380c147a4cb` |
| `crates/jcode-app-core/src/server/live_turn.rs` | `590ee80730f15ce663889ac80151c4e5030a808ea8543abb16fe1cab2dd26079` |
| `crates/jcode-app-core/src/server/swarm/claims.rs` | `4818f73e39a881ee6a404797cef47208e02fd362ba844b5258c61c182aaf338b` |

Detailed tool outputs: `/tmp/jcode-bg-tasks/2693633uw6.output`, `/tmp/jcode-bg-tasks/469430phq0.output`. They contain exact JSON observations and cleanup confirmation. These are ephemeral host logs; essential evidence is preserved above. Coordinator received the proven red baseline immediately at 14:18:33 UTC, before production control edits.

Earlier setup-only failures were investigated, not counted as reproduction: `578332nhci` refused host-mounted `/sys/class/net` listings despite private network namespace, corrected to namespace-aware netlink; `825454yv8n` timed out on incorrect public event tag `session_id`, corrected to actual wire tag `session`. Neither altered the behavior oracle. Existing unrelated deprecated `fetch_update` compiler warning remains untouched.

### Final verification, 2026-11-10 14:24 UTC

After binding temporary fixtures to `<repo>/scratch` through `TMPDIR`, ran through `selfdev test`:

```sh
JCODE_WORKER_STOP_BINARY=/home/nyaptor/.jcode/target/debug/jcode cargo test -p jcode --test e2e worker_stop:: -- --include-ignored --nocapture
```

Task `664796enkp`, exit 101: **1 passed, 1 failed**. `refuses_shared_network_before_starting_daemon` passed without namespace changes or daemon startup. The actual-binary acceptance again failed only the intended post-stop native-write assertion. Serving SHA256 `ccb5c00328aefe406fad7a1474fae7999217101021df923dcbfdd133bc5df344`, same version/source hashes as above; worker `session_whale_1794320678916_0ef5f6c9b57f29b9`; public stop `{"id":5,"type":"done"}`, marker appeared after 0.010 s with the same marker SHA256. Cleanup reported owned daemon reaped and fixture closed. Log: `/tmp/jcode-bg-tasks/664796enkp.output`.

`rustfmt --check --edition 2024 tests/e2e/worker_stop.rs`, embedded Python `compile(...)`, and focused `git diff --check` passed after final formatting. The only tracked test registration diff is one line. No production source edits or Git metadata mutations were made. The refusal regression is non-ignored; actual-binary regression stays explicit/ignored and intentionally red on this baseline.

## Limits and handoff

This proves exactly the selected busy-stop failure: terminal public success precedes actual native mutation. It does not prove idle/tool-busy settlement, durable restart barriers, ownership after membership churn, generation-bound waits, resume races, stale report replay or unresolved external-child handling. Tasks 2–7 must implement and verify those separately. The baseline remains intentionally red until production repair. Do not weaken its marker oracle, replace the real binary with in-process helpers, or treat ordinary ignored offline test success as this acceptance run. Final review/commit/archive and any activation remain separate gates.


## Refresh 2026-11-19: actual-binary acceptance against committed durable-stop control

Command (through `scripts/dev_cargo.sh`, no reload, private namespace/socket/JCODE_HOME as documented above): `JCODE_WORKER_STOP_BINARY=/home/nyaptor/.jcode/target/selfdev/jcode cargo test -p jcode --test e2e worker_stop:: -- --include-ignored --nocapture`. Binary `jcode v1.0.221-dev (177ac0abb, dirty)` sha256 `33341d764a219db78ace2d01e8465657f6e6f5eb031c7919ee3f5a50822fc649`, `/proc/<pid>/exe` and public Ping `binary_path` matched. Result: 2 passed. Worker `session_horse_1795066516986_ada7a02f42a9e534` was `running`/`is_processing=true` before stop; `comm_stop` returned `{"id":5,"type":"done"}`; native `write` marker did NOT appear within the 5.004 s observation window (baseline: marker after 0.010 s); provider saw one `/v1/responses` request; `generation` is still null on the wire (not exposed). This supersedes the red baseline for the busy-stop scenario only; other 7.1 scenarios remain unexercised. Fixture repair: removed `'model':'inherit'` from the `comm_spawn` request because the current daemon rejects agent-selected models.

## 5.2 partial: await cache key bound to generation vector (worker, 2026-11-19)

- `await_members_state::generation_bound_key(base, watched)` hashes each watched member's `last_allocated_generation` (or `-` / `unreadable`) into the persisted await key. `comm_await.rs` now derives the key after resolving watched members. A final response cached under an older generation can no longer satisfy a wait on newer work.
- Tests: `-p jcode-app-core --lib -- await` gave 30 passed (26 existing, 3 re-keyed, plus new `await_key_changes_when_watched_member_generation_advances`). Log: `scratch/await-gen.log`.
- Still open for 5.2: generation on completion reports, and per-request generation for queued work. 5.2 stays unchecked.

## 3.1 doc + 4.1/4.2 partial claim tests (worker, 2026-11-19)

- `docs/SWARM_ARCHITECTURE.md` gained a "Durable stop control" section with the lifecycle transition table (runnable, stop_requested, draining, stopped, unresolved), idempotency, fail-closed, claim-release and known-limit rules. It states that no production resume path exists yet (`ControlStore::resume` is `#[cfg(test)]`), which bears on 5.1.
- New tests in `server/swarm/claims.rs`: `revoked_generation_cannot_mutate_in_any_claim_mode` (Off/Warn/Block: write rejected, bytes unchanged) and `new_owner_may_claim_only_after_predecessor_settles` (claim held until settlement, then a new owner can reserve).
- Command: `scripts/dev_cargo.sh test -p jcode-app-core --lib -- swarm::claims`. Result: 13 passed, 0 failed. Log: `scratch/claims.log`.
- Not covered, so 4.1 and 4.2 stay unchecked: revoked patch/apply_patch tool paths (only `FileClaims::write` is exercised), a real native tool-busy stop integration test, and the automatic-reassignment guard.

## 4.1 follow-up: all native mutation tools share the gated path (worker, 2026-11-19)

- `rg` shows `tool/edit.rs`, `write.rs`, `patch.rs` and `apply_patch.rs` mutate only through `FileClaims::write` and `FileClaims::remove_file`. Both go through `FileClaims::mutate`, which reserves the current generation token and fails when the generation is revoked.
- `revoked_generation_cannot_mutate_in_any_claim_mode` now asserts both `write` and `remove_file` are rejected in Off, Warn and Block. `-p jcode-app-core --lib -- swarm::claims`: 13 passed. Log: `scratch/claims.log`.
- Not yet shown: per-tool tests that drive `patch`/`apply_patch` end to end with a revoked token. The shared-gate argument is by inspection. 4.1 stays unchecked.

## 2.2 / 3.3 partial: real-socket denial and idempotent stop (worker, 2026-11-19)

- `tests/e2e/worker_stop.rs` now also drives, over the private daemon socket: (a) `comm_stop` from an unrelated subscribed session against the busy worker, which must be denied and leave the worker `is_processing`; (b) a repeat `comm_stop` after the first `done`.
- Binary: rebuilt with `selfdev build` (no reload), `jcode v1.0.223-dev (52626ede5, dirty)`, sha256 `a641e193e2c42811e5668eecf25e84d71d625b57a0eb58f6468a344a1f084c55`. The harness's own revision-match guard rejected the earlier stale binary, then passed on this one.
- Command: `JCODE_WORKER_STOP_BINARY=target/selfdev/jcode scripts/dev_cargo.sh test -p jcode --test e2e worker_stop:: -- --include-ignored --nocapture`. Result: 2 passed. Log: `scratch/stop-e2e2.log`.
- Observed: unrelated denial text `Refusing to stop session '...' because it was not spawned by this coordinator. Pass force=true ...`; owner stop `done`; repeat stop `done`; marker absent after the 5.009 s window; provider saw one request. `generation` is still null on the wire.
- Not covered, so 2.2 and 3.3 stay unchecked: allow/deny after member removal, owner reconnect, ambiguous aliases and explicit force override over the socket, plus persistence-error and delayed-settlement cases.
- This supersedes the 33341d76 binary identity for 7.1. Final 7.1/7.2 still needs the coordinator's frozen baseline.

## 2.2 partial: stop authorization survives member removal (worker, 2026-11-19)

- Defect found: `handle_comm_stop` computed `stop_allowed` only from the live `swarm_members` entry, so once the member projection was removed even the owner was refused, although the retained control record already carried the owner chain.
- Fix in `server/comm_session.rs`: when no member entry exists, authorize with `store.inspect(target)` plus `store.authorized(requester, target, force)`. No record means no authority, so unknown ids stay denied. The live-member path is unchanged.
- New test `comm_session_tests::stop_authorizes_removed_member_from_retained_owner_record` runs `handle_comm_stop` with an empty member map: owner stop gives `Done` and state `Stopped`, an unrelated stranger gets `Refusing to stop` and the record stays `Runnable`, and an explicit `force` gives `Done`.
- Commands: `-p jcode-app-core --lib -- comm_session_tests` gave 41 passed. `-- session_control:: stop` gave 57 passed. Logs: `scratch/commsess.log`, `scratch/stopreg.log`.
- Still open for 2.2: owner reconnect, ambiguous aliases, real-socket coverage after removal, runtime command docs. 2.2 stays unchecked.

## 2.1 / 2.2 cross-crate step, approved by coordinator (worker, 2026-11-19)

Paths touched: `crates/jcode-swarm-core/src/control.rs` (new), `crates/jcode-swarm-core/src/lib.rs`, `crates/jcode-swarm-core/Cargo.toml` (dev-dep `serde_json`), `crates/jcode-protocol/src/lib.rs`, `crates/jcode-protocol/src/comm_format.rs`, `crates/jcode-protocol/src/protocol_tests/comm_responses.rs`, `crates/jcode-app-core/src/server/session_control.rs`, `server/comm_sync.rs`, `server/comm_session.rs`, `server/comm_session_tests.rs`, `src/protocol_tests/comm_responses.rs`, `tool/communicate_tests/input_format.rs`.

- `ControlRecord`/`ControlState` moved to `jcode_swarm_core::control` with `validate()`, `as_str()`, `admission_revoked()`. `session_control.rs` re-exports them and uses `validate` on load. On-disk shape is unchanged (schema_version 1).
- Compatibility fields: `AgentStatusSnapshot.control: Option<ControlStatusSnapshot>` with `#[serde(default, skip_serializing_if = "Option::is_none")]`. Absent means no record or an older daemon, never runnable. Unreadable control is reported as `unresolved` with category `control_unreadable`. Retention boundary: the record file persists beyond member removal and GC. Nothing deletes it in this slice.
- Tests: `-p jcode-swarm-core` 21 passed (was 16; adds legacy defaults, unknown-field tolerance, unknown-version refusal, corrupt/inconsistent rejection, stale-snapshot independence). `-p jcode-protocol` 124 passed (was 122; adds legacy decode without `control`, roundtrip and formatter). `-p jcode-app-core --lib -- session_control::` 13 passed. `-- comm_session_tests` 42 passed, including `status_resolves_removed_member_by_exact_id_for_owner_only` (owner gets control state, stranger refused, unknown id still "not in the same swarm") and the earlier removed-member stop test.
- Defect fixed: `handle_comm_status` required current swarm membership, so an owner could not inspect a worker that left. It now resolves exact IDs from the retained record with owner-ancestry authorization.
- Logs: `scratch/core.log`, `scratch/proto.log`, `scratch/commsess.log`, `scratch/sc.log`.

## 2.3 admission audit (2026-11-19)
- Entry points: `agent/turn_execution_tests.rs::every_agent_entry_point_refuses_stopped_session_without_new_generation` passed (run_once_capture, run_once, run_once_streaming_mpsc, execute_tool refuse; provider never called; generation/revision unchanged).
- Wake path: `idle_live_agent` checks `runnable`; lightweight registration checks it (inspection).
- Assignment: `server/comm_control.rs` new `ensure_assignable` (uses `session_control::store().runnable`). Called in explicit-target resolution, filters auto-assign candidates, re-checked before the spawned assigned run starts, and before `task_agent_session` in assign and replace/salvage paths.
- Test: `comm_control_tests/assign_task.rs::assign_task_refuses_stopped_worker_and_leaves_plan_unmutated` (explicit and auto target both refused, plan item unassigned/queued, member status unchanged).
- Command: `scripts/dev_cargo.sh test -p jcode-app-core --lib -- comm_control` -> 55 passed, 0 failed.
- Not yet covered by a dedicated test: scheduled delivery and recovery admission (checked by inspection only).

## 3.3 / 6.1 additions (2026-11-19)
- `server/comm_session.rs`: `STOP_DRAIN_DEADLINE` const (25 s prod, 300 ms under `cfg(test)`); behavior unchanged in production.
- Test `comm_session_tests::stop_with_unsettled_work_reports_unresolved_then_retry_settles`: with a reserved WorkGuard, `handle_comm_stop` returns `Error` containing `unresolved session=… request=coord:7 generation=N … settlement not confirmed` (never `Done`), repeat stop while busy is identical, control state is `Unresolved` and `admit` fails; after the guard drops a retry returns `Done` and state is `Stopped` with higher revision. `-- comm_session` -> 43 passed.
- Wire shape: nonterminal results use the existing `ServerEvent::Error` text (older clients cannot read it as success). No dedicated `stop_requested`/`draining` variant was added; see tasks.md 3.1/3.3 notes.
- 6.1: `server/swarm_persistence.rs::from_persisted_member` now consults `session_control::store().inspect` first. Settled stop -> `stopped` (no spec orphaning/retry); StopRequested/Unresolved or corrupt/unreadable record -> new non-terminal `stop_unresolved` status (`STOP_UNRESOLVED_STATUS`); Runnable/no record -> legacy recovery. Test `swarm_persistence_tests::restore_reconciles_durable_control_before_membership_or_runnable_work` (corrupt bytes preserved). `-- swarm_persistence` -> 26 passed.
- Not done: real daemon restart test; uncertain child liveness beyond the existing `shell_children` category test.

## 7.1 final-binary acceptance (2026-11-19)
- Binary: `target/selfdev/jcode` `jcode v1.0.223-dev (52626ede5, dirty)`, sha256 `b5c8b4e2b2156ec1d618fbd7719e9eeba90d084b2d37a3ba4f75f891ce411f13`, built by `selfdev build` (no reload; shared daemon untouched).
- Command: `JCODE_WORKER_STOP_BINARY=target/selfdev/jcode scripts/dev_cargo.sh test -p jcode --test e2e worker_stop:: -- --include-ignored --nocapture`. Result: 2 passed (exit 0). Log: `$JCODE_SCRATCH_DIR/stop-e2e3.log`. Private user/net namespace, loopback-only, owned temp JCODE_HOME, private socket.
- Observed on the public socket: busy `comm_status` carries `control{generation:1,revision:1,state:runnable}`; unrelated `comm_stop` denied; owner stop `done`; repeat stop `done`; marker file absent after 5.001 s.
- Limits (not claimed): model-busy only, loopback Responses fixture; native-tool-busy, resume races, stale wait replay, restart-mid-stop and unresolved external work were NOT exercised on the real binary (covered only by unit/integration tests above). The harness top-level `generation` field is still null (harness does not read `control.generation`).

## Final gates (2026-11-19, worker)
- `-p jcode-swarm-core` exit 0; `-p jcode-protocol` exit 0; `-p jcode --test e2e` exit 0 (62 passed, 9 ignored); `openspec validate make-worker-stop-durable --strict --no-interactive` valid; `git diff --check` exit 0.
- `-p jcode-app-core --lib`: 2211 passed, 4 failed in the full parallel run (`client_actions::tests::notify_session_runs_scheduled_task_immediately_for_idle_live_session`, `tests::wake_turn_tracks_member_status_and_emits_terminal_done`, `mobile_local::phone_photo_starts_same_session_turn_with_image_content`, `mobile_local::idle_session_that_is_briefly_locked_still_gets_a_turn`), all 5 s `Elapsed` timeouts. Each passes alone and the group passes with `--test-threads=1` (12 passed). Cause under parallel load not proven; not claimed fixed.
- Workspace clippy: first run exit 101 (2 regressions mine: `handle_comm_stop` lost `#[allow(clippy::too_many_arguments)]` when I inserted the const; `cloned_ref_to_slice_refs` x2 in `await_reload_final.rs`). Fixed. Final `JCODE_REMOTE_CARGO=0 scripts/dev_cargo.sh clippy --workspace --all-targets -- -D warnings` -> exit 0. Log `$JCODE_SCRATCH_DIR/clippy2.log`.

## 4.1 per-tool revoked mutation (2026-11-19)
- Test `server::session_control::tests::every_native_mutation_tool_is_denied_after_revocation_with_bytes_unchanged` drives the real `Registry::execute` for `write`, `edit`, `patch` and `apply_patch` (update + add + delete). Positive control: the same four inputs succeed for a runnable session. After `request_stop`, all four are denied, target bytes unchanged, deleted file kept, added file not created.
- Command: `scripts/dev_cargo.sh test -p jcode-app-core --lib -- every_native_mutation_tool` -> 1 passed. `-- every_native_mutation_tool session_control::` earlier run (before the control was added) -> 14 passed.
- Still unverified for 4.1: no old-generation cleanup of new claims; preserved live conflict behavior across a generation change.

## 4.2 automatic reassignment guard (2026-11-19)
- `ControlStore::predecessor_unsettled(id)` (true for StopRequested/Draining/Unresolved, fail-closed barrier, or unreadable record). Used by `swarm::lifecycle::salvage_assignments_of_dead_member` (returns empty, task stays assigned) and `comm_control::next_runnable_task_id_reclaiming_stranded` (assignee not treated as dead). Stopped or no record keeps the old behavior.
- Test `swarm::tests::salvage_and_stranded_reclaim_wait_for_unsettled_predecessor`: with a reserved WorkGuard and stop requested, salvage returns empty, auto reclaim returns None, item stays `running`/assigned. After the guard drops and `settle`, salvage requeues `task-1`.
- Test hygiene: my first version used id `worker`; because it mutates `JCODE_RUNTIME_DIR` and other tests do not take the env lock, three `owner_notifications` tests saw the unsettled record and failed. Fixed with a unique id. Repeated 3x: `-- salvage unsettled_predecessor owner_notifications` -> 27 passed each. This shows existing tests share the global runtime dir when run in parallel with env-mutating tests (possible contributor to the unproven 5 s timeouts; not shown).
- Not covered: a real tool-busy stop on the real binary, and replace/assign-time reassignment while unsettled (assign refuses non-runnable targets via `ensure_assignable`, but a runnable new target taking an unsettled predecessor's task via explicit `assign_task` is not tested).

## Second full run after 4.1/4.2 work (2026-11-19)
- `-p jcode-app-core --lib` full parallel: 2213 passed, 4 failed (exit 101). Failing names are in `$JCODE_SCRATCH_DIR/fails2.txt`; all are 5 s `Elapsed` timeouts in the same wake/mobile_local/scheduled family as the first run. The set is not identical between runs (a different `mobile_local` test failed), which points to load/timing, but the cause is unproven and I did not fix it.
- Workspace clippy: run 3 exit 101 on two errors in my new tests (`useless_format` in `session_control.rs`, `await_holding_lock` in `swarm/tests.rs`). Fixed (repo convention `#[allow(clippy::await_holding_lock)]`). Final `JCODE_REMOTE_CARGO=0 scripts/dev_cargo.sh clippy --workspace --all-targets -- -D warnings` -> exit 0 (`$JCODE_SCRATCH_DIR/clippy4.log`).
- The 4th failure in this run differs in kind: `tool::user_response::tests::user_response_desktop_registration_is_reversible_and_isolated` asserted `"# Hello"` but got `"# Hello\n\n[post_tool_feedback]\ncall=call tool=user_response\npaths="`, i.e. a `post_tool_feedback` hook output leaked in, which suggests process-global hook configuration set by a concurrent test. It passes alone (`-- user_response` -> 6 passed). I did not touch hooks or `user_response`; cause unproven, not fixed.

## 2.3 lightweight registration + persistence failure (2026-11-19)
- `client_lightweight_control::tests::corrupt_control_record_fails_closed_and_registers_no_member`: a persisted session with a corrupt control record registers no member and no swarm, and the corrupt bytes are preserved. Positive control: the existing `persisted_headless_session_is_registered_with_its_working_dir` uses identical setup (`isolated_home`, `empty_state`, saved session) and does register. Existing `stopped_stub_cannot_recreate_ownerless_membership` covers the stopped case. `-- client_lightweight_control` -> 4 passed.
- Persistence failure at the store: `session_control::tests::failed_stop_persistence_retains_local_barrier` and `corrupt_unsupported_and_stale_records_fail_closed_without_overwrite` (pre-existing).

## 2.2 alias/reconnect + docs (2026-11-19)
- Test `comm_session_tests::aliases_stay_swarm_local_while_exact_id_survives_owner_reconnect`: owner reconnected in `swarm-new`, worker in `swarm-old`. `resolve_stop_target_session("swarm-new","otter")` errors, `handle_comm_stop` by alias returns `Error` and leaves the worker `Runnable`, the exact ID returns `Done` and the worker is `Stopped`. Existing tests cover ambiguous alias (`stop_target_rejects_ambiguous_friendly_name`), unrelated denial and explicit override (`stop_authorizes_removed_member_from_retained_owner_record`). `-- comm_session_tests` -> 44 passed.
- Docs: `docs/SWARM_ARCHITECTURE.md` "Stop target resolution and authorization".
- Real-socket: e2e (7.1) covers unrelated denial, owner stop and repeat stop only. Removed-member, alias and override are handler-level, not real-socket. 2.2's real-socket requirement is therefore only partly met.

## 7.1 native-tool-busy on real binary (2026-11-19)
- New ignored e2e `worker_stop::public_stop_during_native_tool_never_reports_stopped_while_tool_runs` (scenario arg `bash` in the same python harness): fixture model emits one `bash` call (`echo running > started; sleep 6; echo finished > finished`). Harness waits for `started`, then sends `comm_stop` over the private socket.
- Binary: `target/selfdev/jcode` `jcode v1.0.223-dev (52626ede5, dirty)`, sha256 `32a573670e3a1665f2e2efee565876aec8b93de0296802445b75a0bf30c0635c` (built after the 4.2 guard). Command: `JCODE_WORKER_STOP_BINARY=target/selfdev/jcode scripts/dev_cargo.sh test -p jcode --test e2e worker_stop:: -- --include-ignored --nocapture` -> 3 passed (exit 0). Log `$JCODE_SCRATCH_DIR/stop-e2e4.log`. (An earlier run of the new test on the previous binary `b5c8b4e2…` also passed: `$JCODE_SCRATCH_DIR/stop-bash.log`.)
- Observed: `comm_stop` returned `error` `unresolved session=… request=…:30 generation=1 revision=5 categories={"shell_children"}; settlement not confirmed` (never `done`). Control after stop: `state=unresolved, revoked_through=1, unresolved_categories=[shell_children]`. The bash command was NOT killed: `tool_finished_after_stop=true` (finished ~6.25 s later). Only 1 provider request, so no new model turn started after the stop.
- Consequence to decide (not changed by me): `bash` marks `shell_children` before dispatch, and `settle` refuses while any unresolved category is set. So a worker that ever ran `bash` can never reach `stopped`, and its claims are never released, even after the shell command finished. That is safe (no false success) but means stop cannot complete for such workers. Clearing the category when the owned child is proven gone is task 3.2's "owned child accounting", which is not implemented.
- Still NOT run on the real binary: resume races, stale wait replay, restart-mid-stop, claims transfer to a new owner.

## Final worker gates (2026-11-19 07:06-07:10 UTC)
- `scripts/dev_cargo.sh test -p jcode-swarm-core` exit 0; `-p jcode-protocol` exit 0; `-p jcode --test e2e` exit 0 (62 passed, 10 ignored).
- `JCODE_REMOTE_CARGO=0 scripts/dev_cargo.sh clippy --workspace --all-targets -- -D warnings` exit 0 (`$JCODE_SCRATCH_DIR/f-clippy.log`).
- `-p jcode-app-core --lib` exit 101: 2214 passed, 5 failed in the full parallel run (`notify_session_runs_scheduled_task_immediately_for_idle_live_session`, `jade_relay::pending_messages_drain_as_one_turn_after_busy_turn_ends`, `wake_turn_holds_reservation_until_terminal_status_is_published`, `mobile_local::idle_session_is_reported_live_and_a_phone_message_starts_a_turn`, `mobile_local::idle_session_that_is_briefly_locked_still_gets_a_turn`). The same 5 filters pass with `--test-threads=1` (11 passed, exit 0). The failing set varies between full runs (4 earlier, 5 now). Cause is NOT proven (5 s timeouts under parallel load; suspected shared runtime-dir/env contention). Not fixed by me; not proven pre-existing either, since I did not run the suite on a clean tree.
- `openspec validate make-worker-stop-durable --strict` valid; `git diff --check` exit 0.

## 4.2 explicit assign guard (2026-11-19 07:11 UTC)
- Code: `crates/jcode-app-core/src/server/comm_control.rs` `handle_comm_assign_task_with_mode` now computes `predecessor_reason` under the plan lock: an explicit `task_id` whose current assignee is a different session with `ControlStore::predecessor_unsettled` is refused (`Task 't1' belongs to '…', whose stop has not settled`), before any plan mutation.
- Test: `explicit_assign_refuses_task_of_unsettled_predecessor_and_leaves_plan_unmutated` (stop requested with a live work reservation; assign to a different runnable worker is an error; assignee, status and plan version unchanged). `scripts/dev_cargo.sh test -p jcode-app-core --lib -- comm_control` exit 0, 56 passed.

## 2.3 scheduled delivery (2026-11-19 07:13 UTC)
- Test `server::client_actions::tests::notify_session_never_runs_scheduled_task_on_stopped_live_session` (isolated `JCODE_RUNTIME_DIR`, registered then stopped session, live client attached, one queued provider response). After `handle_notify_session` + 300 ms: provider queue still length 1, no assistant message, no `TextDelta` on the member stream. `scripts/dev_cargo.sh test -p jcode-app-core --lib -- notify_session` exit 0, 3 passed (includes the positive control for an idle runnable session). Path under test: `live_turn::idle_live_agent` -> `session_control::runnable`.

## Gates after 4.2/2.3 additions (2026-11-19 07:13 UTC)
- `JCODE_REMOTE_CARGO=0 scripts/dev_cargo.sh clippy --workspace --all-targets -- -D warnings` exit 0 (`scratch/g-clippy.log`).
- Scoped `-p jcode-app-core --lib -- comm_control comm_session swarm_persistence client_lightweight_control notify_session session_control swarm::tests`: parallel run exit 101, 181 passed, 1 failed: `notify_session_runs_scheduled_task_immediately_for_idle_live_session` (`scheduled task should start streaming promptly: Elapsed`, 2 s timeout). That test was already in the flaky parallel-timeout family before my new test existed. Same scoped command with `--test-threads=1`: exit 0, 182 passed. Cause of the parallel timeouts remains unproven.
- `openspec validate --strict` valid, `git diff --check` 0.

## 6.1/7.1 restart after unresolved stop on real binary (2026-11-19 07:15 UTC)
- New ignored e2e `worker_stop::restart_after_unresolved_stop_keeps_worker_revoked` (scenario `restart` of the private-namespace harness, same isolation: unshare user+net, loopback fixture, owned JCODE_HOME/runtime, `JCODE_TEST_SESSION`-style no shared state). Steps: headless worker runs a 6 s `bash` tool; `comm_stop` -> `error unresolved … shell_children`; `process.kill()` (SIGKILL) the daemon; remove stale socket; start a second daemon on the same state; `subscribe` with `target_session_id` = original requester; `comm_status` on the worker.
- Binary `target/selfdev/jcode` sha256 `32a573670e3a1665f2e2efee565876aec8b93de0296802445b75a0bf30c0635c` (`v1.0.223-dev (52626ede5, dirty)`). Command: `JCODE_WORKER_STOP_BINARY=target/selfdev/jcode scripts/dev_cargo.sh test -p jcode --test e2e worker_stop:: -- --include-ignored` -> 4 passed, exit 0 (`scratch/stop-e2e5.log`).
- Observed after restart: `control {state: unresolved, generation 1, revision 5, revoked_through 1, stop_request …:30, unresolved_categories [shell_children]}`, member `status: stop_unresolved`, detail `stop not settled before reload; work state unresolved`, `new_requests_after_restart = 0`, requester session resumed as the original id.
- Note: killing the daemon with SIGKILL also kills its in-process bash child group only if the OS does; the harness does not check the orphaned `sleep`. Not claimed.

## Gates after restart e2e (2026-11-19 07:17 UTC)
- `JCODE_REMOTE_CARGO=0 scripts/dev_cargo.sh clippy --workspace --all-targets -- -D warnings` exit 0 (`scratch/h-clippy.log`); `-p jcode --test e2e` exit 0 (62 passed, 11 ignored). `openspec validate --strict` valid; `git diff --check` 0.

## 7.2 coordinated selfdev test (2026-11-19 07:17 UTC)
- `selfdev test` (request `selfdev-build-81f20b87a60c44cb87673aaddb61b738`, task `638442q593`): `scripts/dev_cargo.sh test -p jcode-app-core --lib -- --test-threads=1 comm_control comm_session swarm_persistence client_lightweight_control notify_session session_control swarm::tests await_members` -> exit 0, 185 passed, 0 failed. Serial by design because the parallel run has the unproven timeout flakes. Full `-p jcode-app-core --lib` was not run through `selfdev test`.

## 2.2 alias over the public socket (2026-11-19 07:19 UTC)
- New ignored e2e `worker_stop::alias_stop_is_swarm_local_over_the_public_socket` (scenario `alias`, same private-namespace harness, binary sha256 `32a573670e3a1665f2e2efee565876aec8b93de0296802445b75a0bf30c0635c`). Observed: alias `elephant`; unrelated session `comm_stop target=elephant` -> `error Unknown swarm session 'elephant'…`, worker control stays `runnable` (revoked_through 0); owner `comm_stop target=elephant` -> `error unresolved … shell_children` (the worker ran a 6 s bash), control `unresolved`, revoked_through 1; 1 provider request, no faults.
- Gates: `worker_stop::` 5 passed (exit 0); workspace clippy `-D warnings` exit 0 (`scratch/i-clippy.log`); `-p jcode --test e2e` exit 0 (62 passed, 12 ignored).

## 3.3 persistence failure during stop (2026-11-19 07:20 UTC)
- `comm_session_tests::stop_persistence_failure_reports_error_and_keeps_worker_revoked`: worker control file replaced with a directory, `handle_comm_stop` -> single `ServerEvent::Error`; `reserve(old token)` and `admit` both fail afterwards. `scripts/dev_cargo.sh test -p jcode-app-core --lib -- comm_session_tests` exit 0, 45 passed. Touched: `session_control.rs` `fn path` -> `pub(super)`; `comm_session_tests.rs`.

## Round 3 (2026-11-19 07:30-07:58 UTC): decisions 6.2, 3.2, 5.1, 3.1 plus 5.2/4.1 stale generation
Binary: `target/selfdev/jcode` sha256 `cc01740ee832a9e9abe085013658002b5ab794aae2bcfe2c794006ddf07d26bc` (built directly with `scripts/dev_cargo.sh build --profile selfdev`; the selfdev build request was superseded by source drift). Daemon never reloaded; all runs on private sockets in a user+net namespace with owned JCODE_HOME.

### 6.2 compat floor
- `jcode-swarm-core/src/control.rs`: `CONTROL_SCHEMA_VERSION=2`, `CONTROL_MIN_READER_VERSION=2`, `min_reader_version` field (serde default 1), accepted schema 1..=2. Newer schema or a floor above the reader fails closed, bytes preserved. `session_control.rs`: `save_next` stamps schema+floor on every write, reads never rewrite; `ControlStore::incompatible_records()`; `swarm_persistence::load_runtime_state` logs each incompatible record.
- Tests: swarm-core unit tests (22 passed) and `newer_reader_floor_fails_closed_preserves_bytes_and_legacy_records_upgrade_on_write`.
- Rollback: needs a build at or above floor 2, or operator removal of `session-control/`; bytes are never migrated or deleted automatically. No launcher/`src/cli` hook. Documented in `docs/SWARM_ARCHITECTURE.md` "Compatibility floor and rollback". Limit: a pre-change binary never reads `session-control/`, so nothing here stops it serving state.

### 3.2 shell child accounting
- `jcode-swarm-core`: `OwnedProcess{pid,start_time}`, `ControlRecord.owned_children`. `session_control.rs`: `GenerationToken::own_shell_child()`/`OwnedChild` (records pid on spawn, reconciles on drop, 250 ms poller if still alive), `process_gone`/`group_has_live_member`, `reconcile_children`, `drain` reconciles children and clears only `shell_children` (any other category stays `unresolved`).
- `tool/bash.rs` (outside owned paths, approved): `own_shell_child()` at foreground, detached and background spawn sites. `tool/mod.rs`: `external_effect_category("bash")` returns `None` (bash marks itself) with its unit test updated.
- Tests: `shell_child_clears_only_after_proven_exit_and_then_stop_settles`, `surviving_grandchild_or_unreaped_child_keeps_stop_unresolved`, unknown-effect test. Real binary: bash scenario now reports `done` only when the tool is proven finished (`finished == finished_at_stop`), state `stopped`.
- Limits: proof is Linux `/proc` only; non-Linux Unix `kill(-pgid,0)` fallback and non-Unix never-proves are unverified. Detached/promoted commands and backgrounded grandchildren stay `unresolved`. Nothing unrelated is killed.

### 5.1 production resume
- `ControlStore::admit_new_work(requester,target,force) -> NewWorkAdmission{AlreadyRunnable,Resumed(gen)}`; settled stop becomes `Runnable` at G+1; unauthorized or unsettled is refused without writing; no record = runnable. Old `resume` stays `cfg(test)`. `comm_control.rs::admit_explicit_work` wired into explicit-target assign, the assign path and task-control (wake/resume/retry/reassign).
- Test `explicit_new_work_allocates_one_fresh_generation_and_refuses_unsettled_or_unauthorized` (8 concurrent callers, one winner, stale token rejected, reload does not resume a stopped record). Real binary: `worker_stop::concurrent_wakes_after_settled_stop_admit_exactly_one_generation` (two concurrent `comm_task_control` wakes: one resume to G+1, its turn admits G+2; replies `queued` and `running`).

### 5.2 / 4.1 stale generation
- `ControlStore::project` rejects ("Stale generation projection rejected") inside a scope whose token generation differs from the admitted one. Test `stale_generation_report_cannot_project_onto_the_resumed_generation` (asserts the claim-release closure never runs). Completion reports/queued requests carrying generation on the wire are NOT done, so 5.2 stays open.

### 3.1 events
- `jcode-protocol/src/wire/events.rs`: additive `ServerEvent::StopRequested` and `Draining` (`request_id, session_id, generation, revision, terminal:false`, serde defaults, no `id`, so legacy readers skip them). `comm_session.rs::handle_comm_stop` sends `StopRequested`, plus `Draining` when not already stopped; `tool/communicate/transport.rs` skips both.
- Tests: protocol `stop_requested_and_draining_are_nonterminal_additive_events`; `comm_session_tests::stop_emits_nonterminal_frames_before_the_terminal_reply`; `session_control::multi_attachment_stop_cancels_every_active_turn_and_settles_only_when_all_end`. Real binary frames: `["stop_requested","draining","done"]` and `["stop_requested","draining","error"]`.

### Touched paths this round (beyond earlier lists)
`crates/jcode-swarm-core/src/control.rs`, `crates/jcode-protocol/src/wire/events.rs` (+ `lib.rs`, `protocol_tests/comm_responses.rs`), `crates/jcode-app-core/src/server/{session_control,comm_control,comm_session,comm_session_tests,swarm_persistence}.rs`, `crates/jcode-app-core/src/tool/{bash,mod}.rs`, `crates/jcode-app-core/src/tool/communicate/transport.rs`, `tests/e2e/worker_stop.rs`, `docs/SWARM_ARCHITECTURE.md`. `cargo fmt` churn in `tool/todo.rs`, `agent_tests.rs`, `tool/bash_tests.rs`, `agent/turn_streaming_mpsc.rs` was reverted; `git diff --stat` for bash.rs/transport.rs/mod.rs is 30 insertions and 6 deletions, all mine. Other dirty files in the tree (memory-worker fixtures, ambient, session_launch, mobile_push_tests) are not mine.

### Final gates, round 3 (07:51-07:57 UTC)
- `JCODE_REMOTE_CARGO=0 scripts/dev_cargo.sh clippy --workspace --all-targets -- -D warnings`: exit 0, 0 errors (`scratch/clippy.log`).
- `-p jcode-swarm-core -p jcode-protocol`: exit 0 (22 + 125 passed).
- `-p jcode --test e2e`: exit 0, 62 passed, 13 ignored.
- `JCODE_WORKER_STOP_BINARY=target/selfdev/jcode ... -p jcode --test e2e worker_stop:: -- --include-ignored --test-threads=1`: exit 0, 6 passed (alias, concurrent wakes, native-tool stop, post-stop write, shared-network refusal, restart) in 65.9 s.
- Scoped `-p jcode-app-core --lib -- --test-threads=1 comm_control comm_session swarm_persistence client_lightweight_control notify_session session_control swarm::tests await_members external_effect bash protocol_tests`: exit 0, 258 passed.
- `-p jcode-harness-api -p jcode-harness-api-server -p jcode-sdk`: exit 0 (all suites ok). No dedicated test for `stop_requested`/`draining`; the harness client skips `ApiEvent::Unknown` per its contract.
- `openspec validate make-worker-stop-durable --strict --no-interactive`: valid. `git diff --check`: exit 0.
- Full parallel `-p jcode-app-core --lib`: exit 101, 2223 passed, 6 failed: `wake_turn_tracks_member_status_and_emits_terminal_done`, three `mobile_local` tests (5 s/2 s `Elapsed`), `websearch::kagi::...redirect_safety`, `websearch::tests::custom_post_policy_transport_and_http_errors`. Re-run serially on `wake_turn_tracks_member_status mobile_local websearch`: 40 passed, 1 failed (`kagi_local_http_success_errors_and_redirect_safety`, loopback `listener.accept()` assertion, `websearch_kagi_tests.rs:163`, files I did not touch). The wake/mobile family pass serially; cause of the parallel timeouts is unproven and unfixed, and I did not run the suite on a clean tree, so I cannot say the websearch failures are pre-existing.

### Round 3 residual gaps
- 7.1: stale wait replay not exercised on the real binary; claims hand-off to a new owner after settlement not run on the real binary; restart mid-drain and graceful reload not run; ambiguous alias, `force` override, removed-member stop/status, owner reconnect not driven over the socket.
- 2.3: recovery admission (`worker_revive`) has no dedicated test; no concurrent stop/wake test beyond the resume race.
- 5.2: generation not on completion reports/queued requests.
- 6.1: unchanged (restart after settled stop and graceful reload not on the real binary).
- 7.2: full `-p jcode-app-core --lib` not green (above), not run through `selfdev test` this round.
- 7.3: docs updated (`docs/SWARM_ARCHITECTURE.md`) but reciprocal P04 links and overlap revalidation with `add-swarm-handoff-contracts` not done.

## Round 4 (2026-11-19 09:12-09:57 UTC): 2.2, 2.3, 4.2, 5.2, 6.1, 7.1, 7.3

Binary: `target/selfdev/jcode`, `jcode v1.0.228-dev (44f2a28e5, dirty)`, sha256 `bfdd804161528e7bbb17f9fccc8154349a8dffaddd9c1b6ce1cb8fca23044c1e` (built 09:16 UTC via `scripts/dev_cargo.sh build --profile selfdev -p jcode --bin jcode`, exit 0). It contains all runtime code of this change. Source edits after the build were test-only (`tests/e2e/worker_stop.rs`) plus docs. Every e2e below ran with `JCODE_WORKER_STOP_BINARY=target/selfdev/jcode`, private user+net namespace, isolated `JCODE_HOME`, loopback scripted provider, `--include-ignored`.

### Real-binary scenarios (all in `tests/e2e/worker_stop.rs`; full set `worker_stop::` = 14 passed in 197 s, `scratch/r-f-ws.log`)
- 2.2 `owner_control_survives_churn_ambiguity_denial_and_force_over_the_public_socket`: ambiguous alias refused, unrelated session denied, `force` override stops, owner reconnect (new connection, same session) controls the worker. Earlier `alias_stop_is_swarm_local_over_the_public_socket` covers the swarm-local alias.
- 2.3 `concurrent_stop_and_wake...` (stopwake, passed twice): never a runnable generation at or below the revoked one, no model turn after the barrier. Resume: DMs to a stopped worker start no provider turn; two concurrent wakes admit exactly one winner.
- 4.2 `reassign_waits_for_stop_settlement_then_hands_the_task_off`: reassign while the stop is unsettled (40 s shell child alive) refused "not settled", zero provider turns; after settlement the reassign succeeds and the successor owns the task.
- 5.2 / 7.1 stale wait replay `wait_bound_to_stopped_generation_never_completes_from_resumed_work` (`scratch/r-e2e9.log`): a non-background `comm_await_members` bound before the stop, then stop, then explicit wake (worker generation 3, `admitted_generation` 3): reply `completed: false`, member `done: false`.
- 6.1 restart: `restart_after_unresolved_stop_...`, `restart_while_draining_keeps_worker_revoked`, `restart_after_settled_stop...` (DMs start no turn, wake resumes at a new generation).
- 6.1 graceful `reload` (`{"type":"reload","force":true}` on the private socket; `JCODE_TEST_SESSION` is not set in the harness so the daemon really execs, same pid):
  - `graceful_reload_after_settled_stop_stays_stopped_until_explicit_wake`: after reload state `stopped`, `revoked_through` 1, detail "stop settled before reload; not restored as runnable"; DMs start no turn; explicit wake gives runnable generation 3 and a new provider turn.
  - `graceful_reload_while_draining_keeps_worker_revoked`: stop unsettled (shell child), reload: state `draining`, status `stop_unresolved`, unresolved category `shell_children`, `new_requests` 0, DMs accepted but no turn.
  - Not exercised: `UpgradeRuntime` packaged upgrade (needs a strictly newer signed candidate binary; not available in isolation). It refuses while any turn or background tool is active and never interrupts work, so it cannot restore a revoked worker; no test drives it.

### Unit and crate gates (no host Cargo gate: another job's `jcode-base config_composition` held `jcode-cargo-build.lock` for 25+ min, so these ran with `JCODE_CARGO_GATE=off`, NOT through `selfdev test`)
- `-p jcode-swarm-core -p jcode-protocol`: exit 0 (125 + 22 passed, 1 ignored).
- Clippy `-p jcode-swarm-core -p jcode-protocol -p jcode-app-core -p jcode --all-targets --no-deps -- -D warnings`: exit 0.
- `-p jcode --test e2e`: exit 0, 62 passed, 21 ignored.
- `-p jcode-harness-api -p jcode-harness-api-server -p jcode-sdk`: exit 0.
- `-p jcode-app-core --lib -- --test-threads=1` (full, serial): exit 0, 2239 passed, 0 failed, 25 ignored.
- `openspec validate make-worker-stop-durable --strict --no-interactive`: valid. Dependency validator (`validate_dependencies.py --root /home/nyaptor/.jcode make-worker-stop-durable --json`): valid, closure size 1. `git diff --check`: exit 0 (after removing one blank line at EOF in `worker_stop.rs`).
- Unit tests for this round: `session_control::report_stamps_only_the_producing_generation_and_binds_waits`, `await_report_done::{wait_bound_to_old_generation_never_completes_from_resumed_work, pending_wait_generation_bindings_survive_restart_and_legacy_states_stay_unbound}`, `queue_tests::settled_stop_discards_queued_soft_interrupts_live_and_persisted`, `assign_double::task_control_reassign_waits_for_unsettled_predecessor_then_hands_off`.

### 7.2 flaky failures: pre-existing, compared against pristine HEAD (`scratch/applyall/base`, `CARGO_TARGET_DIR=scratch/applyall/target-base`)
- Base, filtered serial (`wake_turn_tracks_member_status...`, `mobile_local`, kagi, websearch custom_post): 11 passed (`scratch/r-base-serial.log`). My tree, same filter: 11 passed (`scratch/r-f-filter.log`).
- Base, full parallel `-p jcode-app-core --lib`: exit 101, 4 failed of 2198: `wake_turn_holds_reservation_until_terminal_status_is_published`, `mobile_local::phone_photo_starts_same_session_turn_with_image_content`, `mobile_local::idle_session_that_is_briefly_locked_still_gets_a_turn`, `tool::user_response::...desktop_registration_is_reversible_and_isolated` (`scratch/r-base-full.log`).
- My tree, full parallel: exit 101, 5 failed of 2241 (`scratch/r-f-libpar.log`): `notify_session_runs_scheduled_task_immediately_for_idle_live_session`, `wake_turn_tracks_member_status...`, three `mobile_local`. Earlier parallel run failed a different 6 (incl. kagi/websearch).
- Conclusion supported by evidence: the pristine base also fails 4 tests in the full parallel run, the failing set differs per run on both trees, and every one passes serially on both. That proves the failures are load-dependent timing flakes that pre-exist this change. It does not prove each individual test's exact cause; none was investigated or fixed. The kagi/websearch failures appeared only in an earlier parallel run of my tree and passed on every serial run.

### Workspace clippy: external blocker
`--workspace` clippy fails in untracked `crates/jcode-base/src/config/composition.rs` (unused `SourceStamp` enum and `read` fn) and modified `config_file.rs`/`task_policy.rs`, none owned by this change. Pristine base lacks `composition.rs`. Owned-crate clippy above passes.

### 5.2 contract (see `docs/SWARM_ARCHITECTURE.md` "Generation-bound reports and waits")
`ControlRecord.report_generation`, `AwaitedMemberStatus.generation`, `PersistedAwaitMembersState.bound_generations` (all serde-default, backward readable); `project_report` stamps the producing generation under the entry lock; waits bind at invocation, ignore reports from other generations, treat a member that resumed past its bound with no matching report as superseded (not done), persist bindings across restart; legacy waits stay unbound. Settled stop discards queued and persisted soft interrupts from before the stop (`discard_pre_stop_soft_interrupts` in `comm_session.rs`). Report revision/changed-file enrichment stays with `add-swarm-handoff-contracts`.

### 2.3 admission audit
`queue_soft_interrupt_for_session` is a passive enqueue; its call sites in `comm_control.rs` sit behind `admit_explicit_work`, `predecessor_unsettled` and the wake path. Recovery revival (`worker_revive::live_or_revived_agent`, callers `live_turn.rs:80`, `comm_control.rs:216`) restores an agent but starts no work; any turn it leads to passes `session_control::run`. Covered on the real binary by restart_settled and graceful reload (DMs to a revived stopped worker start no turn). No dedicated unit test for `live_or_revived_agent` alone.

### 7.3
`docs/plans/PRODUCT_ROADMAP.md`: both `make-worker-stop-durable` entries changed from "(proposed, not implemented)" to "(implemented in the working tree; private-binary acceptance in its evidence.md, independent review and archive pending; not P04-wide completion)". Reciprocal link present: `add-swarm-handoff-contracts` proposal.md (`depends_on` and consume-generation bullet) and tasks.md. Overlap boundary unchanged and consistent: this change owns generation identity and wait freshness, that change owns report revisions, changed files and acceptance results.

### Touched paths this round
`crates/jcode-swarm-core/src/control.rs`; `crates/jcode-protocol/src/{lib.rs,comm_format.rs,protocol_tests/comm_requests.rs,protocol_tests/comm_format_awaited.rs}`; `crates/jcode-app-core/src/server/{session_control,comm_await,comm_session,await_members_state,swarm/lifecycle}.rs` and tests `comm_control_tests/{await_*.rs,assign_double.rs}`, `queue_tests.rs`; `crates/jcode-app-core/src/protocol_tests/comm_requests.rs`; `crates/jcode-app-core/src/tool/communicate_tests.rs`, `tool/communicate_tests/input_format.rs`; `crates/jcode-app-core/src/tool/selfdev/reload.rs` (OUTSIDE owned set: one test-literal line `bound_generations: Default::default()`); `tests/e2e/worker_stop.rs`; `docs/SWARM_ARCHITECTURE.md`; `docs/plans/PRODUCT_ROADMAP.md`; this change directory.

### Still open
- 7.2 left unchecked: the gates were not run through coordinated `selfdev test` (host Cargo gate held by another job) and the full parallel app-core lib run is red from the load flakes above. Needs a coordinator decision (accept serial-green plus base comparison, or rerun under the gate).
- Removed-member stop/status is handler-tested only, not driven over the socket.
- 8.x untouched.

## Round 5 stop-durable-fixes lane (2026-11-19, worker)

Test cmd: `JCODE_REMOTE_CARGO=0 scripts/dev_cargo.sh test -p jcode-app-core --lib -- <filter> --test-threads=1`. The shared cargo gate was held 20+ min by an idle `jcode_base memory` test run from another lane (0% CPU), so the final run used `JCODE_CARGO_GATE=off`.

- M2: `RegistryHost.execution` captured via `current_token()` and re-scoped around code-mode sub-calls; manual `/subagent` runs inside `session_control::run`. Tests `codemode_tests::controlled_session_subcalls_keep_generation_across_spawn`, `client_actions_tests::manual_subagent_runs_under_controlled_session_generation`.
- M1: member removal releases claims unless `predecessor_unsettled`. Test `claims_tests::member_removal_releases_runnable_worker_but_keeps_unsettled_stop`.
- M4+m1: `CommReport.generation` is on the wire (optional, old clients omit it and never bind). `project_report` stamps the work's own generation (scope token, else wire generation, else the process's last run), never ambient `admitted_generation`. `admit_new_work` records `pre_admitted`; the next `admit` adopts it so resume+turn allocate one generation and `awaited_generation` names it. Stop clears it. Tests `resume_then_turn_admission_allocates_one_generation_that_waits_bind_to`, `report_generation_is_the_works_own_never_ambient_admitted_state`, `test_comm_report_roundtrip`.
- M3: `comm_stop` joins the 120s mutating deadline class; `format_stop_outcome` prints `Stopped agent` only for `Done`, else `Stop pending ... inspect with communicate action=status target_session=<id>`; server `unresolved` message carries the same command. Tests `stop_never_reports_stopped_for_a_nonterminal_outcome`, `mutating_swarm_requests_get_extended_deadline`, `comm_session_tests` unresolved assertions.
- m2: `settle_rereading` (3 bounded re-reads on stale revision only). Test `settle_rereads_after_a_concurrent_revision_bump_but_keeps_other_refusals`.
- m3: `request_stop` no longer sets `fail_closed` before authorization. Test `unauthorized_or_unknown_stop_installs_no_barrier`.
- m4/m5: drain's child reconcile and the dropped-`OwnedChild` poller run the /proc scan and record write in `spawn_blocking`; the poller gives up after 600s (record stays listed and unresolved, later drains reconcile). Not covered by a dedicated test.
- 6.2 UNCHECKED: no launcher/updater/reload hook compares a candidate binary against `CONTROL_MIN_READER_VERSION`. `reload_exec_target` only guards by mtime. Remaining gap: a gate that reads the candidate's version and refuses reload/exec below the floor, plus a decision on the minimum release version. Records themselves already fail closed for newer schemas (`corrupt_unsupported_and_stale_records_fail_closed_without_overwrite`).
- 7.1 UNCHECKED: resume races, stale wait replay and claims transfer were never run on the real binary.
- 5.2 kept checked for the report/wait generation binding (now on wire); stale-notification wake suppression is covered only by existing `comm_await` tests.

## Round 6 stop-durable-final (2026-11-19 12:27-12:55 UTC, worker)

### 1.1 preflight (against committed code)
- Repo `/home/nyaptor/.jcode`, origin `https://github.com/leonardoacosta/jcode.git`, branch `main` (user retains main over remote HEAD `dev`, see 2026-11-10 preflight), HEAD `0a442e8e3`; code commit `5a53124cb` is an ancestor of HEAD. Git identity `leonardoacosta <leo@leonardoacosta.dev>` (also the author of `5a53124cb`). Nothing staged (`git diff --cached` empty).
- Tracked dirty paths at the end of this round: mine = `crates/jcode-app-core/src/server/{session_control,reload,reload_exec_target,runtime_upgrade,util,debug_command_exec}.rs`, `server.rs`, `tool/selfdev/reload.rs`, `src/cli/commands/report_info.rs`, `src/cli/commands_tests.rs`, `docs/SWARM_ARCHITECTURE.md`, `tests/e2e/worker_stop.rs`, this change dir. Not mine: `tool/mcp.rs`, `tool/registry_mcp_notification_tests.rs`, `crates/jcode-base/src/mcp/manager.rs`, `openspec/changes/handle-mcp-server-notifications/tasks.md`, `src/cli/terminal.rs`.
- Binary `target/selfdev/jcode` (built by `selfdev build` target=tui, request `selfdev-build-5ee5285162ce4e1699ae2f197bf1feec`, exit 0): `jcode v1.0.237-dev (0a442e8e3, dirty)`, sha256 `3dcbe06dc14da6c2af1b57af460e7a27d34ba864f58473ff776ffac436a51526`, `version --json` reports `control_reader_version: 2`. Built before the last test-only (`tests/e2e/worker_stop.rs`) and docs edits; no runtime source changed after it.
- Overlap with `add-swarm-handoff-contracts`: its proposal.md depends on this change (`depends_on: [..., make-worker-stop-durable]`) and consumes the generation/wait contract; this round touched none of its files. Boundary unchanged.

### 6.2 candidate compatibility gate
- Daemon-side, fail closed. `session_control::ControlStore::required_reader_floor()` = max `min_reader_version` over `session-control/*.json` (no field = 1, unparsable or non-numeric = this build's floor, unreadable directory = error, no directory = none). `reload_exec_target::ensure_candidate_control_floor(candidate)` returns Ok with no probe when there are no records; else runs `<candidate> --no-update version --json` (5 s timeout, 64 KiB cap, `JCODE_SOCKET` pointed at a nonexistent path so it never contacts a daemon) and requires `control_reader_version >= floor`. Absent field (pre-change build), lower value, failed/timed-out/garbage probe = refusal. `jcode version --json` now prints `control_reader_version` (= `CONTROL_MIN_READER_VERSION`).
- Hooks: `server/reload.rs::await_reload_signal` (vets the resolved exec target before `persist_reload_recovery_intents`/graceful shutdown, then execs that same target; refusal logs `refusing reload`, records trace `refused_control_floor`, writes reload state `Failed`, and the loop continues serving), `runtime_upgrade::prepare_upgrade`, `tool/selfdev/reload.rs::do_reload` (before publishing), `debug_command_exec.rs` `reload` (before publishing).
- Tests: `-p jcode-app-core --lib -- control_floor_tests` 6 passed (no records no probe; at/above floor accepted; below/legacy-without-field refused; unprobeable (garbage, nonzero exit, missing) refused; record floor 7 refuses candidate at 2 and accepts 7; legacy/corrupt records set floor and bytes are unchanged). Real binary: `worker_stop::reload_into_candidate_below_control_floor_is_refused_and_daemon_keeps_serving` (stopped worker, a newer-mtime fake `builds/stable/jcode` that reports no `control_reader_version`, public `reload force:true`): output `{"scenario":"reload_refused_below_floor","same_pid":true,"refused_logged":true,"before":{state stopped,revoked_through 1},"after":{same},"new_requests":0,"dms":[done,done]}`.
- Rollback and bytes: unchanged from Round 3 (docs "Compatibility floor and rollback"); the probe never writes control records.
- UNENFORCED paths (documented in `docs/SWARM_ARCHITECTURE.md`): cold start of a binary picked by the launcher, systemd unit or operator (no running daemon to vet it); any pre-change binary (never reads `session-control/`); file replacement between probe and exec on ordinary reloads (the packaged upgrade re-checks identity). Packaged `UpgradeRuntime` gate has no dedicated test; it is a one-line call to the same function.

### 7.1 real-binary acceptance (final binary above, isolated runs)
Command: `JCODE_REMOTE_CARGO=0 JCODE_WORKER_STOP_BINARY=target/selfdev/jcode scripts/dev_cargo.sh test -p jcode --test e2e worker_stop:: -- --include-ignored --test-threads=1` (private user+net namespace, owned JCODE_HOME/runtime/socket, loopback scripted provider, no credentials; shared daemon untouched).
- First run (`scratch/applyall/sd-ws-full.log`): 13 passed, 1 failed in 195.6 s. The failure was a stale assertion: `concurrent_wakes_after_settled_stop_admit_exactly_one_generation` expected generation G+2 but the daemon reported `generation 2, admitted 2, revoked_through 1` (stopped at G=1), the intended Round 5 behavior (resume pre-admits G+1 and its turn adopts it; two wakes replied `running` and `queued`, 4 provider requests all accounted for). Assertion corrected to G+1 (still fails on a second resume, which would give G+2).
- Rerun with the new scenario (`scratch/applyall/sd-ws-fix.log`): `worker_stop::` 15 passed, 0 failed, 211.6 s. Coverage: idle (churn worker2 stopped by force/owner), model-busy (`public_stop_prevents_post_stop_native_write`: marker absent), native-tool-busy (`public_stop_during_native_tool_...`: `stop_frames` stop_requested,draining,done, tool finished before `done`, no new model turn), stop/wake and resume races (`concurrent_stop_and_wake_...`, `concurrent_wakes_...`), owner control after churn plus unrelated denial plus ambiguity plus force (`owner_control_survives_churn_...`, `alias_stop_is_swarm_local_...`), stale wait replay (`wait_bound_to_stopped_generation_never_completes_from_resumed_work`: `completed:false`, member `done:false` after wake), claims transfer (`reassign_waits_for_stop_settlement_then_hands_the_task_off`: refused "not settled" while the 40 s child lives, then handed off to the successor), restart mid-stop and after settled/unresolved stop, graceful reload settled and draining, refused reload (6.2), shared-network refusal.
- Not exercised on the real binary: packaged `UpgradeRuntime` (needs a strictly newer signed candidate), non-Linux platforms, an older client binary.

### 7.2 gates (serial, `JCODE_REMOTE_CARGO=0 scripts/dev_cargo.sh`, script `scratch/applyall/sd-gates.sh`, log `scratch/applyall/sd-gates.log`; run through dev_cargo, not the `selfdev test` tool, no `JCODE_CARGO_GATE=off`)
- `test -p jcode-app-core --lib -- --test-threads=1`: exit 0, 2266 passed, 0 failed, 25 ignored.
- `test -p jcode-swarm-core`: 22 passed. `test -p jcode-protocol`: 125 passed, 1 ignored. `test -p jcode --test e2e`: 62 passed, 22 ignored (ignored = the real-binary set run above). `test -p jcode-harness-api -p jcode-harness-api-server -p jcode-sdk`: all ok (51, 202, 104, 32, 8, 5 ... passed).
- `clippy -p jcode-swarm-core -p jcode-protocol -p jcode-app-core -p jcode --all-targets --no-deps -- -D warnings`: exit 0.
- `git diff --check`: exit 0. `openspec validate make-worker-stop-durable --strict --no-interactive`: valid. `validate_dependencies.py --root /home/nyaptor/.jcode make-worker-stop-durable --json`: valid, closure size 1.
- Parallel (default threads) app-core lib was not rerun: its load-dependent flakes predate this change (Round 4, pristine base also fails 4).
