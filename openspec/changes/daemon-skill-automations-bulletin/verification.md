# Proposal Verification

Date: 2026-09-26. State: user approved revision `9635a9425` at 07:08:40 UTC. Implementation in progress, not ready for use.

## Implementation checkpoint

- Coordinator: chick. Current branch remains `codex/systemone-config-routing`; unrelated dirty changes are preserved.
- New code under `crates/jcode-app-core/src/automations/`, daemon integration under `server/automations.rs`, CLI under `src/cli/automations*.rs`. Dependencies: chrono-tz for IANA timezone rules, axum for maintained HTTP parsing, locally bundled HTMX 2.0.8 with license.
- Expected RED evidence: coordinated test `8663865ve4` failed for missing Schedule/Store, `9602194jto` failed for missing Config and foundation symbols before implementation.
- Integration checks `2853684wlu` and `45837886bp` failed on new-code compile errors. No passing implementation verification claimed. Security/lifecycle review found draft defects; fixes and regression tests remain in progress.
- No live service installation, linger changes, Tailscale route changes, automation enablement, or shared-daemon restart was performed by this feature work. External server reloads interrupted workers; source and tool evidence remain authoritative.
- Remaining acceptance: all unchecked tasks in tasks.md, including actual daemon skill execution, browser workflows, second-device access and service/logout gates. Do not archive or mark complete based on this checkpoint.

## Fresh evidence during implementation

- Final formatted-source check `288707poz9` PASSED: 22 automation library tests, one CLI authorization/parsing test, and `cargo check -p jcode --bin jcode`. Root test compilation required adding the missing `pending_question_tx: None` field in an existing selfdev test initializer, reflecting unrelated API drift.
- Fresh binary build `085596x1tl` PASSED. Real managed Chromium acceptance on isolated daemon PASSED pairing, native one-minute creation, edit entry, corrected weekday/time/timezone edit preserving the same definition, validation-error recovery, and pause persistence. At 390px width the document had no horizontal overflow. Screenshot: `scratch/automation-acceptance/mobile.png` (not committed).
- Runtime dispatched a due fixture into a real headless session and persisted a running record/session link. Successful final completion was NOT established with the local provider. The fixture was paused. This is not evidence of two successful unattended runs.

## Remaining work, not a completed feature

- Complete two successful unattended runs, crash/restart recovery against the real daemon, timeout/permission cancellation tests using a representative provider, and safe shutdown confirmation.
- Verify real Tailscale HTTPS from a second device, denied-device policy, logout/reboot service continuity, and owned-route cleanup. Live provisioning was not authorized or performed in this session.
- Close UI contract gaps: interval shorthand (`15m`, `1h`) is not yet exposed, edit currently relies on JavaScript, full keyboard-only/no-JavaScript workflows and polling/page-state behavior need acceptance tests. Pause persisted but the native 204 response left the existing button stale until refresh.
- Harden provisioning with concurrent-operation ownership tests, exact unrelated-route preservation after changes, process timeout descendants, and service-unit escaping verification. Status currently reports reachability, not full daemon/tailnet health.
- Do not archive this change or mark all tasks complete. Implementation is committed as an opt-in checkpoint, with no production service or network change.

- `135251v7kt`: automation library tests passed 20/20, including actual HTTP pairing/CSRF/origin isolation and durable schedule regressions. Subsequent CLI check in that command failed, then `21734265iw` passed `cargo check -p jcode --bin jcode` after correction.
- `2756252eud`: fresh TUI binary built successfully with the supported `scripts/dev_cargo.sh build --profile selfdev -p jcode --bin jcode` fallback. Two coordinated build requests aborted before execution during external reloads.
- Isolated daemon uses `scratch/automation-acceptance/home`, a separate `JCODE_RUNTIME_DIR`, and a distinct socket. Inherited named-provider environment needed clearing. No production daemon or service was modified. Live status confirmed loopback listener readiness.
- Owned managed Chromium session `automation-bulletin-chick`: unauthenticated page protected, one-time browser pairing succeeded after a browser-discovered referrer-policy correction. `no-referrer` caused native form POST Origin to become null in Chromium. Use `strict-origin` instead: preserve origin validation while excluding paths/query tokens from referrers. This is a security-compatible implementation adjustment, not relaxed CSRF validation.
- Browser inspection found an incorrect 900-second HTML minimum and stale page-level referrer override. Source fixes restore the approved 60-second minimum and consistent strict-origin policy. Full fresh-binary workflow retest remains required.
- Foundation checkpoint committed as `f3cf9f764`. Remaining integration is not complete, and no real tailnet, service/logout, or unattended successful model-run acceptance has passed yet.

Revision incorporates user-required weekday/time-of-day schedules and first-class Tailscale access. Supersedes the earlier interval-only/local-only scope.

- Source evidence: inspected `crates/jcode-base/src/gateway.rs` for MagicDNS discovery and existing remote access. Its wildcard WebSocket transport is not reused as a private HTTP security model.
- Dependency evidence: no `chrono-tz` in inspected Cargo.lock. IANA-aware calendar implementation requires library selection, not fixed-offset arithmetic.
- Scenario mapping: tasks 1.1–1.3 cover storage, both schedule types, DST, clock changes and recovery. Tasks 2.1–2.2 cover execution/outcomes. Tasks 3.1–3.3 cover authenticated HTML, schedule controls and accessibility. Task 3.4 covers private Serve provisioning, remote authentication and failure boundaries. Tasks 4.1–4.2 cover unattended operation and actual second-device/network-boundary acceptance.
- Dependency review: all task dependencies refer to defined earlier tasks and remain acyclic. User approval clears the implementation gate. Live service/network provisioning retains its explicit authorization boundaries.
- PASS: `openspec validate daemon-skill-automations-bulletin --strict` and `git diff --check -- openspec/changes/daemon-skill-automations-bulletin` completed successfully after this revision.
- NOT RUN: compilation, runtime, browser, service, Tailscale provisioning, second-device, logout, or DST implementation checks. This is an artifact revision only.

Approved decisions: skip nonexistent DST times, use the earlier repeated time, private Tailscale Serve HTTPS plus app browser pairing, one calendar time per definition, existing serial execution/retention/deadline defaults. Human-controlled Tailscale login, HTTPS configuration, network policy and endpoint permissions remain explicit provisioning prerequisites. Their absence must block remote readiness claims.
