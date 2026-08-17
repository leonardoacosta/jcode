## Context

MX is an external health authority with independent source, persistence, and workflow layers. The committed `mx.health.v1` response intentionally permits provider checks to remain `ok` while persistence is `down` and dependent workflows are `blocked`. Command Center must preserve that causal picture across a secret-bearing daemon boundary, a typed Rust projection, a generated browser contract, and an accessible visual surface.

The authoritative MX input is pinned to commit `6f9ac51a419807a3636b17f5e697ae23c37cacff`. The implementation response type is:

```text
{
  version: string,
  generated_at: RFC3339 timestamp,
  overall: string,
  redacted: boolean,
  checks: [{
    id: string,
    layer: string,
    status: string,
    reason_code: string,
    summary: string,
    depends_on?: string[]
  }]
}
```

Current produced overall values are `ok`, `degraded`, and `down`. Current check values are `ok`, `degraded`, `down`, and `blocked`. The contract does not provide per-check timestamps, cached-data state, data age, or recovery metadata.

## Goals / Non-Goals

### Goals

- Consume the pinned `mx.health.v1` contract through bounded authenticated server-side I/O.
- Preserve MX overall/check status, reason code, safe summary, layer, generation time, and dependencies without severity upgrades.
- Distinguish upstream MX health from Jcode adapter freshness and failures.
- Render a compact causal topology that remains complete without SVG, pointer input, motion, or color.
- Fail closed on auth, transport, redaction, version, schema, and safety errors.

### Non-Goals

- Recompute MX health from Jcode probes.
- Invent fields absent from `mx.health.v1`.
- Build a generic service map or observability platform.
- Add MX mutations, alerting, notifications, or health history.
- Expose endpoint values, tokens, DSNs, provider identities, or raw upstream errors.

## Architecture

```mermaid
flowchart LR
    MX[MX gateway\nGET /health/v1] -->|Bearer auth, bounded timeout| A[Jcode MX adapter]
    A --> V[Version, redaction, and schema validator]
    V --> C[Atomic in-memory last-known-good cache]
    V --> P[Redacted Command Center projection]
    C --> P
    P --> H[Authenticated Jcode read endpoint]
    H --> U[/mx Solid route]
    U --> S[Inline SVG topology]
    U --> L[Semantic layer and check list]
    U --> D[Keyboard-accessible details]
```

The browser requests only the Jcode projection. The adapter owns MX URL/token configuration, timeout, response-size limit, fetch coalescing, response parsing, validation, and safe error mapping.

## Contract Pinning and Drift

Apply copies a minimal redacted fixture and provenance manifest into Jcode test data. The manifest records repository URL, commit, version, source paths, and SHA-256 digests from the proposal. A deterministic check verifies the fixture shape against the pinned schema/OpenAPI artifact without requiring runtime access to the sibling checkout.

Any material upstream change requires a new pinned commit/digest and OpenSpec review. Additive unknown JSON fields may be ignored because Go's default decoder and the documented version remain compatible, but required known fields and their safe semantics must still validate. Unknown `overall` or check `status` values are incompatible in this Jcode release and fail closed because the current MX contract does not declare extensible enum behavior.

## Adapter and Projection

### Upstream response handling

- Authenticate with a daemon-held bearer token.
- Treat authenticated HTTP 200 and 503 as contract-bearing responses and validate both bodies.
- Treat 401/403 as upstream authentication failure.
- Treat other HTTP statuses, timeout, DNS/connect failure, body-size overflow, malformed JSON, or invalid schema as typed adapter failures.
- Require `version == "mx.health.v1"` and `redacted == true`.
- Require a valid UTC/RFC3339 `generated_at`, recognized `overall`, nonempty unique check IDs, recognized check statuses, nonempty layer/reason/summary values, and dependencies that reference existing IDs.
- Preserve check ordering from MX unless the presentation applies a documented stable layer order without changing data semantics.

### Jcode projection

The additive projection contains:

- MX contract version and pinned provenance identifier;
- MX `generated_at` and Jcode `fetched_at`;
- MX overall status and redaction confirmation;
- every check's ID, layer, status, reason code, safe summary, and dependency IDs;
- adapter state: `live`, `stale`, `unconfigured`, `unauthorized`, `unreachable`, `timeout`, or `invalid_contract`;
- current safe failure category when adapter state is not live;
- optional last-known-good projection with its fetch timestamp and computed age only when explicitly stale.

Jcode-generated age is adapter freshness metadata, not an MX check attribute. Cached data never upgrades MX `degraded`, `down`, or `blocked` state.

## Configuration and Cache Behavior

- Add daemon-only MX base URL and bearer token configuration plus bounded timeout, response-size cap, refresh window, and stale-cache limit.
- Use existing secret-safe environment/config conventions and redact configuration summaries.
- Coalesce concurrent refreshes so a browser burst produces at most one active upstream request.
- Keep only one in-memory last-known-good value. Do not persist history.
- Replace it atomically only after successful validation.
- When a fetch fails, return stale data only if it is within the configured stale limit, and label both the stale observation and the current adapter failure.
- When unconfigured or no valid cache exists, return no fabricated checks.

## Jcode HTTP Boundary

Add one authenticated read endpoint under the existing Command Center router. Browser authentication and permission checks run before adapter invocation. Its response never contains MX URL, token source, authorization headers, raw body, or raw errors. Retry invokes this same bounded read endpoint. No endpoint for MX mutation is added.

## UI Composition

- Add one stable `MX health` navigation item and `/mx` route using the existing shell.
- Reuse `.page`, `.page-bar`, `.surface`, `.status`, `.state-card`, `.side-link`, and `.mobile-link` conventions.
- Use existing palette tokens. `--teal` communicates `ok`; `--pink` communicates degraded/down/blocked attention; `--faint` supports unavailable/secondary state. Text and shape/iconography always accompany color.
- Page header shows overall text status, MX generation age, adapter freshness, and concise deterministic impact copy.
- The desktop SVG uses a horizontal layer map. Narrow layouts use a vertical map or make the ordered HTML list primary. Neither creates page-level horizontal scroll.
- Dependencies alone determine connector edges. Jcode does not invent links absent from `depends_on`.
- Selecting a node or equivalent list control updates one HTML details region containing the exact committed fields and adapter context.
- Valid MX HTTP 503 renders the MX down topology, not a generic fetch error.
- Unconfigured, unauthorized, unreachable, timeout, invalid-contract, stale, and loading states use the current Command Center state-card/surface language.

## Accessibility

- One `h1`, logical region headings, an accessible SVG name/description, a complete semantic list, and a concise live refresh region.
- Interactive checks are native HTML controls mirrored visually in SVG, or SVG controls with equivalent keyboard behavior and one-to-one HTML controls.
- Connectors and decorative marks are hidden from assistive technology.
- Selection and retry are keyboard operable with visible focus. Passive refresh does not move focus, reset selection, or collapse details.
- Forced-colors mode preserves boundaries, status, focus, and selection.
- Nonessential animation is absent or disabled under `prefers-reduced-motion`.

## Security and Privacy

- Browser code never calls MX or receives its endpoint/token configuration.
- Logs contain stable adapter categories and safe check IDs only.
- Raw upstream bodies, authorization headers, configured URLs containing credentials, DSNs, provider identities, account details, and raw provider errors are forbidden.
- Upstream `redacted: true` is mandatory.
- Fixtures contain only synthetic safe values and tests scan browser payloads, logs, screenshots, traces, and retained artifacts for secret sentinels.

## Compatibility and Conflicts

- Existing `/inbox`, `/ambient`, `/find`, and initiative routes remain unchanged.
- The projection is additive to generated Command Center types.
- The feature reuses the daemon-hosted lifecycle and does not create a new listener, database, or workflow authority.
- Dirty swarm and isometric-map work is unrelated and must remain untouched.
- Apply rechecks proposed paths for new overlap before each batch.

## Alternatives Rejected

- **Browser calls MX directly:** exposes credentials/topology and creates a second auth/CORS boundary.
- **Infer from `/sources`:** misses persistence/workflow state and duplicates MX semantics.
- **Treat all non-2xx as transport failure:** incorrectly hides the authoritative 503/down contract.
- **Static image only:** cannot support live state, keyboard selection, semantic parity, or responsive reflow.
- **Generic topology framework:** unnecessary scope.
- **Recovery buttons:** no approved MX mutation contract exists.

## Rollout and Rollback

1. Land pinned fixtures, drift checks, adapter, and projection behind daemon-only configuration.
2. Add the authenticated read endpoint and safe state matrix.
3. Add `/mx`, navigation, semantic UI, and SVG topology.
4. Pass deterministic fixture acceptance through the isolated managed daemon.
5. Exercise the real redacted MX endpoint without retaining secrets.

Rollback disables MX configuration and removes navigation exposure while preserving all existing Command Center behavior. The route may continue to show setup-required during staged rollout.

## Verification Strategy

- Rust tests cover validation, HTTP 200/503 contract responses, auth/transport failures, coalescing, stale caching, redaction, and secret safety.
- Contract generation checks ensure additive deterministic TypeScript output.
- Solid tests cover every UI state, semantic/SVG parity, keyboard selection, retry, passive refresh stability, and no mutation controls.
- Playwright covers 320/768/desktop widths, 200% zoom, forced colors, reduced motion, no horizontal overflow, session auth failures, and public `/mx` state behavior.
- The isolated managed-daemon launcher serves exact-schema healthy, degraded, down/503, unauthorized, unreachable, invalid, and stale sequences.
- A real endpoint gate verifies only the redacted contract shape and browser secret absence.
