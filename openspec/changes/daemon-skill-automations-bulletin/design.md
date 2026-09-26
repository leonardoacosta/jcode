## Context and Evidence

- `crates/jcode-app-core/src/ambient.rs`: `ScheduledItem` and `ScheduleTarget` represent one-shot tasks, not recurring definitions.
- `crates/jcode-app-core/src/ambient/persistence.rs` and `ambient/manager.rs`: persisted queue and direct-task dequeue. Do not implement recurrence by repeatedly calling the one-shot tool.
- `crates/jcode-app-core/src/ambient/runner.rs`: dispatches ready direct deliveries. Reuse execution conventions without requiring Ambient enablement.
- `crates/jcode-base/src/skill.rs`: `SkillRegistry::load_for_working_dir`, `get`, and `parse_invocation` provide project-aware skill resolution.
- `crates/jcode-app-core/src/server/headless.rs`: `create_headless_session` provides provider, directory, MCP, and memory setup. It is a private integration seam, not an already suitable public automation API.
- `crates/jcode-app-core/src/server/runtime.rs`: owns task cancellation and shutdown. `server/lifecycle.rs` explicitly applies idle shutdown to temporary servers.
- `crates/jcode-app-core/Cargo.toml`: Tokio, serde, chrono, UUID, and reqwest available. No direct general HTTP server dependency identified in inspected manifests. Select a maintained HTTP server during implementation if no existing reusable server exists.
- `crates/jcode-base/src/gateway.rs`: existing MagicDNS detection and remote-device pairing concepts. Its wildcard WebSocket listener is not a private HTTP serving template. Reuse discovery where appropriate, do not copy wildcard binding or manual HTTP parsing. No `chrono-tz` entry found in the inspected lockfile; calendar arithmetic needs an IANA-aware library selected during implementation, not fixed UTC offsets.
- Existing changes cover Ambient AgentMail and overnight questions, not this capability. Shared server/agent files may conflict with concurrent work, but neither proposal is a functional dependency.

## Decisions

### One owner, separate recurring state

Add a daemon-owned automation manager, independent of Ambient enablement and attached to the existing shutdown boundary. Keep the one-shot queue unchanged. Store versioned automation definitions and bounded run records under the existing Jcode data directory, using atomic replacement and a single-writer lock. Temporary/test daemons do not acquire production automation ownership or listeners by default. Corruption or write failure blocks dispatch rather than silently replacing state.

Definition fields: stable ID, skill name, plain optional arguments, absolute working directory, tagged schedule (interval seconds, or nonempty weekday set/local HH:MM/IANA timezone), enabled flag, creation time, next due time, selected provider/model reference. Secrets are never copied into definitions. Resolve the skill in that directory at creation and each run. Use current installed skill contents and record resolved path/content digest for traceability. Missing skill/directory pauses that automation with an actionable error.

### Intervals, calendar schedules, and conservative recovery

Frequency accepts positive integer minute/hour/day units, minimum 60 seconds, with checked conversion and timestamp arithmetic. Store UTC due times, display localized timestamps with timezone labels. Calendar mode accepts one or more weekdays, one 24-hour HH:MM time, and an explicit IANA timezone. Suggest the host timezone only when reliably identified, otherwise require selection. Persist the zone name, never only the current UTC offset. Creation/resume/schedule edit sets the next due time to the first matching occurrence strictly after now (now plus interval in interval mode). Show a human-readable schedule and the next three occurrences with UTC offsets before saving. Pausing does not kill an already running session. No delete or manual-run control in v1.

Dispatch the earliest due enabled automation when the global automation slot is free, ties by ID. Persist a running record and advance its due time to the first matching schedule occurrence strictly after the current time before starting execution. At startup, advance all overdue schedules into the future without catch-up. While an automation runs, coalesce its elapsed ticks into the next future occurrence so it cannot immediately rerun for its own missed ticks. Other due automations wait for the slot without accumulating separate queued runs.

Calendar rules: skip nonexistent local times during spring-forward, run only the earlier occurrence of a repeated local time during fall-back. Store the claimed UTC instant and calendar local date/time identity to prevent duplicate execution. Host timezone changes do not affect an explicitly zoned schedule. Recompute future calendar occurrences using current timezone rules at startup; never replay already claimed local occurrences. Unknown zones or unavailable timezone data pause affected calendar definitions with an actionable error. Interval schedules remain elapsed durations and are unaffected by DST.

Mark orphaned running records interrupted after restart, never automatically replay them. This is deliberately not exactly-once execution across external side effects. Use a 30-minute run deadline, propagate cancellation, retain the slot until termination is acknowledged, and record timeout. Failures advance normally rather than creating retry storms. An unacknowledged cancellation blocks further dispatch and exposes degraded state.

### Existing runtime and permission boundaries

Run the normal skill invocation in a fresh headless session with the captured directory and configured provider/model. Do not inherit an interactive parent session or create swarms. Preserve normal tool permissions. A request needing fresh human authorization ends as blocked rather than hanging or granting approval. Persist session ID, start/end times, outcome, bounded final response, and sanitized failure reason. The final response is the bulletin entry, not a second model-generated summary.

### A single HTML page, not a frontend application stack

Serve the page and same-origin HTML fragments from one listener inside the daemon. Bundle pinned HTMX locally. Poll visible status/results every five seconds with bounded pages, preserve form focus/unsaved inputs, and provide ordinary form POST/redirect fallback. Render output as escaped text preserving whitespace, not executable HTML or raw Markdown HTML. Show automation skill/schedule/timezone/state/next run and local plus tailnet connection status, run timestamps/duration/outcome, and expandable result text. First page shows newest 50 runs. Retain at most 1,000 terminal records and 64 KiB UTF-8-safe output per run, explicitly mark truncation, never prune active records. Existing session retention remains unchanged.

### Tailscale is a first-class access path

Keep the internal HTTP listener on IPv4 loopback, but publish it privately through persistent Tailscale Serve HTTPS. Loopback is an implementation boundary, not the product's access boundary. The owner can create/edit/pause automations and read live results from an authorized second tailnet device using the canonical HTTPS MagicDNS URL. All fragments, form actions, polling, and assets use relative same-origin URLs. Never enable Funnel, wildcard/LAN binding, or public ingress as a fallback.

Provisioning checks Tailscale installation, connected identity, MagicDNS/HTTPS readiness, available Serve configuration, and required privileges. Inspect existing Serve routes before claiming a dedicated HTTPS endpoint; persist exactly which mapping this feature owns. If the default HTTPS endpoint is occupied, require an approved nonconflicting endpoint rather than overwriting it. Use supported persistent Serve configuration, verify the reported route, and show both loopback and canonical tailnet URLs plus separate health states. No automatic login, ACL/grant broadening, or certificate/account-policy changes. Missing prerequisites produce specific setup instructions and `tailnet not provisioned`, not a success claim. Exact Serve syntax and supported-version behavior must be checked against official documentation during implementation.

Tailnet policy limits network reachability but does not authorize every tailnet member to operate Jcode. Require app-level browser pairing for local and remote access. A CLI-issued, short-lived single-use bootstrap token pairs the intended browser, exchanges for an opaque server-side session, then redirects to a clean URL. Expire bootstrap tokens after five minutes, cap failed attempts, avoid third-party assets, set Referrer-Policy: no-referrer, and redact tokens in logs. Remote cookies are Secure, HttpOnly, SameSite=Strict and host-only; local cookies are host-only and distinct. Sessions expire after 24 hours and support explicit revocation. Logout/revocation invalidates access on subsequent requests. Tailscale identity headers alone never grant app permission. Do not trust client-supplied forwarded headers to select public origin or authorization.

### Browser security and bounded resources

Allow only provisioned loopback and tailnet Host/Origin pairs; account explicitly for the trusted Serve proxy's documented host forwarding without allowing arbitrary forwarded hosts. Disable CORS, protect all mutations with per-session CSRF tokens, and require authenticated reads. Apply origin checks even when the authenticated request reaches the loopback upstream through Serve. No arbitrary filesystem serving. Limit request bodies to 16 KiB and concurrent HTTP requests to 16. Set a restrictive CSP compatible with HTMX configured without eval. Mask known credentials in captured diagnostics/output while warning that skill output can still contain sensitive user data.

A Tailscale outage does not stop local scheduling or local access. Report degraded remote access and recover the owned route when connectivity returns without changing unrelated Serve configuration. Unprovisioning removes only the owned mapping, not all Serve state, and invalidates bulletin sessions. Service restart and host restart must restore both daemon and private endpoint when their prerequisites remain available.

### Always-on means supervised daemon, not browser activity

Opt-in provisioning starts/enables one Linux user service supervising the existing daemon and its embedded listener. Reuse existing service/launcher mechanisms if discovered during implementation. Do not start a second daemon against the same socket/state. Reloads release and reacquire listener/state ownership. Bind failure leaves normal Jcode sessions and scheduling operational, with a visible degraded bulletin status.

User-systemd alone is insufficient after logout: provisioning checks linger, explains its effect, and requires explicit authorization before enabling it. If user systemd/linger is unavailable or denied, report that login-independent operation is not provisioned. Do not claim boot persistence. Other platforms may run daemon-lifetime mode, with the limitation stated. Uninstall stops/disables the managed service without deleting automation history or unrelated services. Disabling bulletin does not implicitly disable existing automation definitions, and this distinction must be explicit in controls/docs.

## Migration and Rollback

Feature disabled by default. Existing configs and schedule files load unchanged. Provisioning creates separate versioned state and service metadata only after authorization. Rollback pauses automations, disables the managed service/listener, removes only the owned Serve mapping, revokes bulletin sessions, and retains records. Unknown future state versions fail closed without rewriting them.

## Risks

Unattended skills can spend money or change files: explicit enablement, normal permissions, serial runs, deadlines, visible outcomes. Clock jumps can change observed cadence: UTC due times and no catch-up bursts. Disk exhaustion blocks new dispatch. Crash recovery cannot infer whether external side effects completed: interrupted outcomes, no automatic replay. Service setup can disrupt shared sessions: adoption/restart must be deliberate and verified, never kill an unrelated daemon.

## Approval Boundary

Weekday/time-of-day scheduling and first-class Tailscale reachability are user requirements. Approved on 2026-09-26: IANA timezones, skip spring-forward gaps/use first fall-back occurrence, Tailscale Serve HTTPS plus browser pairing, Linux supervised provisioning, and retention/deadline defaults. Human-controlled prerequisites are Tailscale login, HTTPS readiness, network policy permitting intended devices, and permission to create a nonconflicting Serve endpoint. Their absence blocks remote provisioning, not implementation of the feature. Dependency selection and precise CLI spelling remain implementation details constrained by this contract.
