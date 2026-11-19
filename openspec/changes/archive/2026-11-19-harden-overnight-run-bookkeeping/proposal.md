---
execution:
  version: 1
  depends_on: []
---
Roadmap owner: [P05: Durable overnight, scheduling and recovery](../../../docs/plans/PRODUCT_ROADMAP.md#p05-durable-overnight-scheduling-and-recovery)

Related, not prerequisites: `reconcile-overnight-runs-after-restart` (auto-resume policy), `enforce-overnight-cancellation-and-limits` (mid-turn cancel, hard caps). This change is the bookkeeping foundation those changes read from; it never resumes, cancels or re-admits work.

## Why

Evidence from `~/.jcode/overnight/runs` at commit `52626ede5` (152 manifests): 61 are stuck `running` (oldest 2026-10-21, supervisor PID gone), 2 carry status `blocked` that `OvernightRunStatus` cannot deserialize, and events across all runs are ~151 `run_started` out of ~160 total. Causes in code:

- `latest_manifest()` (`crates/jcode-app-core/src/overnight.rs:808-826`) uses `?` per manifest, so one unreadable or unknown-status manifest fails `/overnight status`, `/overnight log`, `/overnight review`, `cancel_latest_run()`, the TUI card (`state_ui_messages.rs:518`) and silently returns `false` from `schedule_overnight_poke_followup_if_needed` (`commands_overnight.rs:325-328`), ending auto-poke without a trace.
- Every TUI auto-poke exit (`commands_overnight.rs:229,256,305` and the early returns in `313-414`) drops `overnight_auto_poke` without writing a manifest status or event, so the run stays `running` forever.
- The TUI poke loop writes no events at all; only the headless supervisor does.
- `process_id` is stored (`jcode-overnight-core/src/lib.rs:50`) but no reader checks liveness, so a dead run and a live run render identically.
- The server start path refuses a second active run per session (`runtime_special_commands.rs:324-333`); the TUI start path (`commands_overnight.rs:26-60`) does not, and two runs 45 s apart from one session were observed.

## What Changes

- Manifest reads become tolerant: `latest_manifest()` skips manifests that fail to parse (logging each skipped path) instead of failing the whole listing; `OvernightRunStatus` gains `Blocked` and a serde `#[serde(other)] Unknown` variant so legacy files render instead of erroring. `load_manifest(run_id)` for an explicit ID keeps failing loudly; `admission::read_manifest_at` is unchanged for authority paths.
- Every TUI auto-poke exit records one `auto_poke_stopped` event with the reason (`cancelled`, `final_done`, `stalled`, `poke_budget`, `non_retryable_error`, `run_mismatch`, `manifest_unreadable`, `provider_failover`, `guardrail_stop`, `retry_budget_exhausted`, `auth_failed`; `run_finished` when the run was already terminal) and, when the status is still `Running`, writes `Completed` (for `final_done`) or `Failed` with `completed_at` set; the reason lives in the event summary. Cancel keeps the existing `CancelRequested` path.
- The TUI poke loop records `auto_poke_sent` (phase, stalled turn count) per poke and `auto_poke_turn_error` on turn errors it observes.
- Readers derive a `stale` indicator: a `Running`/`CancelRequested` manifest whose `process_id` is not alive renders as `running (stale, supervisor pid N gone)` in `format_status_markdown` and the TUI card. Derivation only; the manifest file is not rewritten by status views.
- TUI `/overnight <hours>` refuses to start when the latest manifest for the same coordinator session is `Running`/`CancelRequested` and its `process_id` is alive, with the same message as the server path.

## Non-scope

- Automatic resume, re-admission or cancellation of stale runs (`reconcile-overnight-runs-after-restart`).
- Mid-turn cancellation, `max_agents`, time or token caps (`enforce-overnight-cancellation-and-limits`).
- Rewriting historical manifests on disk. A one-off migration is not part of this change.
- Desktop app surfaces.

## Capabilities

### New Capabilities

- `overnight-run-bookkeeping`

## Impact

- `crates/jcode-overnight-core/src/lib.rs`: `OvernightRunStatus` variants and `label()`.
- `crates/jcode-app-core/src/overnight.rs`: `latest_manifest`, `format_status_markdown`, new `supervisor_is_alive(&OvernightManifest) -> bool` reusing the `libc::kill(pid, 0)` pattern from `crates/jcode-storage/src/active_pids.rs:219-225`.
- `crates/jcode-tui/src/tui/app/commands_overnight.rs`: auto-poke exit finalization, per-poke events, start guard.
- `crates/jcode-tui/src/tui/app/state_ui_messages.rs`: card stale label.
- Tests in `overnight.rs` tests module, `admission.rs` tests (update the `latest_manifest().is_err()` assertion at `admission.rs:741` to the new tolerant contract) and `commands_overnight.rs` tests.

## Resolved defaults

1. Unknown status rendering: serde `other` → `Unknown`, label `unknown`. Treated as terminal by every `matches!(Running | CancelRequested)` guard so it never blocks a new start.
2. Finalization status on non-success exit: `Failed`, not a new variant, so existing counters and views need no new branch beyond `Blocked`/`Unknown`. `cancel_latest_run()` treats `Blocked`/`Unknown` as terminal like `Completed`/`Failed`.
3. Stale detection: `kill(pid, 0)` on unix; non-unix returns alive when `pid != 0` (same fallback as `active_pids.rs:228-233`).
4. Ownership: one worker lane per crate boundary (`overnight-core` + `app-core`, then `tui`) because the TUI lane depends on the new status variants and helper.
