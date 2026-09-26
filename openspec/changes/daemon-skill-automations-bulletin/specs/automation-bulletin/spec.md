## ADDED Requirements

### Requirement: Local and tailnet single-page automation bulletin
The system SHALL serve one HTMX-powered HTML page from an opt-in daemon-owned loopback listener and a first-class private Tailscale HTTPS endpoint, with bundled assets and no separate frontend runtime.

#### Scenario: Inspect and manage automations
- **WHEN** an authenticated owner opens the page
- **THEN** it shows skill/schedule/timezone/state/next due time, allows creation with skill and interval or weekday/time/timezone inputs plus optional arguments/directory, and supports pause, resume, and schedule changes.
- **WHEN** a form is invalid
- **THEN** a field-level error preserves entered values and does not mutate the definition.

#### Scenario: Live results
- **WHEN** a run finishes while the page is visible
- **THEN** its outcome and expandable final response appear within ten seconds on a healthy local connection without a full-page reload or loss of form focus/unsaved inputs.

#### Scenario: Empty, unavailable, and no-script states
- **WHEN** no automations or results exist
- **THEN** the page explains the empty state and offers creation.
- **WHEN** refresh requests fail
- **THEN** the page keeps existing results and shows stale/disconnected status.
- **WHEN** JavaScript is disabled
- **THEN** ordinary forms and manual refresh still provide management and result access.

### Requirement: Private and inert output
The server SHALL require an authenticated browser session on both local and Tailscale endpoints, reject public/LAN listener binding and unapproved hosts/origins, protect mutations from CSRF, and treat all skill output as untrusted display content.

#### Scenario: Bootstrap and unauthorized requests
- **WHEN** the owner opens a valid, unexpired one-time CLI bootstrap URL locally or over Tailscale HTTPS
- **THEN** the server issues a host-only HttpOnly SameSite=Strict session cookie, also Secure for remote access, and redirects to a clean URL, invalidating the bootstrap token.
- **WHEN** a request lacks a valid session or attempts to reuse the bootstrap token
- **THEN** it cannot read automation results or mutate definitions.

#### Scenario: Cross-origin or forged requests
- **WHEN** a request uses an unexpected Host/Origin or a mutation lacks its valid CSRF token, or forged forwarding/identity headers try to bypass checks
- **THEN** the request is rejected with no state change and no permissive CORS response.

#### Scenario: Hostile result content
- **WHEN** a skill returns script tags, event handlers, HTMX attributes, or file paths
- **THEN** the bulletin displays inert escaped text and exposes neither script execution nor arbitrary file downloads.

### Requirement: Bounded history and accessible controls
The bulletin SHALL show newest results first, page in groups of at most 50, retain at most 1,000 terminal records, and cap each stored final response at 64 KiB with visible truncation. Forms and results SHALL support keyboard use, labeled fields, visible focus, and status text independent of color.

#### Scenario: Large result and long history
- **WHEN** a result exceeds the output cap or terminal history exceeds retention
- **THEN** truncation respects UTF-8 boundaries, the UI identifies truncation, and oldest terminal records are pruned without deleting active runs or linked session files.

#### Scenario: Keyboard and narrow viewport
- **WHEN** the owner uses only a keyboard or a 390-pixel-wide viewport
- **THEN** creation, validation, pause/resume, pagination, and result expansion remain operable without clipped controls.

### Requirement: Listener lifecycle and failure isolation
The listener SHALL start with the configured production daemon, stop on shutdown, expose its actual URL/status, and bound request bodies to 16 KiB and concurrent requests to 16.

#### Scenario: Port occupied or listener reload
- **WHEN** the configured port cannot bind
- **THEN** Jcode sessions and automation scheduling remain operational and CLI status identifies the unavailable bulletin.
- **WHEN** the daemon reloads
- **THEN** the previous listener releases ownership and at most one replacement listener serves the configured address.

#### Scenario: Oversized traffic and temporary daemons
- **WHEN** a client exceeds the request-body or concurrency limit
- **THEN** excess work is rejected or bounded without stalling scheduling.
- **WHEN** a temporary/test daemon starts with defaults
- **THEN** it neither exposes a production bulletin nor acquires production automation state.

### Requirement: First-class private Tailscale access
The system SHALL provision and report a persistent private Tailscale Serve HTTPS endpoint. Authorized browsers on other tailnet devices SHALL have the same automation management, results, and live-refresh capabilities as local browsers. Public Funnel access SHALL NOT be enabled.

#### Scenario: Remote device acceptance
- **WHEN** the owner pairs a browser on a second authorized tailnet device using the reported HTTPS MagicDNS URL
- **THEN** creation, schedule editing, pause/resume, result expansion, pagination, and live updates work without localhost URLs, mixed content, or manual forwarding.

#### Scenario: Missing prerequisites or endpoint conflict
- **WHEN** Tailscale is missing, logged out, lacks HTTPS readiness, or the selected Serve endpoint conflicts with an existing mapping
- **THEN** provisioning reports the specific blocker without overwriting routes, changing access policy, enabling public ingress, or claiming remote readiness.

#### Scenario: Unauthorized access and session expiry
- **WHEN** an unpaired tailnet browser requests the page, fragments, or mutations
- **THEN** no automation data is disclosed and no mutation occurs, even if network policy permits connectivity.
- **WHEN** a bootstrap token is older than five minutes or a browser session is expired after 24 hours or explicitly revoked
- **THEN** further access requires a new valid pairing, and repeated invalid bootstrap attempts are rate-limited.

#### Scenario: Network outage and recovery
- **WHEN** Tailscale becomes unavailable
- **THEN** local scheduling and local access continue, remote status becomes degraded, and the private endpoint recovers when connectivity returns.

#### Scenario: Restart and scoped removal
- **WHEN** daemon and Tailscale services restart with prerequisites available
- **THEN** the same provisioned HTTPS endpoint resumes serving the bulletin without browser-driven startup.
- **WHEN** tailnet serving is unprovisioned
- **THEN** only this feature's owned Serve mapping is removed and bulletin sessions are revoked while unrelated mappings remain unchanged.

#### Scenario: Network boundary verification
- **WHEN** a device excluded by tailnet access policy or a device outside the tailnet attempts access
- **THEN** it cannot reach the private endpoint, and provisioning has created no LAN or public listener.
