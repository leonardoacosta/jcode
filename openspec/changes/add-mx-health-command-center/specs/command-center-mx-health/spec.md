## ADDED Requirements

### Requirement: Pinned committed MX health authority
Jcode SHALL consume MX health only from the committed authenticated and redacted `mx.health.v1` authority pinned by repository, commit, source paths, and digests in this change.

#### Scenario: Pinned authority is unchanged
- **WHEN** implementation or a drift check begins
- **THEN** it verifies the recorded MX commit and contract artifact digests
- **AND** exact-schema redacted fixtures identify that immutable provenance

#### Scenario: Authority drifts materially
- **WHEN** the upstream version, required fields, statuses, authentication, or redaction semantics differ from the pinned authority
- **THEN** implementation stops before accepting the new response
- **AND** this OpenSpec change is updated, reviewed, and revalidated rather than guessing compatibility

#### Scenario: Fields absent from v1
- **WHEN** the adapter or UI projects an MX check
- **THEN** it does not invent per-check timestamps, cached-data flags, data age, recovery metadata, or mutation capabilities absent from the pinned contract

### Requirement: Bounded server-side authenticated MX adapter
The Jcode daemon SHALL fetch MX health through bounded authenticated server-side I/O and SHALL NOT expose direct MX access or credentials to browser code.

#### Scenario: Successful HTTP 200 response
- **WHEN** MX returns HTTP 200 with a valid redacted `mx.health.v1` body
- **THEN** Jcode validates and projects the authoritative MX state
- **AND** the browser receives no MX token, endpoint configuration, or authorization header

#### Scenario: Authoritative HTTP 503 response
- **WHEN** MX returns HTTP 503 with a valid redacted `mx.health.v1` body whose overall state is `down`
- **THEN** Jcode accepts and projects that authoritative down state
- **AND** does not replace it with a generic adapter-unreachable error

#### Scenario: Upstream authentication fails
- **WHEN** MX returns HTTP 401 or 403
- **THEN** Jcode returns a stable safe upstream-unauthorized adapter state
- **AND** no token, authorization header, configured endpoint, or raw body appears in browser data or logs

#### Scenario: MX is slow or unreachable
- **WHEN** DNS, connection, or bounded timeout fails
- **THEN** Jcode terminates the request and reports a typed unreachable or timeout adapter state
- **AND** concurrent browser refreshes create at most one active upstream fetch for the coalescing window

#### Scenario: Unexpected upstream status
- **WHEN** MX returns an HTTP status other than the supported contract-bearing or authentication statuses
- **THEN** Jcode reports a safe typed adapter failure
- **AND** does not expose or parse untrusted content as health

#### Scenario: Browser session is absent, expired, or forbidden
- **WHEN** the Jcode health endpoint is requested without an authorized Command Center session
- **THEN** existing Command Center authentication or forbidden behavior applies before adapter invocation
- **AND** no MX configuration is exposed

### Requirement: Strict v1 validation and safe projection
The adapter SHALL validate the pinned v1 response, preserve valid MX semantics, and fail closed on incompatible or unsafe data.

#### Scenario: Valid response
- **WHEN** the body has `version: "mx.health.v1"`, `redacted: true`, valid `generated_at`, recognized overall/check statuses, unique check IDs, safe required strings, and resolvable dependencies
- **THEN** Jcode projects version, generation/fetch times, overall state, and every check's ID, layer, status, reason code, summary, and dependency IDs

#### Scenario: Persistence failure with blocked workflow
- **WHEN** MX reports provider checks `ok`, persistence `down`, and workflows `blocked` with dependency `persistence`
- **THEN** Jcode preserves each independent status and the dependency edge
- **AND** never presents the system as healthy merely because provider checks remain `ok`

#### Scenario: Unsafe or incompatible response
- **WHEN** version is wrong, `redacted` is not true, JSON or timestamp is malformed, required fields are missing/empty, overall or check status is unknown, check IDs are duplicated, dependencies dangle, or the payload exceeds the cap
- **THEN** Jcode rejects the response as `invalid_contract`
- **AND** raw invalid content does not enter browser responses or logs

#### Scenario: Additive JSON field
- **WHEN** a valid pinned v1 response contains an unknown additive field but all required known semantics remain valid
- **THEN** Jcode ignores that field
- **AND** continues to validate every known required field

### Requirement: Separate adapter freshness and last-known-good semantics
Jcode SHALL distinguish authoritative MX health from current adapter fetch health so stale information cannot appear live.

#### Scenario: Successful fetch replaces cache
- **WHEN** a new response validates successfully
- **THEN** Jcode atomically replaces its one in-memory last-known-good projection
- **AND** records MX `generated_at` separately from Jcode `fetched_at`

#### Scenario: Current fetch fails with eligible cached data
- **WHEN** a current fetch fails and a valid last-known-good projection remains inside the configured stale limit
- **THEN** Jcode may return it only with adapter state `stale`, its prior fetch and MX generation timestamps, computed age, and the current safe failure category
- **AND** cached MX `degraded`, `down`, or `blocked` states retain their severity

#### Scenario: Current fetch fails without eligible cached data
- **WHEN** no validated projection exists or the cache exceeds the stale limit
- **THEN** `/mx` presents the typed adapter failure without fabricated MX checks

#### Scenario: MX is unconfigured
- **WHEN** daemon MX endpoint or token configuration is absent
- **THEN** Jcode returns a setup-required state without attempting an upstream fetch
- **AND** exposes no endpoint or token-source value

### Requirement: Accessible authenticated `/mx` route
Command Center SHALL provide a stable authenticated `/mx` route communicating MX overall state, impact, layers, checks, dependencies, generation/fetch freshness, and adapter state through responsive SVG and complete semantic HTML.

#### Scenario: Healthy system
- **WHEN** MX reports `overall: "ok"` and all checks are `ok`
- **THEN** `/mx` presents textual healthy state, MX generation age, adapter freshness, topology, semantic check list, and legend

#### Scenario: Partial degradation
- **WHEN** MX reports `overall: "degraded"` with degraded or unreachable source checks while persistence remains available
- **THEN** `/mx` identifies affected checks, preserves healthy independent checks, and presents concise impact text
- **AND** every status uses text and shape/iconography in addition to color

#### Scenario: Authoritative system down
- **WHEN** the validated contract reports `overall: "down"`, persistence `down`, and workflows `blocked`
- **THEN** `/mx` shows provider availability separately from persistence/workflow failure and causal dependencies
- **AND** identifies the body as authoritative MX health even though the upstream HTTP status was 503

#### Scenario: Stable unconfigured route
- **WHEN** an authenticated operator opens `/mx` before MX configuration is available
- **THEN** the route and navigation entry remain present with setup-required guidance
- **AND** no configuration value is displayed

#### Scenario: Keyboard inspection
- **WHEN** a keyboard user traverses checks and selects one
- **THEN** each selectable check is reachable with visible focus
- **AND** topology and HTML controls expose the same named details without pointer-only interaction

#### Scenario: Assistive-technology reading path
- **WHEN** SVG is ignored or unavailable
- **THEN** headings, status regions, impact summary, ordered layers/checks, dependencies, and details expose the complete information in logical order
- **AND** decorative paths are hidden from assistive technology

#### Scenario: Passive refresh
- **WHEN** data refreshes without explicit user action
- **THEN** one concise result is announced
- **AND** focus, selected check, details visibility, and unrelated scroll state remain stable

#### Scenario: Adapter failure and retry
- **WHEN** the page has an unauthorized, unreachable, timeout, invalid-contract, or stale state
- **THEN** it uses explicit safe copy and may offer retry of the Jcode read
- **AND** retry performs no MX mutation

### Requirement: Responsive and preference-safe topology
The `/mx` route SHALL remain usable from 320 CSS pixels through desktop widths, at 200% zoom, in forced-colors mode, and with reduced motion enabled.

#### Scenario: Narrow viewport
- **WHEN** the viewport is 320 CSS pixels wide
- **THEN** the topology reflows vertically or yields visual priority to the semantic list
- **AND** no control is clipped and no page-level horizontal scrolling is required

#### Scenario: Desktop viewport
- **WHEN** sufficient width is available
- **THEN** the SVG presents ordered layers horizontally while preserving semantic-list and details parity

#### Scenario: Zoom and forced colors
- **WHEN** the operator uses 200% zoom or forced-colors mode
- **THEN** text, boundaries, status, selection, and focus remain perceivable and operable

#### Scenario: Reduced motion
- **WHEN** `prefers-reduced-motion` is enabled
- **THEN** nonessential topology and refresh animation is absent or disabled without removing information

### Requirement: Read-only and secret-safe operation
The feature SHALL expose only health reads and SHALL prevent MX secrets or unsupported controls from entering public surfaces or retained artifacts.

#### Scenario: No mutation controls
- **WHEN** `/mx` renders any state
- **THEN** it contains no restart, reconnect, credential-refresh, database-repair, or other MX mutation control
- **AND** browser interactions emit no MX mutation request

#### Scenario: Secret scanning
- **WHEN** unit, browser, and managed-runtime tests complete
- **THEN** browser payloads, logs, screenshots, traces, and retained artifacts contain no MX token, authorization header, endpoint credential, DSN, provider identity, raw provider error, or configured secret sentinel

### Requirement: Public managed-runtime acceptance
The integration SHALL be accepted through an isolated managed Jcode runtime and the public authenticated `/mx` interface before rollout.

#### Scenario: Deterministic state matrix
- **WHEN** the isolated MX fixture serves healthy/200, degraded/200, down/503, upstream-unauthorized, unreachable, invalid-contract, stale-sequence, and unconfigured cases
- **THEN** the public `/mx` route matches the specified state, security, accessibility, and responsive behavior for each case

#### Scenario: Existing routes remain compatible
- **WHEN** MX configuration is absent or MX is unavailable
- **THEN** existing Command Center inbox, ambient, find, initiative, event-stream, and authentication workflows retain their prior behavior

#### Scenario: Real endpoint gate
- **WHEN** acceptance runs against the configured real MX endpoint
- **THEN** the daemon validates a redacted `mx.health.v1` response from the pinned-compatible authority
- **AND** browser-observable surfaces remain secret-free
