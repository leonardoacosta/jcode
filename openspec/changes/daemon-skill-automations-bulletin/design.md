## Context and Evidence

- `crates/jcode-app-core/src/ambient.rs`: `ScheduledItem` and `ScheduleTarget` represent one-shot tasks, not recurring definitions.
- `crates/jcode-app-core/src/ambient/persistence.rs` and `ambient/manager.rs`: persisted queue and direct-task dequeue. Do not implement recurrence by repeatedly calling the one-shot tool.
- `crates/jcode-app-core/src/ambient/runner.rs`: dispatches ready direct deliveries. Reuse execution conventions without requiring Ambient enablement.
- `crates/jcode-base/src/skill.rs`: `SkillRegistry::load_for_working_dir`, `get`, and `parse_invocation` provide project-aware skill resolution.
- `crates/jcode-app-core/src/server/headless.rs`: `create_headless_session` provides provider, directory, MCP, and memory setup. It is a private integration seam, not an already suitable public automation API.
- `crates/jcode-app-core/src/server/runtime.rs`: owns task cancellation and shutdown. `server/lifecycle.rs` explicitly applies idle shutdown to temporary servers.
- `crates/jcode-app-core/Cargo.toml`: Tokio, serde, chrono, UUID, and reqwest available. No direct general HTTP server dependency identified in inspected manifests. Select a maintained HTTP server during implementation if no existing reusable server exists.
- Existing changes cover Ambient AgentMail and overnight questions, not this capability. Shared server/agent files may conflict with concurrent work, but neither proposal is a functional dependency.

## Decisions

### One owner, separate recurring state

Add a daemon-owned automation manager, independent of Ambient enablement and attached to the existing shutdown boundary. Keep the one-shot queue unchanged. Store versioned automation definitions and bounded run records under the existing Jcode data directory, using atomic replacement and a single-writer lock. Temporary/test daemons do not acquire production automation ownership or listeners by default. Corruption or write failure blocks dispatch rather than silently replacing state.

Definition fields: stable ID, skill name, plain optional arguments, absolute working directory, interval seconds, enabled flag, creation time, next due time, selected provider/model reference. Secrets are never copied into definitions. Resolve the skill in that directory at creation and each run. Use current installed skill contents and record resolved path/content digest for traceability. Missing skill/directory pauses that automation with an actionable error.

### Fixed intervals and conservative recovery

Frequency accepts positive integer minute/hour/day units, minimum 60 seconds, with checked conversion and timestamp arithmetic. Store UTC due times, display localized timestamps with timezone labels. Creation/resume/frequency edit sets the next due time to now plus interval. Pausing does not kill an already running session. No delete or manual-run control in v1.

Dispatch the earliest due enabled automation when the global automation slot is free, ties by ID. Persist a running record and advance its due time to the first interval boundary strictly after the current time before starting execution. At startup, advance all overdue schedules into the future without catch-up. While an automation runs, coalesce its elapsed ticks into the next future boundary so it cannot immediately rerun for its own missed ticks. Other due automations wait for the slot without accumulating separate queued runs.

Mark orphaned running records interrupted after restart, never automatically replay them. This is deliberately not exactly-once execution across external side effects. Use a 30-minute run deadline, propagate cancellation, retain the slot until termination is acknowledged, and record timeout. Failures advance normally rather than creating retry storms. An unacknowledged cancellation blocks further dispatch and exposes degraded state.

### Existing runtime and permission boundaries

Run the normal skill invocation in a fresh headless session with the captured directory and configured provider/model. Do not inherit an interactive parent session or create swarms. Preserve normal tool permissions. A request needing fresh human authorization ends as blocked rather than hanging or granting approval. Persist session ID, start/end times, outcome, bounded final response, and sanitized failure reason. The final response is the bulletin entry, not a second model-generated summary.

### A single HTML page, not a frontend application stack

Serve the page and same-origin HTML fragments from one listener inside the daemon. Bundle pinned HTMX locally. Poll visible status/results every five seconds with bounded pages, preserve form focus/unsaved inputs, and provide ordinary form POST/redirect fallback. Render output as escaped text preserving whitespace, not executable HTML or raw Markdown HTML. Show automation skill/frequency/state/next run, run timestamps/duration/outcome, and expandable result text. First page shows newest 50 runs. Retain at most 1,000 terminal records and 64 KiB UTF-8-safe output per run, explicitly mark truncation, never prune active records. Existing session retention remains unchanged.

### Local security and bounded resources

Bind IPv4 loopback only. Provision a random local port once and persist it, expose the actual URL in CLI status. Reject unexpected Host/Origin, disable CORS, protect mutations with per-session CSRF tokens, use an HttpOnly SameSite=Strict browser cookie, and exchange a one-time CLI-issued bootstrap token for that cookie before redirecting to a clean URL. Never log bootstrap tokens or put them in HTML fragments. No arbitrary filesystem serving or unauthenticated result reads. Limit request bodies to 16 KiB and concurrent HTTP requests to 16. Set a restrictive CSP compatible with HTMX configured without eval. Mask known credentials in captured diagnostics/output while warning that skill output can still contain sensitive user data.

### Always-on means supervised daemon, not browser activity

Opt-in provisioning starts/enables one Linux user service supervising the existing daemon and its embedded listener. Reuse existing service/launcher mechanisms if discovered during implementation. Do not start a second daemon against the same socket/state. Reloads release and reacquire listener/state ownership. Bind failure leaves normal Jcode sessions and scheduling operational, with a visible degraded bulletin status.

User-systemd alone is insufficient after logout: provisioning checks linger, explains its effect, and requires explicit authorization before enabling it. If user systemd/linger is unavailable or denied, report that login-independent operation is not provisioned. Do not claim boot persistence. Other platforms may run daemon-lifetime mode, with the limitation stated. Uninstall stops/disables the managed service without deleting automation history or unrelated services. Disabling bulletin does not implicitly disable existing automation definitions, and this distinction must be explicit in controls/docs.

## Migration and Rollback

Feature disabled by default. Existing configs and schedule files load unchanged. Provisioning creates separate versioned state and service metadata only after authorization. Rollback pauses automations, disables the managed service/listener, and retains records. Unknown future state versions fail closed without rewriting them.

## Risks

Unattended skills can spend money or change files: explicit enablement, normal permissions, serial runs, deadlines, visible outcomes. Clock jumps can change observed cadence: UTC due times and no catch-up bursts. Disk exhaustion blocks new dispatch. Crash recovery cannot infer whether external side effects completed: interrupted outcomes, no automatic replay. Service setup can disrupt shared sessions: adoption/restart must be deliberate and verified, never kill an unrelated daemon.

## Approval Boundary

Fixed intervals rather than cron, loopback-only access, Linux supervised provisioning, and retention/deadline defaults are proposed product choices for approval, not claims that the user already selected them. No remaining external account prerequisite. Dependency selection and precise CLI spelling remain implementation details constrained by this contract.
