## 1. Baseline and architecture seam

- [ ] 1.1 Record the current daemon transport, server lifecycle, initiative persistence, ambient scheduling, permissions, and Orca integration seams in implementation notes tied to exact source paths; verify with targeted existing tests before changing code.
- [ ] 1.2 Spike SolidStart build and SSR integration against the Rust daemon and select either in-process asset/SSR hosting or a daemon-supervised private child process; document measured startup, idle-memory, shutdown, and packaging results and stop if the option requires an independently exposed listener or workflow authority.
- [ ] 1.3 Define the command-center crate/package ownership boundaries, affected workspace manifests, generated-client location, and feature/config flag without moving existing domain behavior into the frontend.
- [ ] 1.4 Add a migration-ledger update that maps the source documents listed in `design.md` to absorbed, retained, completed-foundation, partially-superseded, or later-milestone decisions; do not archive any source in this child change.

## 2. Versioned contract foundation

- [ ] 2.1 Add public Rust DTOs for protocol metadata, authoritative snapshots, initiative projections, linked schedule projections, Jcode run references, normalized Orca references, freshness, available actions, and typed errors.
- [ ] 2.2 Add typed command envelopes and results with authentication context, expected entity revision, client idempotency key, pending/completed/failed state, and correlation IDs.
- [ ] 2.3 Add ordered event envelopes with protocol version, authorization-scoped stream ID, stream-local monotonic sequence, timestamp, source, entity references, typed payload, replay cursor, and snapshot-required response; filter authorization before replay persistence/emission and rotate streams when session or subscription scope changes.
- [ ] 2.4 Implement deterministic TypeScript generation from the public Rust DTO boundary and add `scripts/check-command-center-contracts.sh` to regenerate into a temporary directory, compare output, and fail on stale generated files.
- [ ] 2.5 Add serialization, backward/forward compatibility, unknown-event, ID ownership, and generated-output tests for the contract package.

## 3. Daemon-owned web host and security

- [ ] 3.1 Add an experimental command-center configuration that is disabled by default and binds the HTTP listener to loopback when enabled.
- [ ] 3.2 Serve the SolidStart application, snapshots, typed commands, and event stream through one Jcode-managed lifecycle and verify clean startup, shutdown, restart, and disabled-mode behavior.
- [ ] 3.3 Implement trusted local browser bootstrap and short-lived scoped browser sessions without placing provider credentials or reusable bearer secrets in URLs or client persistence.
- [ ] 3.4 Enforce same-origin, CSRF, origin allowlist, method/content-type, session expiry, and security-header requirements for browser requests.
- [ ] 3.5 Reject non-loopback binding unless explicit authenticated remote transport and allowed origins are configured; surface the exposed address and security mode at startup.
- [ ] 3.6 Add `scripts/test-command-center-security.sh` and focused tests for unauthenticated reads, CSRF mutation attempts, expired sessions, disallowed origins, non-loopback misconfiguration, secret leakage, replay-cursor scope isolation, and deterministic tunnel fixtures; the script SHALL exit nonzero on any failed case.

## 4. Query, command, and event services

- [ ] 4.1 Implement initiative list and detail query services over existing durable initiative storage without exposing internal persistence structs.
- [ ] 4.2 Add explicit initiative-to-Orca-project, initiative-to-Jcode-run, initiative-to-schedule, and Jcode-run-to-Orca-run reference models with migration-safe optional fields and canonical Orca IDs.
- [ ] 4.3 Implement initiative milestone/step update, checkpoint, blocker, and next-action commands with revision checks, authorization, idempotency, and authoritative result payloads.
- [ ] 4.4 Implement linked schedule projection with cadence, timezone, next fire, last result/run, retry, missed-wake, stale-claim, and failure evidence while leaving global schedule administration out of scope.
- [ ] 4.5 Implement the Orca adapter for versioned project/run/worker/terminal/gate observations and only `start_initiative_run`, `retry_linked_run`, and `cancel_linked_run`; enforce the design capability table, preserve Orca identifiers and last-observed timestamps, and reject every other runtime mutation before adapter invocation.
- [ ] 4.6 Implement authorization-scoped event persistence/replay or bounded replay buffers with stream IDs, stream-local sequence validation, cursor-scope rejection, authorization/subscription rotation, gap detection, and snapshot-required fallback.
- [ ] 4.7 Add domain tests for stale initiative revisions, duplicate commands, pending downstream actions, missing links, unknown entities, unavailable Orca, capability negotiation, replay success, and replay gaps.

## 5. SolidStart application foundation

- [ ] 5.1 Create the SolidStart workspace with locked dependency versions, formatting/lint/typecheck/test commands, generated client consumption, and no independent domain database or server functions that bypass Jcode.
- [ ] 5.2 Implement the application shell, authenticated bootstrap, route loaders, connection status, error boundary, keyboard focus conventions, accessible live regions, and host-neutral embedded-width layout contract.
- [ ] 5.3 Implement `/initiatives` with authoritative loading, empty, unavailable, error, resumable, historical, blocker, progress-evidence, and freshness states.
- [ ] 5.4 Implement `/initiatives/:initiativeId` and `/initiatives/:initiativeId/runs/:runId` with relationship validation and stable deep links.
- [ ] 5.5 Add a client projection store that installs snapshots atomically, applies only next-sequence events, preserves client-owned layout/filter/selection/draft state, and reconciles through replay or snapshot replacement.
- [ ] 5.6 Add client tests for expired authentication, route-not-found/forbidden, event gaps, unknown events, disconnect/reconnect, snapshot replacement, and preservation of local interface state.

## 6. Split initiative and execution workspace

- [ ] 6.1 Build the resizable durable pane for outcome, status, current milestone, steps, success criteria, blockers, next actions, child references, linked schedules, and checkpoint history.
- [ ] 6.2 Build initiative editing and checkpoint controls with explicit pending state, stale-revision reconciliation, permission handling, inline inspect/retry/dismiss recovery, and no optimistic claim of authoritative success.
- [ ] 6.3 Build the live execution pane for linked Jcode run health, normalized Orca graph, workers/sessions, terminals, gates/approvals, attention items, event timeline, freshness, and explicit no-run state.
- [ ] 6.4 Disable or hide unsupported runtime actions using server-supplied capabilities and render stale/unavailable Orca state without degrading durable initiative management.
- [ ] 6.5 Verify keyboard-only operation, visible focus, semantic headings/regions, status announcements, reduced-motion behavior, narrow embedded-surface layout, and large event-list virtualization.
  - Run `pnpm --dir apps/command-center test -- accessibility embedded-layout virtualization`; expected result is exit 0 with keyboard-only focus traversal, accessible status announcements, reduced-motion assertions, the minimum embedded-width fixture, and bounded large-list rendering all passing.
  - touches: `apps/command-center/src/`, `apps/command-center/tests/`, and accessibility/embedded-width fixtures (new)
  - depends on: 6.1, 6.2, 6.3, 6.4
- [ ] 6.6 Add component and interaction tests covering every loading, empty, unavailable, stale, error, data, pending-command, failed-command, and recovery state required by the specs.

## 7. Managed topology and acceptance workflow

- [ ] 7.1 Add a non-production test configuration that launches one isolated Jcode daemon, command-center web host, deterministic initiative fixture, schedule fixture, and compatible Orca runtime/fixture without using the shared user daemon.
- [ ] 7.2 Add `scripts/test-command-center.sh` and Playwright coverage for authenticated launch, initiative discovery, split route rendering, milestone/step update, checkpointing, linked schedule evidence, live event update, run deep link, scoped-cursor rejection, disconnect, replay, snapshot reconciliation, and initiative resume; expected script result is exit 0 with every named workflow passing.
- [ ] 7.3 Run the same acceptance workflow with Orca intentionally unavailable and verify durable initiative operations continue while runtime state is explicitly degraded and unsafe actions cannot succeed.
  - Run `pnpm --dir apps/command-center test:e2e -- --project orca-unavailable`; expected result is exit 0 with initiative update/checkpoint/resume passing, the execution pane marked unavailable with last-observed evidence, and every unsafe runtime command absent or rejected.
  - touches: `apps/command-center/e2e/` and isolated Orca-unavailable fixtures (new)
  - depends on: 7.1, 7.2
- [ ] 7.4 Verify the authenticated bridge behavior with a repository-local deterministic loopback/tunnel fixture.
  - Run `bash scripts/test-command-center-tunnel-fixture.sh`; expected result is exit 0 with the simulated remote client unable to reach a non-loopback listener directly, able to authenticate through the forwarded endpoint, unable to reuse another stream cursor, and unable to observe provider secrets or host-local execution paths.
  - touches: `scripts/test-command-center-tunnel-fixture.sh` and deterministic bridge fixtures (new)
  - depends on: 3.6, 7.2
- [ ] 7.5 Perform the terminal post gate on the managed Mac/homelab topology after repository-local acceptance passes.
  - Preconditions: SSH aliases `mac` and `homelab`, the managed homelab Jcode service, and an enabled test-only command-center listener reachable only through the approved tunnel.
  - Run `bash scripts/test-command-center-mac-smoke.sh --mac-host mac --jcode-host homelab`; expected result is exit 0 with a Mac-origin browser session authenticated through the tunnel, the initiative route served by the homelab daemon, no browser-visible provider secret, and repository/tool/runtime evidence resolving only to homelab resources.
  - touches: `scripts/test-command-center-mac-smoke.sh` and terminal post-gate evidence output (new)
  - depends on: 7.3, 7.4
- [ ] 7.6 Measure cold start, idle CPU, idle memory, event-update latency, reconnect time, and large-timeline behavior; record thresholds and fail the gate on unbounded redraw, polling, event growth, or leaked child processes.

## 8. Documentation, compatibility, and rollout

- [ ] 8.1 Document command-center enablement, loopback default, authenticated remote bridge, browser-session behavior, troubleshooting, logs, shutdown, and rollback.
- [ ] 8.2 Update architecture documentation to mark SolidStart as the canonical interactive browser UI, the Context Control Room as a lightweight inspector, Orca as project/live-runtime authority, and the future desktop `CommandCenter` surface as a later milestone.
- [ ] 8.3 Verify existing TUI, initiative tool, `/initiatives` commands, schedules, ambient runner, daemon socket clients, browser automation tool, and desktop2 build remain behaviorally compatible when the web feature is disabled.
  - Run `cargo test -p jcode-app-core goal`, `cargo test -p jcode-app-core ambient`, `cargo test -p jcode-app-core browser`, `cargo test -p jcode-tui control_room`, `cargo test -p jcode-tui initiatives`, `cargo check --profile selfdev -p jcode-desktop2`, and `cargo test --profile selfdev -p jcode server`; expected result is exit 0 for every command with the web feature disabled and no existing client, initiative, schedule, browser-tool, or desktop contract regression.
  - touches: compatibility tests adjacent to the affected Rust modules and existing client fixtures
  - depends on: 3.2, 4.7, 5.6, 6.6
- [ ] 8.4 Run Rust formatting/lint/typecheck/tests, Solid formatting/lint/typecheck/tests, generated-contract checks, security tests, Playwright acceptance tests, OpenSpec strict validation, and `git diff --check`; record exact observed results.
  - Run `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check --profile selfdev --workspace`, `bash scripts/check-command-center-contracts.sh`, `bash scripts/test-command-center-security.sh`, `pnpm --dir apps/command-center format:check`, `pnpm --dir apps/command-center lint`, `pnpm --dir apps/command-center typecheck`, `pnpm --dir apps/command-center test`, `bash scripts/test-command-center.sh`, `bash scripts/test-command-center-tunnel-fixture.sh`, `openspec validate add-solidstart-command-center-vertical-slice --strict --no-interactive`, and `git diff --check`; expected result is exit 0 for every command, no stale generated files, no failing security scenario, and no skipped repository-local acceptance path. Task 7.5 supplies the separate managed-topology terminal post gate.
  - touches: generated contract output, test reports, OpenSpec artifacts, and repository metadata only
  - depends on: 7.3, 7.4, 7.6, 8.1, 8.2, 8.3
- [ ] 8.5 Update the durable `jcode-command-center` initiative with the final milestone status, evidence, blockers, next steps, and links to the containing commit and follow-on scheduling, Orca, portfolio, and desktop child changes.
