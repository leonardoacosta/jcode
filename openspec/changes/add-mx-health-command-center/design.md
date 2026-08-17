## Context

MX is an external system whose health includes independent layers. Provider scans can succeed while PostgreSQL-backed request persistence or commitment intelligence is unavailable. Command Center must not collapse those conditions into one green connection dot or duplicate MX health logic. The integration crosses a secret-bearing daemon boundary, a typed Rust projection, a generated browser contract, and a visual SVG surface.

## Goals / Non-Goals

### Goals

- Consume one immutable `mx.health.v1` contract through a bounded, authenticated server-side adapter.
- Preserve MX reason codes, states, timestamps, cached-data semantics, dependencies, and recovery metadata.
- Render a compact topology that communicates causal degradation while remaining semantically complete without SVG or color.
- Fail closed on authentication, transport, version, and schema errors while optionally retaining clearly stale last-known-good data.

### Non-Goals

- Recompute MX health from Jcode-side probes.
- Build a generic service map or observability platform.
- Add MX mutations, alerting, or long-term health history.
- Expose network addresses, DSNs, tokens, account identifiers, or raw upstream errors.

## Architecture

```mermaid
flowchart LR
    MX[MX gateway\nGET /health/v1] -->|Bearer auth, bounded timeout| A[Jcode MX health adapter]
    A --> V[Version and schema validator]
    V --> P[Redacted Command Center projection]
    P --> H[Authenticated Jcode HTTP endpoint]
    H --> U[/mx Solid route]
    U --> S[Inline SVG topology]
    U --> L[Semantic summary and check list]
    U --> D[Keyboard-accessible details]
```

The browser requests only the Jcode projection. The daemon adapter owns the MX URL, token, timeout, response-size limit, validation, and safe error mapping. If the exact MX contract uses different field names from the exploratory draft, the adapter maps those committed fields into the Jcode projection without changing MX semantics.

## Contract Projection

The Jcode projection is additive to existing Command Center contracts and should include only fields justified by the pinned MX schema:

- protocol/schema version and pinned MX contract provenance;
- generated/fetched timestamps and overall state;
- redacted flag;
- check identifier, label, layer/category, state, stable reason code, and safe summary;
- dependency identifiers needed to draw causal edges;
- last attempt, last success, and data age when supplied;
- cached-data availability and explicit stale status;
- safe recovery state/action text when supplied;
- adapter fetch state and last-known-good timestamp.

Unknown fields are ignored only when the committed versioning rules explicitly permit additive compatibility. Unknown values for closed required enums, missing required fields, `redacted != true`, wrong major version, oversized payloads, malformed timestamps, duplicate check IDs, or dangling dependencies fail validation. If the pinned contract declares an enum extensible, the adapter must project its documented unknown-state fallback without treating it as healthy.

## Request and Cache Behavior

- Use a short configurable timeout and response-size cap.
- Coalesce concurrent refreshes so one browser burst does not fan out to MX.
- Use a small in-memory freshness window suitable for an operator dashboard. Do not persist health history in this change.
- On successful validation, replace the last-known-good projection atomically.
- On fetch or validation failure, return a typed adapter state. A last-known-good projection may accompany it only with `stale: true`, the prior observation timestamp, and the current failure category.
- Never convert an MX `degraded`, `down`, `starting`, or `recovering` state to `ok` because cached data exists.

## UI Composition

- Add `MX` to the stable Command Center navigation and route table. When daemon MX configuration is absent, keep the route visible and render a setup-required state without exposing secret or endpoint values.
- Page header contains the overall text state, observation age, and a concise impact statement.
- Desktop topology flows through ordered layer groups. Dependencies determine connector state; Jcode must not invent edges absent from the validated contract or an explicitly documented fixed presentation mapping.
- At narrow widths, the SVG uses a vertical view box or is visually secondary to the ordered semantic list. The page must not require horizontal scrolling.
- Selecting a node opens or focuses an HTML details region with state, reason, timestamps, cached-data semantics, dependencies, and recovery text.
- Loading uses stable geometry or a plain status region. Error states provide retry for the Jcode read only, not an MX mutation.

## Accessibility

- The page has one `h1`, logical headings, a live status region for refresh outcome, and a visible keyboard focus indicator.
- Status uses text and shape/iconography in addition to color. Forced-colors mode preserves boundaries and selected/focus state.
- Interactive SVG nodes are native focusable controls where practical or are paired one-to-one with HTML controls. Connector paths and decorative marks use `aria-hidden="true"`.
- SVG has an accessible name and description, while the semantic list contains the complete data so no user must interpret geometry.
- Details selection and retry work by keyboard. Focus is not moved on passive refresh.
- Animation is optional, subtle, and disabled under `prefers-reduced-motion`.

## Security and Privacy

- Configuration is daemon-side only. The MX token is loaded through Jcode's secret-safe config path and is never serialized into generated frontend configuration.
- Logs contain stable failure categories and check IDs only. Raw MX bodies, authorization headers, URLs with credentials, DSNs, and provider error strings are excluded.
- The Jcode route uses the existing authenticated Command Center session and permission boundary.
- The adapter requires the upstream `redacted` assertion and rejects responses that violate it.
- Tests use synthetic redacted fixtures and scan browser payloads/artifacts for known secret sentinels.

## Alternatives Considered

- **Browser calls MX directly:** rejected because it exposes topology and bearer credentials and complicates CORS/authentication.
- **Derive health from existing `/sources` plus Jcode probes:** rejected because it duplicates semantics and misses persistence/workflow degradation.
- **Static SVG image:** rejected because it cannot express live states, keyboard details, semantic equivalence, or responsive reflow.
- **Generic topology renderer:** rejected as unnecessary scope. This page uses one MX-specific presentation model.
- **Mutation buttons:** rejected until MX publishes explicit authenticated mutation contracts and idempotency/receipt semantics.

## Risks / Trade-offs

- **Contract drift:** pin immutable provenance and fail closed on incompatible versions.
- **Stale data looks current:** label stale state at page and check level with observation age and current fetch failure.
- **SVG accessibility regressions:** require semantic-list parity and keyboard/browser acceptance.
- **Dense mobile layout:** prefer the list/details reading order and a simplified vertical SVG rather than shrinking desktop geometry.
- **Active test-file overlap:** apply must adopt or isolate additions around the existing dirty baseline and review the final diff by path.

## Rollout and Rollback

1. Pin and fixture the committed MX contract.
2. Land adapter and projection behind configuration; when unconfigured, the stable authenticated route remains visible and reports setup required.
3. Add `/mx`, navigation, and fixture acceptance.
4. Configure an isolated managed daemon against a deterministic MX fixture, then a real redacted MX endpoint.
5. Enable the route only after secret scanning, accessibility, responsive, and managed-runtime gates pass.

Rollback removes the navigation exposure and disables MX configuration. Existing Command Center routes and contracts remain compatible because the projection is additive and read-only.

## External Gate

Implementation is blocked until the exact committed MX contract is locally inspectable and its repository SHA, artifact path, schema version, compatibility rules, and SHA-256 digest are recorded in this change's implementation evidence. If the committed contract differs materially from the fields assumed here, update and revalidate this OpenSpec change before coding.
