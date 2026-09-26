## ADDED Requirements

### Requirement: Local single-page automation bulletin
The system SHALL serve one HTMX-powered HTML page from an opt-in daemon-owned loopback listener, with local assets and no separate frontend runtime.

#### Scenario: Inspect and manage automations
- **WHEN** an authenticated owner opens the page
- **THEN** it shows skill/frequency/state/next due time, allows creation with skill and frequency plus optional arguments/directory, and supports pause, resume, and frequency changes.
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
The server SHALL require a local browser session, reject non-loopback binding and unexpected hosts/origins, protect mutations from CSRF, and treat all skill output as untrusted display content.

#### Scenario: Bootstrap and unauthorized requests
- **WHEN** the owner opens a one-time CLI bootstrap URL
- **THEN** the server issues an HttpOnly SameSite=Strict session cookie and redirects to a clean URL, invalidating the bootstrap token.
- **WHEN** a request lacks a valid session or attempts to reuse the bootstrap token
- **THEN** it cannot read automation results or mutate definitions.

#### Scenario: Cross-origin or forged requests
- **WHEN** a request uses an unexpected Host/Origin or a mutation lacks its valid CSRF token
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
