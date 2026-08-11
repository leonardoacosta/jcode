# Command Center

Status: experimental vertical-slice implementation for OpenSpec change `add-solidstart-command-center-vertical-slice`.

The Command Center is the browser-first supervision surface for Jcode initiatives. The approved first slice is a Jcode-daemon-hosted SolidStart application that shows durable initiative intent beside linked live execution state.

## Authority model

- The Jcode daemon owns authentication, browser hosting, snapshots, commands, ordered events, durable initiatives, linked schedules, Jcode run records, permissions, and rollback.
- Orca owns executable project identity and live runtime identity. Jcode stores canonical Orca references, normalized observations, correlations, evidence, and outcomes.
- The SolidStart client owns only layout, focus, filters, selections, scroll position, transient drafts, and other reversible interface state.
- The terminal Context Control Room remains a lightweight inspector. It is not a second command-center implementation.

## Implementation seams

The vertical slice deliberately extends existing Jcode authorities instead of creating a parallel application backend:

| Concern | Existing authority and exact seam | Command Center integration |
|---|---|---|
| Daemon transport and lifecycle | `crates/jcode-app-core/src/server.rs` (`Server::finish_startup_after_bind`) and `crates/jcode-app-core/src/server/runtime.rs` (`RuntimeTaskScope`) | `crates/jcode-app-core/src/command_center.rs::spawn_managed_http_host` starts the loopback HTTP host inside the daemon task scope, so reload and shutdown cancel and join it. |
| Durable initiatives | `crates/jcode-app-core/src/goal.rs` and the existing goal persistence used by the initiative tool | `GoalInitiativeRepository` projects and revision-checks those records through public Command Center DTOs. It does not add a frontend database. |
| Ambient scheduling | `crates/jcode-app-core/src/ambient.rs` and persisted ambient queue state | `AmbientScheduleProjectionSource` exposes initiative-linked evidence without becoming global schedule administration. |
| Jcode run evidence | Existing persisted session/run records | `SessionRunProjectionSource` reads linked run projections and preserves Jcode and Orca references. |
| Permissions and browser security | Command Center auth context, scoped browser sessions, and same-origin HTTP middleware in `crates/jcode-command-center/src/lib.rs` | The browser receives short-lived scoped session material only. Mutations require CSRF, origin, revision, and idempotency evidence. |
| Orca authority | The installed Orca CLI and canonical Orca project/run identifiers | `OrcaCliAdapter` uses the verified `status --json` observation surface. Unsupported start, retry, and cancel mutations fail closed until Orca exposes an approved lifecycle contract. |
| Browser contract | Public Rust DTOs and `generate-command-center-types` in `crates/jcode-command-center` | Generated output is committed at `apps/command-center/src/generated/command-center-contract.ts` and drift-checked by `scripts/check-command-center-contracts.sh`. |

The pre-change seams were exercised after integration through focused goal, ambient, browser, TUI initiative/control-room, desktop2, and server checks with the feature disabled. The managed-host path was separately exercised through an isolated daemon, protected API, real browser deep link, security probes, and listener shutdown.

## Enablement and listener posture

The command center must be disabled by default until its security, compatibility, contract, browser, and managed-topology gates pass. When enabled, the listener must bind to loopback unless an operator explicitly configures authenticated remote access and an origin allowlist.

Required startup evidence:

1. Whether the command center is enabled or disabled.
2. Bound address and port.
3. Security mode: loopback, authenticated tunnel, or rejected remote configuration.
4. Location of logs and the isolated daemon/socket used by the test harness.

Non-loopback binding without authenticated transport and allowed origins is a startup failure.

The daemon lifecycle reads these experimental environment variables:

| Variable | Default | Purpose |
|---|---|---|
| `JCODE_COMMAND_CENTER_ENABLED` | unset/disabled | Set to `1`, `true`, or `yes` to start the managed host. |
| `JCODE_COMMAND_CENTER_BIND_ADDR` | `127.0.0.1:0` | Loopback listener address. Use a fixed port for acceptance or SSH forwarding. |
| `JCODE_COMMAND_CENTER_ASSET_DIR` | unset | Path to the built SolidStart `.output/public` directory. |
| `JCODE_COMMAND_CENTER_ALLOWED_ORIGINS` | empty | Comma-separated browser origins. The bound loopback origin is added automatically when empty. |
| `JCODE_COMMAND_CENTER_AUTHENTICATED_REMOTE` | disabled | Required together with an allowlist before any non-loopback bind is accepted. |
| `JCODE_ORCA_CLI` | `orca` | Orca CLI executable. On the managed Linux host this is normally `/home/nyaptor/.local/bin/orca-ide`. |

The host is owned by the daemon runtime task scope. Daemon shutdown and reload cancel the lifecycle task, gracefully stop the HTTP listener, and release the port. There is no independently managed Node service in the deployed topology.

## Browser sessions

Browser sessions are short lived and scoped to command-center routes and commands. Bootstrap tokens must not contain provider credentials and must not be reusable bearer secrets in URLs, local storage, or generated fixtures. State-changing requests require same-origin and CSRF proof plus a client-generated idempotency key.

## Repository-local gates

These gates are deterministic and safe to run in the repository. They must not use the shared user daemon.

| Gate | Command | Expected result |
|---|---|---|
| Contract fixture sanity | `bash scripts/check-command-center-contracts.sh --fixture-only` | Exit 0. Fixture schema, event order, and closed runtime command set are valid. |
| Generated contract drift | `bash scripts/check-command-center-contracts.sh` | Exit 0 only when generated TypeScript exactly matches Rust DTO output. Fails closed until `JCODE_COMMAND_CENTER_CONTRACT_GENERATOR` is supplied. |
| Security fixture sanity | `bash scripts/test-command-center-security.sh --fixture-only` | Exit 0. Fixture contains no secret-looking values and the closed runtime command set is represented. |
| Repository acceptance fixture sanity | `bash scripts/test-command-center.sh --fixture-only` | Exit 0. Deterministic initiative, schedule, Orca identity, and unsupported-command fixture invariants pass. |
| Tunnel fixture sanity | `bash scripts/test-command-center-tunnel-fixture.sh --fixture-only` | Exit 0. Stream scope is single-authority and fixture events do not expose host paths. |
| Full repository acceptance | `bash scripts/test-command-center.sh` | Exit 0 only with an isolated daemon or isolated base URL, Playwright local acceptance, and Orca-unavailable acceptance passing. |
| Strict OpenSpec and diff hygiene | `openspec validate add-solidstart-command-center-vertical-slice --strict --no-interactive && git diff --check` | Exit 0. |

The repository-local gates distinguish deterministic fixture checks from real browser acceptance. Fixture-only modes are pre-implementation sanity checks and are not substitutes for the full vertical-slice acceptance workflow.

## Managed Mac/homelab terminal post gate

The managed topology gate is intentionally separate from repository-local gates because it depends on external hosts, SSH aliases, and a managed command-center listener.

Command:

```bash
bash scripts/test-command-center-mac-smoke.sh --mac-host mac --jcode-host homelab
```

Expected result:

- A Mac-origin session can authenticate through the approved tunnel.
- The initiative route is served by the homelab daemon.
- Provider credentials remain on the homelab and are not visible to the browser.
- Repository, tool, and runtime evidence resolve to homelab resources.

Failure of this post gate blocks rollout beyond local development, but it should be reported separately from deterministic repository-local gate status.

## Thresholds

The first slice must record these measurements before enablement-by-default is considered:

| Measurement | Initial threshold |
|---|---|
| Cold command-center startup | P95 under 2.5 seconds after daemon readiness. |
| Additional idle memory | Under 150 MiB over disabled-daemon baseline. |
| Additional idle CPU | Under 2% sustained over 60 seconds on the managed homelab. |
| Event update latency | P95 under 250 ms from daemon event emission to rendered status update in the browser. |
| Reconnect to authoritative state | P95 under 3 seconds after transient disconnect. |
| Large timeline rendering | 10,000 events remain bounded through virtualization with no unbounded redraw or browser hang. |
| Shutdown cleanup | No leaked child process, listener, socket, or temporary directory after daemon shutdown. |

If a threshold is exceeded, keep the feature behind the experimental flag and document the blocker in the durable initiative.

### Current measurement status

Repository-local functional measurements are recorded, but task 7.6 remains open until the complete threshold suite runs on the managed homelab hardware. The current evidence is:

- Production SolidStart packaging succeeds and is served directly from `.output/public` by the Rust-managed host. There is no independently exposed Node listener or second workflow authority.
- The real isolated daemon reached the protected API and rendered an authenticated browser deep link successfully.
- Listener shutdown released the configured port and left no managed Command Center child process.
- The 10,000-event client fixture remains bounded to 40 rendered timeline rows through virtualization.
- Repository-local event, reconnect, and resource measurements are not substitutes for the required 60-second idle CPU/memory and P95 managed-host measurements. Those remain a rollout blocker rather than an inferred pass.

The rejected alternative is a daemon-supervised SolidStart server process. It would add a second private listener, extra packaging and shutdown state, and no benefit for this client-rendered slice. The selected topology builds static assets once and keeps all HTTP, authentication, commands, events, and lifecycle ownership in the Rust daemon.

## Operations

- Use an isolated socket and home directory for every test run, for example under `${XDG_RUNTIME_DIR}` or a temporary directory.
- Do not run acceptance against `~/.jcode/builds/shared-server/jcode` unless the test explicitly targets the managed topology post gate.
- Store deterministic fixtures under `fixtures/command-center/` rather than in `apps/command-center` so support gates remain independent of frontend implementation churn.
- Capture logs with command-center security mode, bound address, stream ID, snapshot revision, and command correlation ID. Redact bootstrap tokens, CSRF tokens, provider credentials, and local filesystem secrets.

## Rollback

Rollback is configuration first:

1. Disable the experimental command-center flag.
2. Restart or reload the daemon so no command-center listener is active.
3. Verify existing TUI, initiative, schedule, ambient runner, daemon socket clients, browser automation tool, and desktop2 flows still run with the web feature disabled.
4. Preserve initiative, schedule, session, and Orca reference data. This slice must not make persisted records unreadable by older clients.
5. If a daemon-supervised SolidStart child process exists, verify shutdown removed the process and private listener.

Rollback must not delete durable initiative records or Orca evidence unless a separate user-approved data migration says so.
