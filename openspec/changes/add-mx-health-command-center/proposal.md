## Why

Command Center has no operator-facing view of MX readiness, so provider ingestion can remain healthy while persistence or downstream intelligence silently fails. MX now supplies a versioned, authenticated, redacted `GET /health/v1` projection intended for this integration. Jcode needs a narrow adapter and a dedicated health route that preserve MX as the health authority while presenting degradation clearly and accessibly.

## What Changes

- Add a daemon-owned MX health adapter that fetches and validates the committed `mx.health.v1` response without exposing the MX bearer token to browser code.
- Add a typed Jcode projection for MX overall state, checks, dependencies, timestamps, cached-data semantics, recovery metadata, and contract provenance.
- Add an authenticated `/mx` Command Center route with an accessible responsive SVG topology and an equivalent semantic list/details view.
- Represent loading, healthy, degraded, down, starting, recovering, stale, unauthorized, contract-invalid, and unreachable states without relying on color alone.
- Keep the page read-only. It reports MX state and recovery metadata but does not invent restart, reconnect, credential, or database mutation controls.
- Add contract, adapter, component, accessibility, responsive-layout, security, and managed-runtime acceptance coverage.

## Capabilities

### New Capabilities

- `command-center-mx-health`: authenticated MX health ingestion and the `/mx` operator experience.

### Modified Capabilities

<!-- None. The existing command-center protocol and web vertical slice remain compatible. -->

## Preconditions and Dependencies

- The authoritative MX repository must expose a committed redacted `GET /health/v1` contract with media/schema version `mx.health.v1` before implementation begins.
- Apply MUST record the MX repository identity, commit SHA, contract artifact path or generated OpenAPI location, and SHA-256 digest of the exact consumed contract bytes. The current read-only MX checkout at `/home/nyaptor/dev/personal/mesh` did not yet expose `/health/v1` during authoring, so user-reported contract commitment is an external freshness gate rather than locally verified evidence.
- The Command Center SolidStart application, daemon-hosted authenticated HTTP boundary, generated TypeScript contract workflow, and isolated managed-daemon acceptance launcher remain available.
- MX endpoint and bearer-token configuration must be daemon-only and use the existing Jcode secret/config safety rules.

## Decisions

- Jcode consumes MX health through a server-side adapter. Browser code never calls MX directly and never receives MX credentials.
- MX owns raw check semantics. Jcode owns only validation, safe projection, display ordering, accessibility text, and UI-local selection state.
- `/mx` uses inline SVG for the topology, backed by normal HTML headings, summaries, lists, controls, and details. The SVG is not the only information channel.
- Overall state and impact copy are derived deterministically from the validated contract. Jcode does not reinterpret a lower-severity MX state as healthy.
- Last-known-good data may remain visible only when explicitly marked stale, timestamped, and separated from the current fetch error.
- The first release is read-only. Recovery actions are explanatory metadata or links only when the MX contract supplies safe redacted values.
- `/mx` remains a stable authenticated navigation route when MX is unconfigured; it shows an explicit setup-required state rather than disappearing or exposing configuration.

## Uncertainty Disposition

| Uncertainty | Class | Disposition |
| --- | --- | --- |
| Exact committed MX contract SHA and bytes | Later evidence-dependent action | Terminal pre-implementation gate. Record repository, SHA, path, digest, and schema fixture before writing adapter code. Reject guessing from the earlier exploratory draft. |
| Jcode-to-MX network address and token source | Discoverable fact | Resolve from daemon runtime configuration and secret-safe deployment conventions during apply. Reject browser environment variables and frontend token storage. |
| Topology layout at narrow widths | Safe reversible default | Desktop shows a left-to-right SVG; narrow layouts switch to a vertical SVG/list arrangement with no horizontal page scrolling. Reject fixed 16:9-only rendering. |
| Whether the page mutates MX | Safe reversible default | Read-only in this change. Reject speculative restart/reconnect controls until MX publishes authenticated mutation contracts. |
| SVG interaction model | Safe reversible default | Every selectable node is keyboard reachable and mirrors an HTML details control; non-interactive connectors are hidden from assistive technology. Reject pointer-only SVG interaction. |
| Unconfigured route exposure | Safe reversible default | Keep the authenticated `/mx` route and navigation entry visible with a setup-required state. Reject hiding the route because hidden configuration failures are harder to diagnose and make deep links unstable. |

## Scope and Exclusions

### In scope

- Jcode daemon adapter, configuration, typed DTO/projection, authenticated route support, navigation entry, `/mx` UI, SVG/list/details presentation, tests, documentation, and deployment verification.
- MX layers represented from contract checks, including process, source/provider, broker/auth, dependency, persistence, workflow/intelligence, and freshness categories when present.
- Explicit stale and partial-data handling.

### Excluded

- Changes to the MX repository or `/health/v1` semantics.
- MX process control, database repair, credential refresh, or provider mutation actions.
- Historical health storage, alerting, notifications, or Grafana ingestion.
- A generic cross-provider topology framework.
- Changes to unrelated Command Center initiative, Orca, swarm, external-signal, or isometric-map work.

## Conflicts and Ownership

- `apps/command-center/tests/command-center.test.tsx` is already modified by another active lane. Apply must preserve that baseline and coordinate test additions without overwriting unrelated changes.
- The active `add-solidstart-command-center-vertical-slice` change owns the base shell and managed-host acceptance path. This change extends its stable route and adapter seams but does not close its remaining Orca tasks.
- Current swarm, TUI gallery, external-signal, and isometric-topology edits are unrelated and out of scope.

## Done Means

- An authenticated operator can open `/mx`, understand MX overall health and impact, inspect every projected check, and distinguish live, stale, unavailable, and invalid-contract data.
- The same information is available to keyboard and assistive-technology users and remains usable at 320 CSS pixels, reduced motion, forced colors, and 200% zoom.
- MX credentials and unredacted provider/database details never enter browser payloads, logs, screenshots, or test artifacts.
- Contract provenance is pinned, validation fails closed, focused and full Command Center gates pass, managed-daemon acceptance exercises healthy and degraded fixtures, and strict OpenSpec validation passes.

## Impact

- **Jcode daemon/app core:** new read-only MX client/configuration and normalized health projection.
- **Command Center web:** new `/mx` route, navigation item, SVG topology, semantic fallback/details UI, and state-specific copy.
- **Generated contracts:** additive MX health DTOs with deterministic Rust-to-TypeScript generation if the daemon projection uses the existing generator.
- **Security/operations:** one daemon-held endpoint/token pair, bounded request timeout, no browser-side MX access, redacted diagnostics, and explicit deployment configuration.
- **External dependency:** committed MX `mx.health.v1` contract, verified by immutable SHA and digest at apply time.

## Testing

- Strictly validate this change and verify no artifact drift.
- Run focused Rust adapter/projection/security tests and deterministic generated-contract checks.
- Run Command Center format, lint, typecheck, Vitest, and Playwright suites.
- Run accessibility checks for names, roles, keyboard traversal, focus visibility, status announcements, non-color status text, reduced motion, and semantic equivalence between SVG and list/details content.
- Run responsive checks at 320, 768, and desktop widths, plus 200% zoom, with no clipped controls or horizontal page scroll.
- Run isolated managed-daemon acceptance with healthy, degraded/stale, unauthorized, unreachable, and contract-invalid MX responses.
