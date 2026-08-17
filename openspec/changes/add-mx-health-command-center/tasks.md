## 1. Contract Provenance and Baseline

- [ ] 1.1 Record the Jcode base SHA, dirty baseline, active overlapping lanes, and owned paths before implementation. Preserve unrelated edits, especially the existing modification to `apps/command-center/tests/command-center.test.tsx`.
- [ ] 1.2 Locate the committed MX `GET /health/v1` authority and record MX repository identity, commit SHA, contract/OpenAPI/schema path, schema version, compatibility rules, and SHA-256 digest. Block if the contract is not committed or not redacted/authenticated.
- [ ] 1.3 Copy only a minimal redacted contract fixture into Jcode test data, with provenance metadata and a drift check against the pinned source bytes. Update this proposal first if committed fields materially differ.

## 2. Daemon Adapter and Projection

- [ ] 2.1 Add secret-safe daemon configuration for MX base URL, bearer token source, timeout, response-size cap, and refresh window. Ensure configuration summaries and logs redact secrets.
- [ ] 2.2 Implement the server-side MX health client with bounded I/O, concurrent-refresh coalescing, typed transport/auth errors, and no browser-direct MX access.
- [ ] 2.3 Validate version, required fields, redacted assertion, enums, timestamps, unique check IDs, dependencies, and payload size; fail closed on incompatible or unsafe responses.
- [ ] 2.4 Add the additive Command Center MX health projection and deterministic Rust-to-TypeScript generation. Preserve MX states/reason codes and represent adapter freshness separately from MX health.
- [ ] 2.5 Implement atomic last-known-good caching that can accompany a current failure only as explicitly stale, timestamped data.

## 3. Authenticated Jcode Read Surface

- [ ] 3.1 Add the authenticated read endpoint used by `/mx`, using existing Command Center browser-session and permission rules.
- [ ] 3.2 Return stable safe error categories for unconfigured, unauthorized, unreachable, timeout, incompatible-version, invalid-contract, and unavailable-with-stale-data states.
- [ ] 3.3 Add server tests proving MX tokens, authorization headers, raw responses, DSNs, endpoint credentials, and provider error strings do not enter responses or logs.
- [ ] 3.4 Add endpoint and browser tests for unauthenticated, expired-session, and forbidden access; expected result is the existing Command Center authentication behavior with no MX fetch attempted.

## 4. `/mx` Command Center Experience

- [ ] 4.1 Add the stable `/mx` route and navigation entry without changing existing route behavior.
- [ ] 4.1a Keep `/mx` visible to authenticated operators when MX is unconfigured and render a setup-required state that reveals no endpoint or secret values.
- [ ] 4.2 Build the page header, impact summary, observation age, legend, loading/error/stale regions, retry-read action, semantic layer/check list, and details region.
- [ ] 4.3 Build the inline responsive SVG topology from validated checks and dependencies. Use state text plus shape/icon/color, and do not invent unsupported dependency edges.
- [ ] 4.4 Provide one-to-one keyboard selection between topology nodes and HTML check controls, visible focus, named SVG/description, hidden decorative paths, and a complete non-SVG reading path.
- [ ] 4.5 Support 320px, tablet, and desktop layouts, 200% zoom, forced colors, and reduced motion without clipped controls or horizontal page scrolling.

## 5. Automated Verification

- [ ] 5.1 Add Rust unit/contract tests for healthy, degraded, down, starting, recovering, unknown closed-enum value, documented extensible-enum fallback when applicable, wrong version, unredacted, malformed, oversized, duplicate-ID, dangling-dependency, timeout, unauthorized, stale-cache, and recovery cases.
- [ ] 5.2 Run deterministic contract generation and verify no unexplained generated diff.
- [ ] 5.3 Add Solid component tests for loading, healthy, degraded, down, stale-with-error, no-cache error, details selection, retry, observation age, and non-color status text.
- [ ] 5.4 Add automated accessibility assertions for landmarks/headings, names/roles, live regions, tab order, focus visibility, SVG/list semantic parity, and reduced-motion behavior.
- [ ] 5.4a Add component and Playwright assertions that passive refresh preserves keyboard focus, selected check, and open details while announcing only the concise refresh result.
- [ ] 5.5 Add Playwright responsive coverage at 320, 768, and desktop widths plus 200% zoom and forced-colors emulation; verify no page-level horizontal overflow.
- [ ] 5.6 Add source and browser assertions that `/mx` renders no restart, reconnect, credential-refresh, database-repair, or other MX mutation control and emits no mutation request.

## 6. Managed Runtime and Deployment

- [ ] 6.1 Extend the isolated managed-daemon acceptance launcher with a deterministic authenticated MX fixture that serves the exact pinned schema.
- [ ] 6.2 Exercise healthy, degraded persistence/downstream impact, stale last-known-good, unauthorized, unreachable, and invalid-contract responses through the public `/mx` route.
- [ ] 6.3 Verify the real configured MX endpoint returns only redacted data and that browser network payloads, logs, screenshots, traces, and retained artifacts contain no secret sentinel or forbidden raw field.
- [ ] 6.4 Document configuration, timeout/cache behavior, state meanings, provenance update procedure, and rollback/disable steps.

## 7. Final Gates

- [ ] 7.1 Run `cargo fmt --all -- --check` and focused affected Rust tests; expected result: exit 0.
- [ ] 7.2 Run Command Center `format:check`, `lint`, `typecheck`, `test`, `build`, and `test:e2e`; expected result: exit 0 with no new baseline failures.
- [ ] 7.3 Run `openspec validate add-mx-health-command-center --strict --no-interactive` and `git diff --check`; expected result: exit 0.
- [ ] 7.4 Record final artifact digests and an independent semantic/security/accessibility review mapping every requirement to scenarios, tasks, and checks. Rerun evidence after any artifact mutation.
