# chrome-browser-provider Specification

## Purpose
TBD - created by archiving change add-agent-browser-chrome-provider. Update Purpose after archive.
## Requirements
### Requirement: Explicit Chrome provider routing
Jcode SHALL expose a trusted `agent-browser` executable as the first-class provider selected by `browser: "chrome"`, and explicit Chrome requests SHALL never silently execute through another browser provider.

#### Scenario: Execute an explicit Chrome request
- **WHEN** a browser tool action specifies `browser: "chrome"` and the Chrome provider is ready
- **THEN** Jcode SHALL execute the action through `agent-browser`
- **AND** the result metadata SHALL identify `backend: "agent_browser"` and `browser: "chrome"`.

#### Scenario: Explicit Chrome provider is unavailable
- **WHEN** a browser tool action specifies `browser: "chrome"` and the trusted `agent-browser` executable, supported command surface, or Chrome runtime is unavailable
- **THEN** Jcode SHALL return actionable Chrome-specific readiness or setup guidance
- **AND** it SHALL NOT fall back to Firefox.

#### Scenario: Explicit Firefox behavior remains isolated
- **WHEN** a browser tool action specifies `browser: "firefox"`
- **THEN** Jcode SHALL preserve the existing Firefox Agent Bridge path
- **AND** it SHALL NOT execute the request through Chrome.

### Requirement: Trusted Chrome executable readiness and setup
Jcode SHALL diagnose the optional Chrome provider without requiring it for startup, SHALL trust and pin the executable before use, and SHALL perform installation work only after an explicit Chrome setup request.

#### Scenario: Discover a trusted executable
- **WHEN** Chrome status, setup, or action execution needs an `agent-browser` executable
- **THEN** Jcode SHALL prefer an absolute `JCODE_AGENT_BROWSER_BIN` override, otherwise discover `agent-browser` from `PATH`
- **AND** it SHALL canonicalize and pin the selected executable fingerprint before any provider action.

#### Scenario: Reject unsafe executable discovery
- **WHEN** a discovered executable is relative, current-directory based, repository-local, not a regular executable, insecurely writable, replaced after readiness, or otherwise fails trust validation
- **THEN** Chrome status SHALL report not ready with actionable trust guidance
- **AND** Jcode SHALL NOT execute automatic Chrome actions through that candidate.

#### Scenario: Report healthy Chrome readiness
- **WHEN** a compatible pinned agent-browser version runs `doctor --json --offline`, reports no failing checks, and its launch test passes
- **THEN** Chrome provider status SHALL report ready
- **AND** warning and informational checks SHALL remain available as diagnostics without making readiness false
- **AND** Jcode SHALL disclose that doctor may clean only stale daemon socket, pid, and version sidecars.

#### Scenario: Agent-browser version or protocol is incompatible
- **WHEN** the discovered CLI is outside the supported `>=0.27.3,<0.28.0` range or lacks a required JSON operation
- **THEN** Chrome status SHALL report not ready with upgrade or compatibility guidance
- **AND** Jcode SHALL NOT attempt browser actions through that CLI.

#### Scenario: Agent-browser executable is missing
- **WHEN** Chrome status or setup cannot discover an executable
- **THEN** Jcode SHALL report that the optional CLI is missing and provide installation guidance
- **AND** Jcode SHALL NOT invoke a package manager automatically.

#### Scenario: Chrome runtime is missing during explicit setup
- **WHEN** the trusted CLI exists, doctor reports the Chrome runtime unavailable, and the user invokes setup with `browser: "chrome"`
- **THEN** Jcode SHALL run `agent-browser install` with a bounded timeout
- **AND** it SHALL rerun doctor and report the resulting readiness state.

#### Scenario: Chrome is already healthy during setup
- **WHEN** the user invokes setup with `browser: "chrome"` and doctor already reports ready
- **THEN** Jcode SHALL perform no installation and SHALL return the healthy status.

### Requirement: Per-session Chrome isolation
Jcode SHALL execute Chrome automation in a unique named agent-browser session derived from the complete Jcode session and SHALL avoid the user's existing browser profile, configuration, and behavior-changing environment by default.

#### Scenario: Create an isolated Chrome session
- **WHEN** a Jcode session performs its first Chrome browser action
- **THEN** every provider command SHALL include a collision-resistant sanitized `--session jcode-<readable-prefix>-<stable-hash>` value
- **AND** it SHALL use a Jcode-owned neutral config and working directory
- **AND** it SHALL remove inherited `AGENT_BROWSER_*` settings before setting only Jcode-controlled values.

#### Scenario: Hostile configuration or environment exists
- **WHEN** user, project, environment, or CLI defaults specify profile, state, session-name, auto-connect, CDP, extension, init-script, engine/provider override, executable override, remote-provider, proxy credential, or auth-vault behavior
- **THEN** Jcode SHALL ignore or clear those settings for Chrome provider actions unless the request uses the explicit `profile` capability below
- **AND** it SHALL NOT attach to the user's daily browser profile or restored auth state.

#### Scenario: Explicitly select a Chrome profile
- **WHEN** a request uses `browser: "chrome"` with a `profile` name
- **THEN** Jcode SHALL accept only a bounded name and SHALL reject filesystem paths or traversal
- **AND** it SHALL resolve a matching custom profile beneath the agent-browser profile directory, otherwise pass the validated Chrome profile name
- **AND** it SHALL isolate the credential-bearing profile in a profile-specific Jcode session
- **AND** it SHALL mark the result metadata as using a credential-bearing profile.

#### Scenario: Attempt profile selection through another route
- **WHEN** a request supplies `profile` with `browser: "auto"`, Firefox, or another provider
- **THEN** Jcode SHALL reject the request instead of silently ignoring or rerouting the profile.

#### Scenario: Sanitized session names would collide
- **WHEN** two distinct Jcode session IDs normalize to the same readable session-name prefix, including punctuation, Unicode, empty-prefix, or long-ID cases
- **THEN** their stable hash suffixes SHALL differ
- **AND** the Chrome sessions SHALL remain isolated.

#### Scenario: Separate Jcode sessions use Chrome concurrently
- **WHEN** two Jcode sessions open pages and mutate browser storage through Chrome
- **THEN** their tabs, cookies, local storage, session storage, and active-page state SHALL remain isolated.

#### Scenario: Chrome actions race within one Jcode session
- **WHEN** multiple Chrome browser actions are requested concurrently for the same Jcode session
- **THEN** Jcode SHALL serialize those provider actions for that session
- **AND** it SHALL preserve active-tab and ref-dependent ordering.

#### Scenario: Abandoned Chrome session cleanup
- **WHEN** a named Chrome browser session becomes idle
- **THEN** the launched provider daemon SHALL have a bounded idle timeout controlled by Jcode-owned provider settings.

### Requirement: Normalized Chrome browser actions
The Chrome provider SHALL implement Jcode's normalized core browser actions where agent-browser has an equivalent and SHALL reject unsupported targeting explicitly.

#### Scenario: Navigate and inspect a page
- **WHEN** Jcode performs open, snapshot, get-content, interactables, list-tabs, get-active-tab, new-tab, or select-tab through Chrome
- **THEN** the adapter SHALL invoke the corresponding agent-browser operation
- **AND** it SHALL normalize the result into Jcode's existing browser output and metadata shapes.

#### Scenario: Interact with page controls
- **WHEN** Jcode performs click, type, fill-form, select, wait, eval, scroll, upload, or press through Chrome with valid inputs
- **THEN** the adapter SHALL execute the equivalent selector, accessibility-ref, semantic-locator, keyboard, mouse, or stdin-batch operation without shell interpolation.

#### Scenario: Click by coordinates
- **WHEN** a Chrome click supplies valid x and y coordinates rather than a selector or text locator
- **THEN** the adapter SHALL execute a mouse move, down, and up sequence at those coordinates
- **AND** it SHALL return one normalized click result.

#### Scenario: Chrome iframe interaction through snapshot refs
- **WHEN** a Chrome snapshot contains supported iframe content
- **THEN** the snapshot SHALL expose agent-browser's inlined iframe accessibility refs
- **AND** later supported actions SHALL accept those refs without a separate frame switch.

#### Scenario: Unsupported frame or window targeting
- **WHEN** a Chrome action supplies `list_frames`, `window_id`, `frame_id`, `all_frames`, or another targeting combination that the adapter cannot faithfully honor
- **THEN** Jcode SHALL return an actionable unsupported-provider error
- **AND** it SHALL NOT silently discard the targeting input.

#### Scenario: Provider-specific command is requested through Chrome
- **WHEN** a Chrome request uses `provider_command`
- **THEN** Jcode SHALL reject the action as unsupported for Chrome in this change
- **AND** it SHALL NOT expose agent-browser auth, profile, connection, install, upgrade, dashboard, chat, recording, file-producing, cross-session close, or arbitrary CLI surfaces.

### Requirement: Opaque Chrome tab references without breaking Firefox IDs
Jcode SHALL preserve existing Firefox integer `tab_id` behavior and SHALL add an opaque string `tab_ref` field for Chrome tab identifiers.

#### Scenario: Select a Chrome tab by stable ID
- **WHEN** Chrome tab listing returns a stable ID such as `t1` and a later request supplies that string as `tab_ref`
- **THEN** Jcode SHALL select the same tab without coercing the ID to an integer.

#### Scenario: Preserve Firefox integer compatibility
- **WHEN** an existing Firefox caller supplies an integer `tab_id`
- **THEN** Jcode SHALL serialize and route that identifier as before.

#### Scenario: Reject an invalid identifier for the selected provider
- **WHEN** a provider cannot accept the supplied identifier representation or both `tab_id` and `tab_ref` are supplied ambiguously
- **THEN** Jcode SHALL return provider-specific guidance rather than selecting a different target.

#### Scenario: Tool schema is transformed by provider dialects
- **WHEN** a model-provider schema transformer receives the browser tool schema
- **THEN** the transformed schema SHALL preserve legacy `tab_id` and additive `tab_ref` fields without relying on a string-or-integer union for one field.

### Requirement: Chrome output normalization and secret safety
Jcode SHALL parse, bound, and normalize agent-browser process output without exposing typed field values, secret-bearing URL components, scripts, or native command echoes through rendered provider output, metadata, traces, or diagnostics.

#### Scenario: Parse a successful provider envelope
- **WHEN** agent-browser exits successfully with `{ "success": true, "data": ... }`
- **THEN** Jcode SHALL expose normalized `data` plus backend metadata
- **AND** it SHALL NOT require consumers to understand the native envelope.

#### Scenario: Parse a provider error envelope
- **WHEN** agent-browser returns `success: false`, exits nonzero, times out, emits malformed JSON, exceeds an output limit, or omits mandatory output
- **THEN** Jcode SHALL return a bounded actionable error containing safe provider context
- **AND** it SHALL terminate or reap a timed-out child process.

#### Scenario: Bound subprocess input and output
- **WHEN** Chrome provider actions execute through a subprocess
- **THEN** Jcode SHALL use argument arrays and bounded stdin/stdout/stderr readers rather than a shell or unbounded output buffering
- **AND** provider and user timeouts SHALL be clamped by an outer hard deadline.

#### Scenario: Redact stdin batch command echoes and direct arguments
- **WHEN** text-bearing actions are sent through agent-browser or provider output repeats user-controlled values
- **THEN** Jcode SHALL remove command arrays and recursively redact known typed values before constructing tool output, metadata, trace summaries, or side-panel content.

#### Scenario: Attach a Chrome screenshot
- **WHEN** a Chrome screenshot action succeeds and produces a PNG file at the expected exclusive Jcode-owned path
- **THEN** Jcode SHALL verify the file is regular, bounded, valid PNG data within dimension limits, attach the image to the tool output, include safe provider metadata, and remove the temporary file.

#### Scenario: Screenshot output file is invalid
- **WHEN** agent-browser reports screenshot success but the expected file is missing, substituted, symlinked, non-regular, oversized, or not a valid PNG
- **THEN** Jcode SHALL return an actionable screenshot error
- **AND** it SHALL clean up any remaining temporary path.

#### Scenario: Upload path is invalid
- **WHEN** a Chrome upload action references a missing, symlinked, non-regular, or provider-substituted file
- **THEN** Jcode SHALL reject the upload before or during provider execution with an actionable file error.

### Requirement: Sticky readiness-aware automatic browser selection
Jcode SHALL use provider readiness for `browser: "auto"` while preserving strict explicit-provider behavior and same-session workflow continuity.

#### Scenario: Firefox is healthy in automatic mode
- **WHEN** `browser: "auto"` is requested, the session has no sticky provider affinity, and Firefox Agent Bridge is ready
- **THEN** Jcode SHALL select Firefox
- **AND** it SHALL NOT start a Chrome session for the action beyond bounded readiness evaluation.

#### Scenario: Firefox is unavailable and Chrome is healthy after parity gate
- **WHEN** `browser: "auto"` is requested, the session has no sticky provider affinity, Firefox is not ready, explicit Chrome parity has passed, and the Chrome readiness probe is ready
- **THEN** Jcode SHALL execute through Chrome
- **AND** result metadata SHALL report the selected Chrome backend and the Firefox fallback reason.

#### Scenario: Automatic provider affinity exists
- **WHEN** a Jcode session has already completed a browser action through Firefox or Chrome and a later request uses `browser: "auto"`
- **THEN** Jcode SHALL reuse the same provider while it remains healthy
- **AND** it SHALL NOT silently migrate existing tabs, refs, or active-page state to a different provider.

#### Scenario: Affinity provider fails
- **WHEN** the sticky provider for a Jcode session becomes unhealthy
- **THEN** Jcode SHALL return an actionable provider failure with alternate-provider readiness context
- **AND** it SHALL require an explicit reset or explicit provider choice before using another provider for that session.

#### Scenario: No browser provider is ready
- **WHEN** `browser: "auto"` is requested and neither Firefox nor Chrome is ready
- **THEN** Jcode SHALL return combined bounded diagnostics for both providers
- **AND** it SHALL identify the explicit setup action for each provider.

#### Scenario: Automatic status inspection
- **WHEN** status is requested with `browser: "auto"`
- **THEN** Jcode SHALL report each provider's readiness and the provider that would be selected
- **AND** it SHALL NOT install a browser, modify provider configuration, open a user profile, or run destructive doctor repair
- **AND** it MAY perform documented stale daemon sidecar cleanup required by the Chrome doctor probe.

#### Scenario: Automatic setup while a provider is ready
- **WHEN** setup is requested with `browser: "auto"` and Firefox or Chrome is already ready
- **THEN** Jcode SHALL return the selected ready provider without invoking an installer.

#### Scenario: Automatic setup while no provider is ready
- **WHEN** setup is requested with `browser: "auto"` and neither provider is ready
- **THEN** Jcode SHALL preserve Firefox-first compatibility by using the Firefox setup path
- **AND** Chrome runtime installation SHALL require explicit `browser: "chrome"`.

#### Scenario: Reuse cached readiness safely
- **WHEN** repeated automatic actions occur within the bounded readiness cache lifetime
- **THEN** Jcode MAY reuse provider readiness without rerunning doctor
- **AND** setup, provider execution failure, affinity reset, or executable fingerprint change SHALL invalidate the relevant cache entry.

### Requirement: Chrome provider verification
The repository SHALL include deterministic and live verification that proves command mapping, compatibility, isolation, safety, schema stability, and real browser behavior before automatic Chrome fallback is active.

#### Scenario: Run deterministic provider tests
- **WHEN** browser-focused jcode-app-core tests run with a fake `JCODE_AGENT_BROWSER_BIN`
- **THEN** they SHALL cover executable trust and replacement, compatible-version checks, routing, command arguments and stdin, `tab_ref` schema compatibility, readiness, affinity, fallback, cache invalidation, cleared environment/config, collision-resistant sessions, serialization, redaction, timeouts, output limits, malformed output, error envelopes, screenshots, uploads, unsupported targets, and Chrome provider-command rejection.

#### Scenario: Run live localhost parity test
- **WHEN** the ignored live test is explicitly enabled with `JCODE_AGENT_BROWSER_LIVE=1` on a machine with a healthy agent-browser runtime
- **THEN** it SHALL exercise open, snapshot, fill, click, content read, selection, tabs, same-origin and cross-origin iframe refs where supported, screenshot, cookies, local storage, session storage, tab separation, active-tab separation, collision-prone session IDs, and close against deterministic localhost pages
- **AND** it SHALL leave no live test session behind.

#### Scenario: Preserve Firefox regression coverage
- **WHEN** the complete browser-focused test gate runs
- **THEN** existing Firefox provider tests SHALL continue to pass alongside Chrome provider tests.
