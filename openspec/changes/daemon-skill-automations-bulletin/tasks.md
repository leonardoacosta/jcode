## Execution Gate

All tasks are blocked on explicit proposal approval. Do not install services or modify implementation while this change is a draft. Check concurrent changes before editing shared files, preserve unrelated work.

## 1. Durable definitions and recurrence

- [ ] 1.1 After approval, add versioned definitions/run records and atomic single-writer persistence using existing storage conventions. Verify restore, corrupt/unknown-version preservation, write failure, and duplicate-owner exclusion using isolated temporary data.
- [ ] 1.2 Depends on 1.1. Implement skill/directory validation and checked interval parsing plus IANA-zone weekday/time schedules, create/pause/resume/schedule-edit operations. Verify valid units, invalid values, overflow, project-local skill precedence, weekday selection, time/zone validation, next-three preview, DST gaps/folds, host-zone independence, next-due rules, and unchanged one-shot config/queue behavior.
- [ ] 1.3 Depends on 1.2. Add daemon-owned serial recurrence and persisted pre-execution claims, independent of Ambient. Use deterministic clock tests for ties, waiting definitions, overlap, long-running coalescence, downtime, forward/backward clock jumps, and interrupted-run recovery.

## 2. Skill execution and run outcomes

- [ ] 2.1 Depends on 1.3. Integrate existing headless session and skill invocation paths with captured directory/provider/model and normal permission enforcement. Test success, missing skill/directory, missing credentials, provider errors, human-approval blocking, and no interactive-parent dependency.
- [ ] 2.2 Depends on 2.1. Add deadline/cancellation handling and bounded final-output persistence, source identity, and session linkage. Verify acknowledged/unacknowledged timeout, shutdown interruption, UTF-8 truncation, credential masking, retention excluding active runs, and absence of extra summary requests.

## 3. Embedded bulletin

- [ ] 3.1 Depends on 1.2. Confirm reusable HTTP infrastructure before selecting a maintained server dependency, then implement loopback upstream and explicit local/tailnet origins, local pinned HTMX, expiring one-time bootstrap, revocable browser sessions, local/remote cookie separation, CSRF, Host/Origin checks, CSP, and request limits. Verify unauthorized reads, token reuse, forged mutations, oversized requests, concurrency bounds, occupied port, shutdown/reload, and temporary-daemon defaults with real HTTP requests.
- [ ] 3.2 Depends on 2.2 and 3.1. Implement one server-rendered page and fragments for controls, live status, paginated results, expansion, and plain-form fallback. Test validation preserves inputs, no-results state, disconnected refresh, hostile HTML/HTMX output, truncation labels, newest-first order, and focus preservation.
- [ ] 3.3 Depends on 3.2. Exercise actual browser workflows at desktop and 390px width, keyboard-only and JavaScript-disabled. Verify create/pause/resume/interval-calendar schedule edits, five-second polling with results within ten seconds, pagination, and no credential/bootstrap leakage into URLs after redirect or server logs.

- [ ] 3.4 Depends on 3.1. Verify official supported Tailscale Serve HTTPS behavior and host forwarding before implementing managed endpoint provisioning, status, recovery, and scoped rollback. Reuse existing MagicDNS discovery where suitable. Test missing installation/login/HTTPS, conflicts, denied privileges, untrusted forwarded/identity headers, token/session expiry/revocation, preserved unrelated routes, and no Funnel or wildcard fallback. Never change login or tailnet policy automatically.

## 4. Always-on provisioning and integrated verification

- [ ] 4.1 Depends on 2.2 and 3.4. Add explicit CLI provision/open/status/disable or equivalent existing command extensions. Provision one Linux user-systemd service through supported launcher/socket ownership. Explain and obtain authorization for linger, handle unavailable/denied setup honestly, and implement nondestructive service uninstall. Verify in an isolated user-systemd environment, never the live shared daemon.
- [ ] 4.2 Depends on 3.3 and 4.1. Run the freshly built binary with isolated data and socket. Create a harmless fixture skill at a one-minute interval, close TUI/browser, observe two completed run records, reconnect to see them, restart daemon, verify no replay and future recurrence. Verify logout availability in a disposable login session when linger is approved. From a second authorized tailnet device, pair and exercise every bulletin control, including calendar schedules, results, and polling over HTTPS. Verify blocked-device and outside-tailnet denial, remote outage/local continuity, same URL recovery after daemon/Tailscale restart, and scoped route removal. If host or tailnet prerequisites prevent service/logout/second-device tests, record them as blocked, not passed.
- [ ] 4.3 Depends on 4.2. Document exact implemented commands, interval/calendar semantics, DST behavior, IANA timezone handling, Tailscale HTTPS prerequisites and pairing/revocation, owned-route cleanup, costs/permissions, sensitive-output limits, retention, blocked/interrupted outcomes, Linux linger requirements, daemon-only availability elsewhere, and rollback. Run focused Rust tests, relevant CLI regressions, formatting, and strict change validation. Record exact commands and outcomes in verification.md, preserving an explicit scenario-to-evidence checklist. Commit only scoped changes after passing checks.
