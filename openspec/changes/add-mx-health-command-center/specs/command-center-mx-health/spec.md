## ADDED Requirements

### Requirement: Pinned MX health contract
Jcode SHALL consume MX health only from a committed, immutable, authenticated, redacted `mx.health.v1` contract whose repository identity, commit SHA, artifact path, compatibility rules, and digest are recorded before implementation.

#### Scenario: Contract provenance is available
- **WHEN** implementation begins
- **THEN** the exact MX contract bytes and immutable provenance are recorded in Jcode
- **AND** deterministic fixtures and drift checks use those bytes as their authority

#### Scenario: Contract authority is unavailable
- **WHEN** the MX contract is uncommitted, cannot be located, is not authenticated/redacted, or differs materially from this approved change
- **THEN** implementation stops before adapter code is written
- **AND** the OpenSpec change is updated and revalidated rather than guessing the contract

### Requirement: Server-side authenticated MX adapter
The Jcode daemon SHALL fetch MX health through a bounded server-side adapter and SHALL NOT expose MX credentials or direct MX access to browser code.

#### Scenario: Successful authenticated fetch
- **WHEN** MX returns a valid authenticated redacted `mx.health.v1` response within configured limits
- **THEN** Jcode validates and projects the response for the authenticated Command Center client
- **AND** the browser receives no MX bearer token or privileged endpoint configuration

#### Scenario: MX authentication fails
- **WHEN** MX rejects the daemon credential
- **THEN** Jcode returns a stable safe unauthorized state
- **AND** no authorization header, token, or raw upstream body appears in the browser response or logs

#### Scenario: MX is slow or unreachable
- **WHEN** the MX request exceeds the timeout or cannot connect
- **THEN** Jcode terminates the bounded request and reports a typed unavailable state
- **AND** concurrent browser refreshes do not create an unbounded MX request fan-out

#### Scenario: Command Center session is absent or expired
- **WHEN** an unauthenticated browser or an expired Command Center session requests `/mx` or its Jcode health endpoint
- **THEN** Jcode applies the existing authentication failure behavior
- **AND** does not attempt an MX fetch or expose MX configuration

#### Scenario: Command Center session is forbidden
- **WHEN** an authenticated session lacks permission to inspect the Command Center health route
- **THEN** Jcode returns the existing forbidden behavior
- **AND** does not attempt an MX fetch

### Requirement: Strict health contract validation
The adapter SHALL fail closed on unsafe or incompatible MX responses and SHALL preserve valid MX states and reason codes without upgrading their severity.

#### Scenario: Valid degraded response
- **WHEN** MX reports degraded persistence with downstream workflow checks degraded or unavailable
- **THEN** Jcode preserves those states, reason codes, timestamps, dependencies, cached-data semantics, and recovery metadata in its projection
- **AND** Jcode does not present the system as healthy because provider checks remain healthy

#### Scenario: Invalid or unsafe response
- **WHEN** a response has the wrong major version, missing required fields, `redacted` not equal to true, an unknown value for a closed required enum, malformed timestamps, duplicate check IDs, dangling dependencies, or exceeds the size cap
- **THEN** Jcode rejects it as an invalid contract
- **AND** raw invalid payload content is not returned to the browser

#### Scenario: Additive compatible fields
- **WHEN** the pinned MX compatibility rules permit unknown additive fields
- **THEN** Jcode ignores only those fields while validating all known required semantics

#### Scenario: Extensible enum fallback
- **WHEN** the pinned MX contract explicitly declares an enum extensible and supplies an unknown-state fallback
- **THEN** Jcode projects that fallback as non-healthy according to the contract
- **AND** does not reject or upgrade it based on guessed semantics

### Requirement: Explicit live and stale data semantics
Jcode SHALL distinguish current MX health, adapter fetch health, and last-known-good data so stale information cannot appear current.

#### Scenario: Current response replaces cache
- **WHEN** a newly fetched response validates successfully
- **THEN** Jcode atomically replaces the last-known-good projection
- **AND** reports the new observation and fetch timestamps

#### Scenario: Current fetch fails with cached data
- **WHEN** a current fetch fails and a last-known-good projection exists
- **THEN** Jcode may return the cached projection only with an explicit stale state, prior observation timestamp, age, and current failure category
- **AND** any MX degraded or down state in the cached response remains degraded or down

#### Scenario: Current fetch fails without cached data
- **WHEN** a current fetch fails and no validated projection exists
- **THEN** `/mx` presents an unavailable state rather than fabricated component health

### Requirement: Accessible `/mx` health route
Command Center SHALL provide an authenticated `/mx` route that communicates MX overall state, impact, layers, checks, dependencies, timestamps, cached-data semantics, and recovery metadata through both a responsive SVG topology and complete semantic HTML.

#### Scenario: Healthy system
- **WHEN** all required MX checks report healthy
- **THEN** `/mx` presents a textual healthy overall state, observation age, layer topology, semantic check list, and legend

#### Scenario: Partial degradation
- **WHEN** one layer is degraded and dependent layers are degraded or unavailable
- **THEN** `/mx` identifies the affected layer, preserves healthy independent layers, and presents a concise impact statement
- **AND** status is distinguishable through text and shape/iconography, not color alone

#### Scenario: Keyboard inspection
- **WHEN** a keyboard user traverses the health page
- **THEN** every selectable check is reachable with a visible focus indicator
- **AND** selecting a topology or list control reveals the same named HTML details without pointer-only behavior

#### Scenario: Assistive technology reading path
- **WHEN** the SVG is unavailable or ignored by assistive technology
- **THEN** headings, status regions, summaries, lists, and details expose the complete health information in a logical reading order
- **AND** decorative connectors are hidden from assistive technology

#### Scenario: Passive refresh
- **WHEN** health data refreshes without user action
- **THEN** a concise status update is announced without stealing focus or collapsing the user's selected details

#### Scenario: MX is unconfigured
- **WHEN** an authenticated operator opens `/mx` before daemon MX endpoint and token configuration is available
- **THEN** the stable route and navigation entry remain available with a setup-required state
- **AND** no endpoint, token source, or secret value is exposed

### Requirement: Responsive and preference-safe health visualization
The `/mx` route SHALL remain usable from 320 CSS pixels through desktop widths, at 200% zoom, in forced-colors mode, and with reduced motion enabled.

#### Scenario: Narrow viewport
- **WHEN** the viewport is 320 CSS pixels wide
- **THEN** the topology reflows vertically or yields priority to the semantic list
- **AND** controls remain visible without page-level horizontal scrolling

#### Scenario: Zoom and forced colors
- **WHEN** the operator uses 200% zoom or forced-colors mode
- **THEN** text, boundaries, status, selection, and focus remain perceivable and operable

#### Scenario: Reduced motion
- **WHEN** `prefers-reduced-motion` is enabled
- **THEN** nonessential topology or refresh animation is disabled without removing status information

### Requirement: Read-only fail-closed operation
The MX health page SHALL remain read-only and SHALL expose only adapter capabilities verified by this contract.

#### Scenario: Operator retries a read
- **WHEN** the operator activates retry after a fetch failure
- **THEN** Jcode repeats the bounded health read
- **AND** does not restart MX, reconnect a database, refresh credentials, or claim recovery success

#### Scenario: Recovery metadata is present
- **WHEN** MX supplies safe redacted recovery state or explanatory action text
- **THEN** `/mx` displays it as informational context
- **AND** does not convert it into a mutation control without a separately approved authenticated command contract

#### Scenario: Mutation controls remain absent
- **WHEN** `/mx` renders any healthy, degraded, stale, unavailable, or recovery state
- **THEN** it exposes no restart, reconnect, credential-refresh, database-repair, or other MX mutation control
- **AND** browser interaction emits no MX mutation request

### Requirement: Managed-runtime and secret-safety acceptance
The integration SHALL be verified through an isolated managed Jcode runtime and the public `/mx` interface using exact-schema fixtures and a real redacted endpoint before rollout.

#### Scenario: Deterministic state matrix
- **WHEN** acceptance runs against healthy, degraded, stale, unauthorized, unreachable, and invalid-contract fixtures
- **THEN** each public `/mx` state matches the expected contract behavior and accessibility assertions

#### Scenario: Secret scan
- **WHEN** tests and managed-runtime acceptance complete
- **THEN** browser payloads, logs, screenshots, traces, and retained artifacts contain no MX token, authorization header, DSN, endpoint credential, raw provider error, or configured secret sentinel
