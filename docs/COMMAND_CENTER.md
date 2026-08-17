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
| Decision Inbox | Provider-neutral records in `jcode-intake-types`, written by the Telegram and Slack intake adapters | The daemon reads the durable SQLite store and exposes normalized provenance, category, approval state, and delivery evidence through an authenticated read-only endpoint. Provider credentials and raw payloads remain server-side. |
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
| `JCODE_DECISION_INBOX_DB` | `$JCODE_HOME/intake/decision-inbox.sqlite` | Optional override for the provider-neutral durable Inbox store shared by Telegram, Slack, and the Command Center read model. |
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
| Full repository acceptance | `bash scripts/test-command-center.sh` | Builds the SolidStart assets, starts an isolated credential-free Jcode daemon and managed loopback host, then exits 0 only when repository-local and Orca-unavailable Playwright acceptance pass. Set `JCODE_COMMAND_CENTER_JCODE_BIN`, `JCODE_COMMAND_CENTER_DAEMON_CMD`, or `JCODE_COMMAND_CENTER_BASE_URL` only to override the repository defaults. |
| Strict OpenSpec and diff hygiene | `openspec validate add-solidstart-command-center-vertical-slice --strict --no-interactive && git diff --check` | Exit 0. |

The repository-local gates distinguish deterministic fixture checks from real browser acceptance. Fixture-only modes are pre-implementation sanity checks and are not substitutes for the full vertical-slice acceptance workflow.

## Acceptance traceability

The following matrix records the observed result for each changed public boundary. A passing fixture is listed only as supporting evidence. The real isolated-daemon path is identified separately.

| Public requirement or output | Concrete check | Observed result |
|---|---|---|
| Rust protocol metadata, owned IDs, snapshots, commands, typed errors, scoped replay, and generated TypeScript | `cargo test -p jcode-command-center --lib` and `bash scripts/check-command-center-contracts.sh` | 20 Rust tests passed. Generated TypeScript matched the Rust DTO generator with no drift. Unknown events, cursor scope, replay gaps, serialization, and ID ownership were covered. |
| Disabled-by-default, loopback-only listener posture, remote-bind rejection, browser sessions, CSRF, origin, expiry, content type, and security headers | Command Center HTTP host tests plus `bash scripts/test-command-center-security.sh` against the isolated managed daemon | Host tests passed. The real protected API rejected unauthenticated access and the security script passed without exposing provider credentials or reusable browser secrets. |
| Daemon-owned startup, static SPA hosting, deep-route fallback, shutdown, and port release | Start an isolated `target/selfdev/jcode` with unique `XDG_RUNTIME_DIR`, `JCODE_HOME`, `JCODE_SOCKET`, fixed loopback port, and built `.output/public`; request `/initiatives`; stop the daemon; probe the port | The daemon served the SolidStart route from the managed listener. Shutdown terminated the host and released the configured port. No independent Node listener was used. |
| Durable initiative list/detail/update/checkpoint and revision/idempotency behavior | `cargo test -p jcode-app-core command_center --lib`, Command Center domain tests, Solid interaction tests, and Playwright update/checkpoint workflows | Goal-backed reads and revision-checked saves passed. The UI showed pending state and installed the authoritative replacement snapshot after step and checkpoint commands. |
| Telegram and Slack Decision Inbox projection | Focused tests for `jcode-intake-types`, `jcode-intake-telegram`, `jcode-intake-slack`, and `jcode-command-center`; Solid component tests; deterministic Playwright content acceptance; isolated managed-host acceptance | Provider-neutral normalization, source identity, restart persistence, deduplication, reconnect redelivery, credential redaction, authenticated HTTP projection, responsive UI rendering, and empty/failure behavior passed. Live provider ingestion remains credential-gated and is not claimed by repository fixtures. |
| Linked ambient schedule and persisted Jcode run projections | App-core Command Center tests and Playwright schedule/run deep-link workflows | Matching ambient evidence and persisted run references were projected without moving schedule or run authority into the frontend. Schedule evidence and stable run links rendered. |
| Orca observation and capability boundary | Live `/home/nyaptor/.local/bin/orca-ide status --json`, app-core adapter tests, and Orca-unavailable Playwright project | Supported observation parsed successfully. Unsupported start/retry/cancel calls returned typed unsupported-capability errors before invocation. Durable initiative controls remained usable while runtime controls were disabled. |
| SolidStart routes, split workspace, state model, accessibility, embedded width, reduced motion, and timeline bounds | `pnpm --dir apps/command-center format:check`, `lint`, `typecheck`, `test`, `build`, and both Playwright projects | Formatting, lint, typecheck, production build, 16 component tests, and 18 fixture browser workflows passed. Keyboard focus, semantic headings, announcements, narrow layout, reduced motion, and 10,000-event virtualization remained bounded to 40 rendered rows. |
| Real browser bootstrap and deep-link rendering through the Jcode daemon | Playwright repository-local project pointed at the real isolated managed listener | The browser bootstrapped a short-lived session, loaded `/initiatives/.../runs/...`, rendered the semantic Command Center heading, and queried the protected API successfully. CSP, missing-resource handling, and heading defects found by this path were fixed before the pass. |
| Disconnect, replay, snapshot replacement, unknown events, and local UI-state preservation | Projection-store unit tests and named Playwright reconnect workflow | Next-sequence events applied, gaps triggered reconciliation, replacement snapshots installed atomically, unknown events did not corrupt state, and client-owned layout/selection state survived replacement. |
| Existing interfaces when the feature is disabled | Focused app-core goal/ambient/browser tests, TUI control-room/initiative tests, desktop2 check, and Jcode server tests | All invoked compatibility commands exited 0. Existing initiative, schedule, browser-tool, TUI, desktop, and daemon behavior remained available with the listener disabled. |
| Managed Mac/homelab topology | `bash scripts/test-command-center-tunnel-fixture.sh --fixture-only` and `bash scripts/test-command-center-mac-smoke.sh --mac-host mac --jcode-host homelab` | Deterministic stream/path isolation passed. The systemd-managed homelab listener served only on `127.0.0.1:43118`; a real headless Google Chrome process on the Mac rendered the initiative route through an SSH local forward, observed durable initiative content, and exposed no provider-secret markers. |
| Full rollout gate | OpenSpec task 8.4 | Blocked, not passed. The approved Orca mutation contract, managed-hardware P95/resource measurements, and a stable repository-wide fmt/clippy window remain required. |

The deployed listener was also exercised directly by the managed security gate,
repository-local Playwright bootstrap, Orca-unavailable browser project, and default
tunnel gate. Security passed with the expected rejected 401/422 probes; both real
browser projects passed; the tunnel gate returned exit 0.

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

The managed gate passed on 2026-08-11 using the systemd-owned homelab daemon at
`127.0.0.1:43118`, an SSH local forward created from the Mac, and the Mac's native
Google Chrome in headless mode. The rendered DOM contained the live durable
`jcode-command-center` initiative and no provider-secret markers. Failure of this
post gate blocks rollout beyond local development and must be reported separately
from deterministic repository-local gate status.

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
- On the managed homelab, an isolated disabled/enabled lifecycle measurement produced 606 ms disabled readiness, 688 ms enabled HTTP readiness, 108/107 ms shutdown, and a 2.2 MiB initial RSS delta.
- A simultaneous 60-second isolated comparison measured 0.017% disabled CPU, 0.000% enabled CPU, a 1.5 MiB enabled RSS delta, and 72 KiB growth during the enabled sample. This passes the idle CPU and memory thresholds without attributing unrelated activity from the shared production daemon to the Command Center.
- The live systemd-managed endpoint returned 100 initiative-route requests at 0.344 ms P95 HTTP latency. This is transport evidence, not a substitute for daemon-event-to-render latency.
- Event-to-render P95 and reconnect-to-authoritative-state P95 remain unmeasured on the managed topology. Task 7.6 therefore remains open rather than inferring those results from fixture timing.

The rejected alternative is a daemon-supervised SolidStart server process. It would add a second private listener, extra packaging and shutdown state, and no benefit for this client-rendered slice. The selected topology builds static assets once and keeps all HTTP, authentication, commands, events, and lifecycle ownership in the Rust daemon.

Release deployment runs `scripts/install_command_center_assets.sh` from the exact
detached commit, installs regular asset files at
`~/.jcode/command-center/public`, and only then reloads the managed Jcode daemon.
Frontend-only commits participate in the same post-commit deployment queue.

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

## Orca Command Center orchestration bridge

OpenSpec change `optimize-orca-command-center-orchestration` layers policy on top of the existing vertical slice rather than replacing it. The approved skill boundary is:

- `orca-cli` owns version-matched Orca runtime mechanics and full handoff operations.
- `orchestration` owns generic supervised Run, Task, Dispatch, and worker coordination.
- `jcode-command-center-orchestration` owns Jcode Command Center policy: initiative and schedule authority, durable-state correlation, permission gates, lifecycle projection, identifier preservation, degraded-state handling, and acceptance evidence.

The Command Center must choose one orchestration pattern before mutating state: observation-only projection, full handoff, supervised DAG coordination, approval-gated mutation, or scheduled retry. Each pattern records the authority that made the decision and the authority that executed it. If the required canonical identity or capability proof is missing, the bridge fails closed and keeps runtime controls disabled.

### Identifier envelope

Command Center records must preserve distinct identifiers instead of collapsing them into a single runtime ID. The envelope includes Jcode initiative/run IDs, canonical Orca repository/project IDs, Orca runtime run IDs, Task IDs, Dispatch IDs, worktree IDs, terminal IDs, schedule attempt IDs, correlation IDs, and idempotency keys. Runtime IDs are evidence, not canonical project identity. Replayed or recovered events must retain the original correlation and idempotency evidence so a retry can be distinguished from a duplicate settlement.

### Replay, scheduling, and cleanup rules

Replay gaps invalidate only the affected stream scope and request an authoritative replacement snapshot. Scheduled triggers enter the same pattern-selection, permission, correlation, idempotency, and receipt-settlement path as interactive commands; every retry creates a distinct causal dispatch attempt. Partial cleanup is represented as a recoverable degraded state with owned resources, attempted cleanup actions, and remaining safe next actions. Unsupported Orca start, retry, cancel, or cleanup mutations remain typed unsupported-capability outcomes until a verified Orca contract exists.
