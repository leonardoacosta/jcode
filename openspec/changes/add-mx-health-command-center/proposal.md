## Why

Command Center has no operator-facing view of MX readiness. MX can keep provider sources serving while persistence-backed workflows are blocked, and its committed authenticated `GET /health/v1` contract now exposes that distinction safely. Jcode needs a narrow daemon-side adapter and a dedicated health route that preserve MX as the authority while making causal degradation understandable and accessible.

## What Changes

- Add a daemon-owned MX health adapter that calls the committed authenticated `mx.health.v1` endpoint without exposing the MX bearer token or endpoint to browser code.
- Add a strict additive Jcode projection for the committed fields: version, generation time, overall state, redaction assertion, checks, stable reason codes, safe summaries, layers, and dependency IDs.
- Add an authenticated `/mx` Command Center route with a responsive inline SVG topology plus equivalent semantic HTML list and details content.
- Represent configured, unconfigured, loading, healthy, degraded, down, unauthorized, unreachable, timeout, invalid-contract, and stale-last-known-good adapter states without relying on color alone.
- Keep the feature read-only. Retry repeats only the bounded Jcode read and never mutates MX.
- Add contract drift, adapter, security, accessibility, responsive-layout, and managed-runtime acceptance coverage.

## Authority and Provenance

This OpenSpec change is the single durable authority for the Jcode work. Session todos and the generated visual mock are supporting evidence only.

The consumed MX authority is:

- Repository: `https://github.com/leonardoacosta/mx.git`
- Commit: `6f9ac51a419807a3636b17f5e697ae23c37cacff`
- Version: `mx.health.v1`
- Implementation: `cmd/mx-gateway/health_v1.go`, SHA-256 `35da7eae62b10732beeb27c828b3c9418d93482d3dd78f5a0edda296cf0d82c4`
- Contract specification: `openspec/specs/gateway-health/spec.md`, SHA-256 `ca17e036c7ba5becedf4f6463779d1f78116c90870fb353d51dbf9264c1e4f1e`
- OpenAPI document: `docs/api/mx-gateway.openapi.json`, SHA-256 `a8af2d00c7e62c24dd8b303329ed483198b875b67b3221f461866519cab258f6`
- Contract tests: `cmd/mx-gateway/health_v1_test.go`, SHA-256 `6788bf1a60a964e2ec0a7c1db5c911d5270b85c9220924848f98683693074d23`

The committed response contains `version`, `generated_at`, `overall`, `redacted`, and `checks`; each check contains `id`, `layer`, `status`, `reason_code`, `summary`, and optional `depends_on`. The current contract does not supply per-check timestamps, cached-data flags, data age, or recovery metadata. Jcode SHALL NOT invent those fields or claim those capabilities.

## Capabilities

### New Capabilities

- `command-center-mx-health`: authenticated MX health ingestion and the `/mx` operator experience.

### Modified Capabilities

- None. Existing Command Center routes, protocols, and initiative behavior remain compatible.

## Decisions

- The Jcode daemon calls MX. The browser calls only Jcode.
- MX owns health semantics. Jcode validates, safely projects, orders, and displays them without severity upgrades.
- HTTP 200 and 503 from authenticated MX health are both valid contract-bearing responses. Transport/auth/schema failures are adapter failures.
- Last-known-good data may accompany a current adapter failure only when explicitly labeled stale with Jcode fetch time and MX generation time.
- The topology uses inline SVG, but semantic HTML remains the complete authoritative reading and interaction path.
- The route remains visible when MX is unconfigured and shows setup-required guidance without exposing configuration values.
- Existing Command Center tokens and visual language are reused: `--teal` for healthy, `--pink` for degraded/down attention, and text/shape labels for every state.
- No MX mutation controls are included.

## Uncertainty Disposition

| Uncertainty | Class | Disposition |
| --- | --- | --- |
| MX contract existence and bytes | Discoverable fact | Verified at the repository, commit, paths, and digests recorded above. |
| Missing timestamps/cache/recovery fields | Discoverable fact | The committed v1 contract lacks them. Project only Jcode fetch freshness and MX `generated_at`; do not invent per-check metadata. |
| Jcode-to-MX address and token source | Safe reversible default | Add daemon-only configuration using existing secret-safe environment/config conventions. Never serialize it to browser state. |
| Treatment of MX HTTP 503 | Safe reversible default | Parse as authoritative `overall: down` when the body validates; do not turn it into an adapter-unreachable error. |
| Refresh/caching values | Safe reversible default | Use bounded configurable values with conservative defaults documented during apply; coalesce concurrent refreshes. |
| Narrow topology | Safe reversible default | Use a vertical presentation or make the semantic list primary at narrow widths; prohibit page-level horizontal scrolling. |
| UI mutation capability | Safe reversible default | Read-only. Reject restart, reconnect, credential refresh, or database repair controls. |

## Scope

### In scope

- Jcode daemon configuration, MX client, schema validation, typed projection, authenticated read endpoint, generated TypeScript contract, `/mx` route, navigation, SVG/list/details presentation, fixtures, tests, documentation, and managed-runtime verification.
- Exact current MX statuses and reason codes, including `ok`, `degraded`, `down`, and `blocked`, plus safe handling of unknown incompatible values.
- Adapter freshness and stale-last-known-good handling separate from MX overall/check health.

### Excluded

- Any change to the MX repository or `mx.health.v1` semantics.
- Fabricated per-check timestamps, cache state, recovery actions, or historical health.
- MX process control, credential refresh, database repair, alerting, or notifications.
- A generic topology or observability framework.
- Unrelated initiative, Orca, swarm, external-signal, and isometric-map changes.

## Conflicts and Ownership

- Jcode is currently at `27bb37f3daf778624e7dd0b4b6cdf3a1ab1acea5` with unrelated active changes in swarm persistence, TUI gallery, and isometric-map surfaces. They are out of scope and must be preserved.
- `crates/jcode-app-core/src/server/swarm.rs` and `swarm_persistence_tests.rs` are dirty but are not required by this feature.
- The existing `add-solidstart-command-center-vertical-slice` change owns the base shell and managed-host machinery. This change extends its stable seams without altering its remaining checklist.
- At authoring time, proposed Command Center web paths are not dirty. Apply must recheck immediately before editing.

## Done Means

- An authenticated operator can open `/mx`, understand MX overall state and impact, inspect every projected check and dependency, and distinguish authoritative MX health from adapter freshness or failure.
- HTTP 503 with a valid redacted `mx.health.v1` body renders authoritative MX down state.
- The page is fully understandable and operable by keyboard and assistive technology at 320 CSS pixels, 200% zoom, forced colors, and reduced motion.
- MX credentials, endpoint configuration, authorization headers, raw errors, DSNs, provider identities, and unredacted payloads never enter browser payloads, logs, screenshots, traces, or retained test artifacts.
- Contract provenance is pinned, drift checks are deterministic, focused and full Command Center gates pass, managed-daemon browser acceptance covers the required state matrix, and strict OpenSpec validation passes.

## Expected Touched Surfaces

- `crates/jcode-command-center/src/lib.rs` and focused tests/fixtures
- `crates/jcode-app-core/src/command_center.rs` or a narrowly owned MX adapter module
- `apps/command-center/src/app.tsx`
- `apps/command-center/src/components/` and `components/shell/navigation.tsx`
- `apps/command-center/src/transport/client.ts`
- `apps/command-center/src/generated/command-center-contract.ts` through deterministic generation
- `apps/command-center/src/styles.css`
- Command Center unit, component, Playwright, security, and managed-runtime fixtures/scripts
- Command Center configuration and operator documentation

## Verification Summary

- Focused Rust adapter/projection/security tests.
- Generated-contract drift check.
- Command Center format, lint, typecheck, component tests, build, and Playwright.
- Public managed-daemon `/mx` acceptance for healthy, degraded, authoritative down/503, unauthorized upstream, unreachable, invalid-contract, stale last-known-good, unconfigured, and expired browser session.
- Accessibility, responsive, no-horizontal-overflow, no-mutation-control, and secret-sentinel checks.
- Strict OpenSpec validation and `git diff --check`.
