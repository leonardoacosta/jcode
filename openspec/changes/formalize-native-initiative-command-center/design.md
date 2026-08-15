## Context

Jcode's initiative tool and app-core goal persistence are durable authorities. The Command Center web host and SolidStart application project that state, while ambient schedules, Jcode runs, and Orca remain linked evidence or external runtime authorities. The current implementation is sound but the boundary is not discoverable enough.

## Goals

- Make authority and projection boundaries explicit.
- Preserve wire compatibility and existing TUI/tool behavior.
- Give future contributors a single extension path.
- Keep degraded and unavailable states honest.

## Non-Goals

- Rebuild the web UI.
- Add a frontend database.
- Replace the TUI.
- Move Orca or scheduling authority into initiatives.
- Add Installfest-specific features.

## Decisions

### D1: Document the authority graph

Create one authoritative guide linking initiative commands, app-core persistence, daemon/API contracts, TUI projections, the web UI, ambient evidence, Jcode runs, and Orca observations. Label each node as authority, projection, or external dependency.

### D2: Use one lifecycle vocabulary

Define status, milestone, step, checkpoint, revision, idempotency, degraded, unavailable, and linked-run terms once. TUI, API, and browser documentation must use the same definitions.

### D3: Extend through native contracts

New browser views must consume generated DTOs and issue public commands through the daemon. New TUI views must use the same goal repository and lifecycle semantics. No consumer may read or write persistence files directly.

### D4: Preserve fail-closed boundaries

Unsupported Orca mutations, missing browser capabilities, replay gaps, stale revisions, and unavailable schedulers remain explicit states. No UI may invent success from missing evidence.

## Verification

- Documentation links and capability matrix validate.
- Contract generation remains drift-free.
- Focused app-core, command-center, TUI, and browser tests pass.
- A clean isolated daemon can list and update an initiative through the supported path.
