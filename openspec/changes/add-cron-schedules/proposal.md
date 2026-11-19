---
execution:
  version: 1
  depends_on: [recover-missed-scheduled-runs, keep-shared-daemon-resident]
---
Roadmap owner: [P05: Durable overnight, scheduling and recovery](../../../docs/plans/PRODUCT_ROADMAP.md#p05-durable-overnight-scheduling-and-recovery)

## Why

The current schedule tool accepts one `wake_in_minutes` or UTC `wake_at` timestamp and persists it as a one-shot `ScheduledItem` (`crates/jcode-app-core/src/tool/ambient.rs:286-298,337-365`). The queue is persisted and transactionally reloaded under a file lock, but its ready-item path removes due work before delivery (`crates/jcode-app-core/src/ambient/persistence.rs:76-104,129-159`). Existing recovery owns that delivery boundary; it does not define recurring occurrences, timezone rules, or bounded execution across recurrence. The shared ambient runner already reads this queue and delivers ready work (`crates/jcode-app-core/src/ambient/runner.rs:678-713`), so recurrence should extend this path rather than add another scheduler or daemon.

## What Changes

- Extend the existing schedule tool and ambient queue with named recurring cron schedules, explicit IANA time zones, persisted lifecycle state, and occurrence-level idempotency keys.
- Define deterministic DST behavior: skip nonexistent local times in a spring-forward gap; run an ambiguous fall-back local time once at its earlier UTC instant. A local calendar occurrence has one stable key regardless of restart or clock movement.
- Support explicit missed-run policies: `skip`, `run-once` (coalesce all missed occurrences into the latest due occurrence), and bounded `catch-up` (run oldest missed occurrences up to a configured per-schedule limit and mark the remainder skipped). Evaluate occurrences by absolute instants and persisted occurrence keys so forward/backward clock changes do not duplicate or erase already-recorded work.
- Add pause, resume, list, and side-effect-free next-run preview operations. Persist schedule definitions, next occurrence, occurrence/run state, limits and control changes across restart.
- Enforce per-schedule and global active-run limits through the shared ambient runner. Add per-schedule spend/token budgets using the existing overnight usage ownership and limit contracts where available; do not treat estimates or post-request reconciliation as authoritative monetary reservation.
- Reuse `recover-missed-scheduled-runs` for durable claim/acknowledgment and uncertain-delivery behavior, and `keep-shared-daemon-resident` for server lifecycle. Do not add a parallel daemon, queue, or scheduler service.

## Capabilities

### New Capabilities

- `cron-schedules`: recurring schedule definitions, occurrence calculation, controls, bounded dispatch, and usage limits on the existing ambient path.

### Modified Capabilities

None. The one-shot recovery contract remains owned by `scheduled-run-recovery`.

## Impact

Primary surfaces: `crates/jcode-app-core/src/tool/ambient.rs`, `crates/jcode-app-core/src/ambient/persistence.rs`, `crates/jcode-app-core/src/ambient/runner.rs`, and their existing app-core tests. Reuse the existing queue and runner, and the existing overnight task admission, cancellation, usage ownership, and budget enforcement contracts. An inventory task must identify actual P06 capability names and supported hard-cap inputs before wiring budget enforcement; do not claim hard spend caps from reporting-only usage data.

## Non-goals

No new daemon, scheduler framework, persistence store, general event trigger, arbitrary user script execution, background task ledger, UI redesign, billing conversion, or automatic retry of uncertain deliveries. This change does not authorize unattended execution beyond the existing task admission and permission policy. It does not claim exactly-once execution of agent effects: occurrence idempotency prevents duplicate scheduling/claim identity, while recovery records uncertain delivery and requires explicit retry when acknowledgment is missing. No budget enforcement may use estimated spend as if it were an authoritative charge reservation.
