# Proposal Verification

Date: 2026-09-26. State: user approved revision `9635a9425` at 07:08:40 UTC, implementation not started.

Revision incorporates user-required weekday/time-of-day schedules and first-class Tailscale access. Supersedes the earlier interval-only/local-only scope.

- Source evidence: inspected `crates/jcode-base/src/gateway.rs` for MagicDNS discovery and existing remote access. Its wildcard WebSocket transport is not reused as a private HTTP security model.
- Dependency evidence: no `chrono-tz` in inspected Cargo.lock. IANA-aware calendar implementation requires library selection, not fixed-offset arithmetic.
- Scenario mapping: tasks 1.1–1.3 cover storage, both schedule types, DST, clock changes and recovery. Tasks 2.1–2.2 cover execution/outcomes. Tasks 3.1–3.3 cover authenticated HTML, schedule controls and accessibility. Task 3.4 covers private Serve provisioning, remote authentication and failure boundaries. Tasks 4.1–4.2 cover unattended operation and actual second-device/network-boundary acceptance.
- Dependency review: all task dependencies refer to defined earlier tasks and remain acyclic. User approval clears the implementation gate. Live service/network provisioning retains its explicit authorization boundaries.
- PASS: `openspec validate daemon-skill-automations-bulletin --strict` and `git diff --check -- openspec/changes/daemon-skill-automations-bulletin` completed successfully after this revision.
- NOT RUN: compilation, runtime, browser, service, Tailscale provisioning, second-device, logout, or DST implementation checks. This is an artifact revision only.

Approved decisions: skip nonexistent DST times, use the earlier repeated time, private Tailscale Serve HTTPS plus app browser pairing, one calendar time per definition, existing serial execution/retention/deadline defaults. Human-controlled Tailscale login, HTTPS configuration, network policy and endpoint permissions remain explicit provisioning prerequisites. Their absence must block remote readiness claims.
