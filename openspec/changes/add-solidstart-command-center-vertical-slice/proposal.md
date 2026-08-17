## Why

Jcode already has durable initiatives, schedules, sessions, approvals, and workflow primitives, but they are fragmented across tools, files, TUI surfaces, and architecture documents. A browser-first command center is needed now to prove one authoritative, highly interactive supervision loop before more behavior accumulates in the temporary terminal Control Room or in disconnected UI experiments.

## What Changes

- Add a Jcode-daemon-hosted SolidStart command-center application as the canonical browser UI.
- Add a versioned snapshot, command, and ordered event contract owned by the Jcode daemon; the web application owns only layout, focus, filters, selections, and transient input state.
- Add an initiative vertical slice with routes for initiative selection, initiative detail, and a selected run.
- Present durable initiative outcome, milestone, child-work references, schedules, blockers, and next actions beside live execution state.
- Normalize linked Orca project/run/worker/terminal/gate events into the Jcode projection without duplicating Orca's executable-project or live-runtime authority.
- Support command idempotency, reconnect reconciliation, visible stale/unavailable states, and inline failure recovery.
- Keep the terminal Context Control Room as a lightweight inspector. This change does not expand it into a second command-center implementation.
- Structure routes and shared client contracts so the same command-center application can later be embedded as a Jcode Desktop `CommandCenter` surface without reimplementing domain behavior.
- Record the approved `Jcode Command Center` durable initiative as the umbrella roadmap; this change delivers its first milestone only.

## Capabilities

### New Capabilities

- `command-center-protocol`: Versioned snapshots, idempotent commands, resumable ordered events, authentication, and authority boundaries for command-center clients.
- `command-center-web`: Daemon-hosted SolidStart routes and the split initiative/live-execution vertical slice, including loading, stale, unavailable, failure, and reconnect behavior.

### Modified Capabilities

<!-- None. The existing terminal Context Control Room remains intentionally lightweight and unchanged in this child change. -->

## Preconditions

- base-commit: jcode@a67b5fc85da29d4d81e35a3f425bd283dd860d38
- Implementation runs in the canonical Jcode source repository on `dev` and preserves the recorded unrelated dirty baseline.
- The existing initiative tool/storage, ambient scheduler, daemon socket clients, permission model, and terminal Context Control Room remain available as compatibility baselines.
- Orca integration is exercised through a compatible managed runtime or versioned deterministic fixture; absence of a safe Orca contract blocks runtime-command enablement but not durable initiative work.
- Existing architecture documents remain readable source references until the umbrella initiative migration ledger verifies their disposition.

## Decisions

- SolidStart is the canonical interactive browser client; HTMX and an expanded terminal overlay are rejected for the flagship live workspace.
- The Jcode daemon owns domain behavior, authentication, browser hosting, snapshots, commands, and ordered events; no independently authoritative Node service or frontend database is allowed.
- The first screen is a split initiative/live-execution workspace.
- Orca owns executable project identity and live runtime identity; Jcode stores references, policy decisions, correlations, evidence, and outcomes.
- Browser delivery precedes desktop integration, while routes and client contracts remain host-neutral for a later desktop `CommandCenter` surface.

## Done Means

- An authenticated user can open an initiative route, see durable intent beside linked execution, update or checkpoint the initiative, observe ordered live events, recover from disconnect, and resume the initiative later.
- The same workflow explicitly degrades when Orca is unavailable without losing durable initiative functionality or reporting unsafe runtime actions as successful.
- Existing clients remain compatible when the experimental web feature is disabled.
- Every required deterministic, security, contract, browser, runtime, and strict OpenSpec gate passes with recorded expected results.

## Testing

- Run focused Rust contract/domain/security tests and deterministic Rust-to-TypeScript generation checks; expected result is exit 0 with no generated diff.
- Run Solid formatting, lint, typecheck, component, reconnect, event-order, and accessibility tests; expected result is exit 0 with every required interface state covered.
- Run Playwright against an isolated managed Jcode and Orca topology, then repeat with Orca unavailable; expected result is exit 0 with authoritative reconciliation and explicit degraded runtime behavior.
- Run `openspec validate add-solidstart-command-center-vertical-slice --strict --no-interactive` and `git diff --check`; expected result is exit 0 for both.

## Impact

- **Jcode daemon:** gains authenticated loopback web serving, SolidStart asset/SSR integration, command-center query/command endpoints, and a resumable event stream.
- **Domain contracts:** initiative, schedule, run, approval, session, and normalized Orca projection DTOs become explicitly versioned and exportable to TypeScript.
- **Frontend:** adds a SolidStart workspace and generated typed client package; no independent domain database or Node-owned server truth is introduced.
- **Desktop:** no desktop implementation in this change, but shared routes and client contracts must remain host-neutral for later surface embedding.
- **Operations/security:** loopback-only by default, explicit authenticated remote access, CSRF protection, no provider credentials in browser state, and visible degraded behavior when Orca is unavailable.
- **Existing documentation:** `docs/WORKFLOW_AUTOMATION_ROADMAP.md`, `docs/CONTEXT_ARCHITECTURE.md`, `docs/DESKTOP_SUPERAPP_WORKSPACE.md`, `docs/DESKTOP_APP_ARCHITECTURE.md`, and `docs/MAC_HOMELAB_SSH_TOPOLOGY.md` remain source references until the umbrella initiative migration ledger explicitly absorbs or retains each decision.
