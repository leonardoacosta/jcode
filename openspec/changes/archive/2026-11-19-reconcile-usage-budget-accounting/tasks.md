## 1. Implementation

- [x] 1.1 Add per-run usage aggregator keyed by run id
  - Evidence: `crates/jcode-app-core/src/overnight/usage.rs` (active-run registry, `usage.jsonl` ledger, idempotent `reconcile_run`) + pure `jcode-overnight-core/src/usage_reconcile.rs`. `scripts/dev_cargo.sh test -p jcode-app-core --lib overnight:: -- --test-threads=1` → 34 passed, 0 failed.
- [x] 1.2 Attribute worker/resumed-session usage to the run
  - Scope: resumed generations are attributed within one process only; the registry is process-local (see design.md).
  - Evidence: `owning_run` walks `session_control` owner chain (new `ControlStore::owned_records`); `observe_stream_usage` stamps `session_control::current_generation`. Hooks: `agent/streaming.rs` (stable `event_id`), `agent/provider.rs` (`note_session_account`). Test `worker_and_resumed_generations_attribute_to_run_and_replays_dedupe` + `cumulative_snapshots_of_one_response_share_an_event_id_and_retries_get_new_ones` pass.
  - Note: `OAuthUsageRecorder` (openai_usage_recording.rs) is untouched; attribution goes through the agent `StreamUsageRecorder`.
- [x] 1.3 Emit `usage_reconciled` and render with limit-bar
  - Evidence: `reconcile_run` writes `usage-reconciled.json` + `usage_reconciled` event; review HTML gains "Usage: projected versus observed" via `build_review_html_with_usage`. Limit bars use the shared `jcode_usage_types::limit_bar` (info widget refactored onto it; `jcode-tui --lib info_widget` 187 passed, `jcode-usage-types` 12 passed, `jcode-overnight-core` 16 passed).
- [x] 1.4 Tests: generation-bound worker/resume usage, same-event-id duplicate suppression (re-execution replays are NOT suppressed: no stable provider response id, see design.md), cancelled/unresolved work, provider/account separation and missing usage not zero. Verify accounting never grants pre-send budget authority.
  - Evidence: app-core `overnight::usage::tests::{worker_and_resumed_generations_attribute_to_run_and_replays_dedupe, cancelled_worker_stays_attributable_and_never_reads_as_zero, no_usage_is_estimated_not_zero_and_report_renders_it, accounting_does_not_change_dispatch_admission (dispatch decision identical before/after usage is recorded), corrupt_control_record_and_garbage_ledger_line_fail_closed, missing_coordinator_record_is_unsettled_lower_bound, reported_provider_renders_tokens_and_reported_badge_end_to_end}` all ok; provider/account separation and `None`-not-zero covered in `jcode-overnight-core` usage_reconcile tests (16 passed).

## 2. Verification

- [x] 2.1 `cargo test -p jcode-app-core` passes for the new tests
  - Evidence: `JCODE_REMOTE_CARGO=0 scripts/dev_cargo.sh test -p jcode-app-core --lib overnight:: -- --test-threads=1` → 37 passed, 0 failed; `... --lib cumulative_snapshots` → 1 passed; `-p jcode-overnight-core` → 16 passed. Scoped clippy on my 4 crates was clean before the review round; after it, `-p jcode-overnight-core -p jcode-app-core --all-targets` reports one diagnostic, in a file that is not mine (`server/background_tasks.rs:856`, MutexGuard held across await) and none in my files.
- [x] 2.2 `cargo clippy --workspace --all-targets -- -D warnings` (coordinator, integrated tree at 59510024e: exit 0)
  - Left to coordinator (workspace gate). Scoped clippy on my 4 crates is clean.
- [x] 2.3 `openspec validate reconcile-usage-budget-accounting --strict`
  - Evidence: "Change 'reconcile-usage-budget-accounting' is valid".

## Review round 1 (2026-11-19)
- M1: no stable provider response id exists on `StreamEvent::TokenUsage`; descoped and documented in design.md, 1.4 relabeled (re-execution replays are not suppressed).
- M2: fail-closed reconcile (unreadable control records, missing coordinator, skipped ledger lines) with tests.
- m1-m4 addressed (dispatch-admission test, 'stream responses only' note, badge assertions + end-to-end HTML test, restart limit in design.md/1.2).
