## 1. Provenance and Conflict Gate

- [ ] 1.1 Recheck Jcode HEAD and dirty paths immediately before apply. Preserve unrelated swarm, TUI, external-signal, and isometric-map changes; stop on new overlap in proposed Command Center paths.
- [ ] 1.2 Verify the pinned MX authority at commit `6f9ac51a419807a3636b17f5e697ae23c37cacff` and the four source digests recorded in `proposal.md`; record the observed result in implementation evidence.
- [ ] 1.3 Add a minimal synthetic redacted `mx.health.v1` fixture set plus provenance manifest and deterministic drift/schema check. Fixtures SHALL include healthy/200, degraded/200, persistence-down/workflows-blocked/503, and invalid variants without copying secrets or provider identities.
- [ ] 1.4 Verify the current MX v1 omission of per-check timestamps, cache state, data age, and recovery metadata; ensure generated DTOs and UI fixtures do not invent them.

## 2. Daemon Configuration, Adapter, and Projection

- [ ] 2.1 Add daemon-only MX base URL and bearer-token configuration using existing secret-safe conventions, plus bounded timeout, response-size cap, refresh window, and stale-cache limit. Configuration display and logs SHALL redact values.
- [ ] 2.2 Implement the MX health client with bearer authentication, bounded body/I/O, and concurrent-refresh coalescing.
- [ ] 2.3 Treat authenticated HTTP 200 and 503 as contract-bearing responses; map 401/403, unexpected status, timeout, and connection failures to distinct safe adapter categories.
- [ ] 2.4 Validate exact v1 semantics: version, redacted assertion, RFC3339 generation time, recognized overall/check statuses, required safe strings, unique IDs, resolvable dependencies, and payload cap. Ignore additive unknown JSON fields but fail closed on unknown status values.
- [ ] 2.5 Add the additive Rust projection and deterministic TypeScript generation for pinned provenance, MX generation time, Jcode fetch time, overall/check fields, dependencies, adapter state, safe failure category, and optional explicitly stale last-known-good projection.
- [ ] 2.6 Implement one atomic in-memory last-known-good cache with stale-limit enforcement. Cached data SHALL never upgrade MX degraded/down/blocked state and SHALL not be returned after expiry.
- [ ] 2.7 Add focused Rust tests for successful 200, authoritative 503/down, upstream unauthorized, unexpected status, timeout, unreachable, oversized, malformed, wrong version, unredacted, unknown status, missing fields, duplicate IDs, dangling dependencies, additive fields, coalescing, cache replacement, eligible stale cache, expired cache, and secret-redacted errors/logs.
- [ ] 2.8 Run focused Rust tests and generated-contract drift checks for Batch 2; expected result is exit 0 before HTTP/UI work begins.

## 3. Authenticated Jcode Read Surface

- [ ] 3.1 Add one authenticated Command Center read endpoint for MX health using existing browser-session and permission middleware before adapter invocation.
- [ ] 3.2 Return stable safe states for unconfigured, live, stale, upstream unauthorized, timeout, unreachable, invalid contract, and unavailable-without-cache, without serializing MX URL/token configuration or raw errors.
- [ ] 3.3 Prove unauthenticated, expired-session, and forbidden requests follow existing Command Center behavior and do not invoke the MX adapter.
- [ ] 3.4 Prove browser payloads and captured logs contain no bearer token, authorization header, endpoint credential, DSN, provider identity, raw body, raw error, or secret sentinel.
- [ ] 3.5 Run focused endpoint/security tests for Batch 3; expected result is exit 0 before frontend integration begins.

## 4. `/mx` Command Center Experience

- [ ] 4.1 Add stable `/mx` routing and one `MX health` desktop/mobile navigation item without changing existing route matching or behavior.
- [ ] 4.2 Extend the browser transport with the authenticated Jcode MX-health read only. Browser code SHALL never call MX directly.
- [ ] 4.3 Build the page header, textual overall/adapter states, MX generation age, Jcode fetch freshness, deterministic impact copy, legend, loading/setup/error/stale regions, and retry-read control using existing Command Center surfaces and CSS tokens.
- [ ] 4.4 Build a complete semantic HTML layer/check list and details region exposing exact check ID/label, layer, status, reason code, summary, and dependencies. Do not show absent v1 metadata.
- [ ] 4.5 Build the inline SVG topology from validated checks and `depends_on` only. Provide accessible name/description, hide decorative connectors, and use text plus shape/icon/color for state.
- [ ] 4.6 Provide one-to-one keyboard selection between topology and HTML controls with visible focus; passive refresh SHALL preserve focus, selected check, details visibility, and unrelated scroll state.
- [ ] 4.7 Implement the authoritative down/503 presentation that keeps healthy providers visually independent from down persistence and blocked workflows.
- [ ] 4.8 Support 320px vertical/list-first layout, tablet, desktop horizontal topology, 200% zoom, forced colors, and reduced motion with no clipped controls or page-level horizontal overflow.
- [ ] 4.9 Ensure no restart, reconnect, credential-refresh, database-repair, or other MX mutation control or request exists.
- [ ] 4.10 Run Solid format, lint, typecheck, component tests, and production build for Batch 4; expected result is exit 0 before managed-runtime acceptance.

## 5. Frontend and Accessibility Verification

- [ ] 5.1 Add component tests for unconfigured, loading, healthy, degraded, authoritative down/503, stale-with-current-error, unauthorized, unreachable, timeout, invalid-contract, and retry states.
- [ ] 5.2 Add tests for status text/non-color distinction, semantic/SVG parity, dependency details, keyboard traversal/selection, visible focus, live-region announcements, and passive refresh stability.
- [ ] 5.3 Add responsive tests at 320, 768, and desktop widths, plus 200% zoom, forced colors, and reduced motion; assert no page-level horizontal overflow.
- [ ] 5.4 Add source/browser assertions that `/mx` exposes no mutation controls and emits no MX mutation request.
- [ ] 5.5 Run focused Vitest and Playwright fixture suites for Batch 5; expected result is exit 0.

## 6. Managed Runtime and Operations

- [ ] 6.1 Extend the isolated managed-daemon acceptance launcher with a deterministic authenticated MX fixture serving exact pinned-schema responses and controllable response sequences.
- [ ] 6.2 Exercise public authenticated `/mx` acceptance for healthy/200, degraded/200, down/503, stale sequence, upstream unauthorized, unreachable, invalid contract, unconfigured, expired browser session, and forbidden session.
- [ ] 6.3 Verify existing inbox, ambient, find, initiative, authentication, and event-stream workflows remain compatible when MX is unconfigured or unavailable.
- [ ] 6.4 Run a credential-gated real-endpoint check against pinned-compatible MX. Verify `redacted: true` and scan browser payloads, logs, screenshots, traces, and retained artifacts for forbidden data and configured secret sentinels.
- [ ] 6.5 Document daemon configuration names, secret handling, timeout/cache behavior, MX-vs-adapter state meanings, provenance update procedure, troubleshooting, and rollback/disable behavior.
- [ ] 6.6 Run `scripts/test-command-center.sh` or the repository's current managed acceptance entry point with the MX matrix; expected result is exit 0 for deterministic projects, with any credential-gated real endpoint reported separately and truthfully.

## 7. Final Gates and Review

- [ ] 7.1 Run `cargo fmt --all -- --check`, focused affected Rust tests, and the relevant broader Command Center regression suite; expected result is exit 0 or an explicitly attributed pre-existing blocker.
- [ ] 7.2 Run Command Center `format:check`, `lint`, `typecheck`, `test`, `build`, and `test:e2e`; expected result is exit 0 with no new failures.
- [ ] 7.3 Run deterministic generated-contract/provenance checks, `openspec validate add-mx-health-command-center --strict --no-interactive`, and `git diff --check`; expected result is exit 0.
- [ ] 7.4 Perform independent semantic, security, accessibility, and scope review mapping every requirement to scenarios, tasks, and checks, including authoritative HTTP 503 handling and absent-v1-field discipline.
- [ ] 7.5 Record final artifact and implementation digests. Rerun validation/review after any authoritative artifact mutation.
- [ ] 7.6 Commit only owned Jcode/OpenSpec changes with evidence references; preserve unrelated dirty work.
