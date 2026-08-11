## Context

Jcode already owns durable initiatives (`Goal` internally), scheduled items, sessions, approvals, directives, background execution, and the terminal Context Control Room. These capabilities are reliable but fragmented: users cannot see durable intent, current execution, attention items, and recovery actions in one interactive surface. The current `Alt+O` overlay is intentionally transient and explicitly does not allocate runtime resources.

The approved product direction is browser-first and desktop-aware:

- the canonical interactive command center is a SolidStart application;
- the Jcode daemon remains the only workflow authority and hosts the application;
- the first flagship screen places durable initiative state beside live execution;
- `prefix+o`/`Alt+O` remains lightweight and eventually opens or focuses the contextual web route rather than becoming a second implementation;
- Orca owns executable project identity and live execution identity; Jcode stores references and durable outcomes;
- the same command-center routes and generated client contracts must later be hostable as a Jcode Desktop `CommandCenter` surface.

This child change implements only the first vertical slice of the global `jcode-command-center` initiative. Later milestones own full scheduling-sidecar management, portfolio/global inbox views, desktop integration, and archival of superseded references.

### Existing sources and disposition for this child

| Source | Current role | Disposition in this change |
|---|---|---|
| `docs/WORKFLOW_AUTOMATION_ROADMAP.md` | Inbox, approvals, schedules, handoffs, and earlier UI alternatives | **Absorbed in part**: server authority, snapshots/events/commands, attention semantics. The prior HTMX recommendation is superseded by the approved SolidStart direction. Portfolio lanes remain later work. |
| `docs/CONTEXT_ARCHITECTURE.md` | Semantic/execution hierarchy and temporary Control Room | **Retained**: provenance and lightweight overlay boundary. The browser command center becomes the interactive surface. |
| `docs/CONTEXT_CONTROL_ROOM_EVIDENCE.md` | Verified current overlay behavior and limitations | **Completed foundation**: no behavior change in this child. |
| `docs/DESKTOP_SUPERAPP_WORKSPACE.md` | Spatial desktop surface model | **Retained with extension**: later add a `CommandCenter` surface. |
| `docs/DESKTOP_APP_ARCHITECTURE.md` | Native custom-rendered desktop and typed local protocol | **Partially superseded by approved product direction**: the command-center surface must be able to host the SolidStart route application. The rest of the desktop may remain native. Final transport/embedding mechanics belong to the desktop child change. |
| `docs/MAC_HOMELAB_SSH_TOPOLOGY.md` | Mac control surface and homelab authority | **Retained**: remote browser access must cross an explicit authenticated tunnel or bridge; no unauthenticated LAN listener. |
| `docs/AMBIENT_MODE.md` | Scheduling and ambient execution foundation | **Retained**: full schedule management is a later milestone; this slice projects linked schedule state only. |

## Goals / Non-Goals

**Goals:**

- Establish a daemon-hosted SolidStart workspace and a typed command-center protocol.
- Deliver `/initiatives`, `/initiatives/:initiativeId`, and `/initiatives/:initiativeId/runs/:runId`.
- Show initiative outcome, current milestone, steps, blockers, next actions, linked child work, and linked schedules beside live execution state.
- Allow initiative milestone/step updates, checkpointing, and safe run actions through idempotent typed commands.
- Project linked Orca execution without copying Orca's project/run authority into a second writable store.
- Reconcile reconnects from authoritative snapshots plus resumable ordered events.
- Provide explicit loading, unavailable, stale, disconnected, failed-command, and recovery states.
- Keep the frontend host-neutral enough for later desktop-surface embedding.

**Non-Goals:**

- Rebuilding the entire TUI or expanding the temporary Control Room into a full application.
- Implementing the global work inbox, complete approvals queue, portfolio overview, or all schedule administration.
- Replacing Orca worktree, worker, terminal, gate, or orchestration ownership.
- Implementing the desktop `CommandCenter` surface in this child change.
- Exposing provider credentials, secrets, arbitrary filesystem access, or an unauthenticated network listener to the browser.
- Archiving the referenced architecture documents before the umbrella migration ledger verifies their remaining requirements.

## Decisions

### 1. SolidStart is the canonical interactive web client

Use SolidStart for SSR, routing, forms, streaming hydration, and fine-grained reactive updates. The first route is intentionally highly interactive; using HTMX would require an expanding custom JavaScript state layer for graph selection, ordered events, optimistic commands, timeline updates, and reconnect reconciliation.

**Alternatives considered:**

- **HTMX:** rejected as the canonical client because the flagship route has high-frequency, interdependent client state. It remains viable for isolated operational pages if ever justified.
- **Native desktop first:** rejected for this milestone because it slows visual iteration and delays a remotely accessible command center.
- **React/Vite:** rejected in favor of Solid's fine-grained reactivity and lower update overhead for dense live surfaces.

### 2. The Jcode daemon hosts the application and owns all domain behavior

The existing daemon gains a command-center web host that serves the built SolidStart assets, supplies SSR data through daemon-owned query services, accepts commands, and exposes the event stream. SolidStart server code MUST NOT create a second workflow database or independently implement initiative, schedule, approval, run, or permission rules.

The listener is loopback-only by default. Remote use requires an explicit authenticated SSH/Tailscale bridge configured by the client. This is the strong reason to permit HTTP for the browser surface while preserving the existing user-owned socket/pipe as the default native/TUI transport.

**Alternative considered:** a separately managed Node/SolidStart service. Rejected because it creates another persistent service, authentication boundary, deployment lifecycle, and temptation to duplicate domain logic.

### 3. Snapshot + commands + ordered events is the client contract

Each page begins from an authoritative versioned snapshot. Mutations use typed commands with a client-generated idempotency key. Live changes arrive as ordered event envelopes containing protocol version, stream ID, stream-local sequence, timestamp, source, entity references, and payload.

The initial browser transport uses HTTP for snapshots/commands and SSE for server-to-client events. WebSocket is reserved for a later requirement that needs sustained bidirectional streaming. Each authenticated subscription receives an authorization-scoped stream ID; sequence numbers are monotonic only inside that stream. Authorization filtering occurs before events enter the replayable stream, so a cursor cannot reveal, count, or replay unrelated entities. Reconnect sends the stream ID and last accepted sequence. A session, authorization, route-scope, or subscription change creates a new stream and snapshot. If replay is unavailable or a gap is detected, the client discards its server projection and requests a fresh snapshot.

Optimistic updates are allowed only for reversible interface actions whose authoritative state is retained until confirmation. Status transitions, schedule mutations, approvals, execution actions, and checkpoints remain visibly pending until acknowledged.

### 4. Rust schemas generate the TypeScript contract

Stable command-center DTOs live in a dedicated Rust contract boundary rather than exposing internal persistence structs. A deterministic generator emits TypeScript types and a client package. CI fails when generated output differs from the Rust source.

The contract separates:

- Jcode-owned identifiers: initiative, milestone, step, schedule reference, command, event, and Jcode run record;
- Orca-owned identifiers: `orcaProjectId`, orchestration run, worker, terminal, gate, host setup, checkout, and worktree;
- client-owned state: active panels, layout dimensions, filters, selections, scroll positions, and drafts.

### 5. The flagship route is a split initiative/execution workspace

The route keeps durable intent visible on the left and selected live execution on the right.

**Durable pane:** outcome, status, current milestone, milestone steps, success criteria, blockers, next actions, child-change references, linked schedules, and checkpoint history.

**Execution pane:** linked Jcode run, normalized Orca graph, workers/sessions, gates/approvals, event timeline, staleness indicator, and safe execution commands.

The browser route does not silently infer that an initiative owns a run. Missing links render an explicit empty/unavailable state and offer only actions the server can safely perform.

### 6. Orca remains authoritative for executable projects and live runtime

Jcode stores stable Orca references and normalized observations. It does not mint a competing executable project identity or accept direct browser writes to Orca-owned state. Runtime commands are submitted to Jcode, policy-checked, forwarded through an Orca adapter, and correlated back to Jcode command/run records.

If Orca is unavailable, durable initiative editing and checkpointing continue. Runtime panels become stale/unavailable with the last observation timestamp and cannot present destructive actions as successful.

The first slice exposes only this closed runtime-command set:

| Command | Owner and preconditions | Idempotency and Orca call | Success evidence | Orca unavailable |
|---|---|---|---|---|
| `start_initiative_run` | Jcode; authorized initiative, canonical `orcaProjectId`, no conflicting active start, server capability present | One Jcode command/result per idempotency key; create a Jcode run record, then request one Orca orchestration run for the selected project/context | Correlated Jcode run ID plus Orca run ID and an accepted/started Orca lifecycle event | Reject before creating an active run; retain a typed unavailable result |
| `retry_linked_run` | Jcode; selected failed/retryable Jcode run, linked Orca project/run, retry policy allows another attempt | One new attempt per idempotency key; request an Orca retry/new orchestration run using the prior run's approved context | New correlated Jcode attempt and Orca run IDs plus accepted/started evidence | Reject without incrementing retry state |
| `cancel_linked_run` | Jcode policy gate; selected nonterminal run, server reports cancel capability, caller authorized for the initiative | Repeated identical cancellation keys return the same command result; request cancellation of the exact Orca run ID | Orca cancellation acceptance followed by terminal cancelled/completed evidence; until then UI remains pending | Reject as unavailable and do not claim cancellation |

Initiative edits and checkpoints remain Jcode-owned commands and do not call Orca. Approval resolution, handoff, schedule mutation, direct gate mutation, worker/terminal control, worktree operations, and arbitrary Orca commands are excluded from this child change. Unknown, unsupported, stale, unauthorized, or capability-mismatched runtime commands are rejected before adapter invocation.

### 7. Security is explicit and browser credentials are scoped

The daemon issues short-lived command-center browser sessions after a trusted local bootstrap or explicit remote pairing. State-changing requests require same-origin checks, CSRF protection, and an idempotency key. Browser sessions contain no provider credentials and cannot invoke arbitrary tools or filesystem paths outside typed commands.

Remote access is disabled by default. Binding beyond loopback requires explicit configuration, authenticated transport, origin allowlisting, and a startup warning identifying the exposed address.

### 8. Desktop compatibility is a contract, not a second implementation

Routes, DTOs, command semantics, and client state boundaries MUST avoid assumptions that require a standalone browser tab. The later desktop milestone may host the SolidStart application as a dedicated `CommandCenter` surface while native session/file/diff surfaces continue using the shared daemon protocol.

This explicitly revises the prior blanket rejection of any WebView-hosted surface. It does not convert the entire desktop into a web shell. Final sandboxing, IPC/HTTP transport, and native focus integration are deferred to the desktop child change.

### 9. `prefix+o` is not part of this vertical slice

The existing terminal Control Room remains unchanged. A later small integration may make the shortcut open/focus the contextual command-center URL once routing, authentication, and focus behavior are stable. This prevents throwaway terminal interaction work.

## Risks / Trade-offs

- **[SolidStart server/runtime integration may not fit a single Rust process directly]** → Treat the build output and SSR adapter as an implementation spike in task 1; the acceptance boundary is one Jcode-managed service and one authority, not a specific bundler trick. If in-process SSR is infeasible, a daemon-supervised private child process is allowed only when it has no independent persistence, listener exposure, or lifecycle authority.
- **[HTTP expands the local attack surface]** → Loopback-only default, short-lived sessions, CSRF/same-origin enforcement, strict CSP, no secrets in the client, and explicit remote exposure configuration.
- **[Event gaps create a misleading live view]** → Sequence validation, visible connection state, resumable replay, snapshot reconciliation, and last-observed timestamps.
- **[Orca schema drift breaks projections]** → Version the adapter contract, represent unknown fields safely, capability-negotiate actions, and test against recorded compatibility fixtures.
- **[The split screen becomes too dense]** → Use resizable panes and progressive detail while keeping outcome, milestone, runtime health, and attention items always visible.
- **[Generated Rust/TypeScript contracts drift]** → Deterministic generation and CI diff checks.
- **[The browser UI diverges from the future desktop]** → Keep route state and command contracts host-neutral; explicitly test embedded-width layouts and keyboard command routing.
- **[Legacy documents continue to conflict]** → Maintain the initiative migration ledger and archive only after every decision is absorbed, retained, or explicitly superseded.

## Migration Plan

1. Add isolated command-center contract types and deterministic TypeScript generation without changing existing clients.
2. Add the loopback web host behind an explicit experimental configuration flag.
3. Build the SolidStart shell and snapshot-driven initiative list/detail routes.
4. Add commands, event sequencing, reconnect behavior, and the split live-execution pane.
5. Add authenticated remote-tunnel acceptance coverage from the Mac control surface.
6. Enable the command center by default only after security, idle-resource, reconnect, and compatibility gates pass.
7. Keep the current TUI and Context Control Room as fallback clients throughout rollout.

Rollback disables the web-host configuration and removes the listener while preserving all initiative, schedule, session, and Orca state. No migration in this child change makes existing persisted records unreadable by older clients.

## Open Questions

None required for authoring. The user explicitly approved SolidStart, daemon hosting, the split initiative/execution layout, snapshot-command-event state flow, browser-first delivery, and later desktop-surface embedding. Implementation-specific dependency versions and bundling details are discoverable during apply and must remain within the authority and security constraints above.
