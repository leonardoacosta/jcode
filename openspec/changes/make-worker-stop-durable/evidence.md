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
