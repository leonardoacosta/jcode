## 1. Implementation

- [x] 1.1 Add cancellation select branch and abort handling in `run_turn_monitored`
  - Evidence: `overnight::enforce::tests::cancel_interrupts_a_running_turn_then_settles_and_blocks_later_admission` (hung provider stream interrupted, passes).
- [x] 1.2 Track generation-owned bg/swarm work on cancel using durable control; verify draining/unresolved versus confirmed stopped, no later admission and no false success from a cancellation deadline.
  - Evidence: `unsettled_owned_work_keeps_cancel_unresolved_not_cancelled` (CancelRequested + Unresolved, no run_cancelled) and the settled path above (Cancelled).
- [x] 1.3 Add `max_agents`, `max_turn_minutes`, `max_run_tokens` manifest fields and enforcement
  - Evidence: jcode-overnight-core 23 tests, `turn_time_limit_stops_a_long_turn`, `worker_spawn_beyond_cap_is_refused_with_a_visible_reason`; flags `--max-agents/--max-turn-minutes/--max-tokens`.
- [x] 1.4 Tests: cancel revokes admission promptly then settles or reports unresolved; spawn beyond cap refused; aggregate token cap includes deduplicated child/resume usage and distinguishes estimates. Verify token caps cannot bypass authoritative monetary dispatch denial.
  - Evidence: `token_cap_counts_deduplicated_child_and_resumed_usage_and_labels_estimates`, `estimated_token_usage_is_a_labelled_lower_bound`, `unavailable_usage_never_claims_an_exact_breach`, `within_limits_produces_no_stop_and_limits_never_authorize_dispatch` (spend enforcer still denies).

## 2. Verification

- [x] 2.1 `cargo test -p jcode-app-core` passes for the new tests
  - Evidence: `scripts/dev_cargo.sh test -p jcode-app-core --lib overnight:: -- --test-threads=1` 45 passed; jcode-tui overnight 27 passed; clippy -p jcode-overnight-core -p jcode-app-core --all-targets -D warnings clean.
- [x] 2.2 `cargo clippy --workspace --all-targets -- -D warnings`
- [x] 2.3 `openspec validate enforce-overnight-cancellation-and-limits --strict`
  - Evidence: `openspec validate enforce-overnight-cancellation-and-limits --strict` valid.

## Review follow-up (2026-11-19, independent review CHANGES REQUESTED, 5 major / 7 minor)

Fixes applied (evidence = tests in `overnight::enforce::tests`, `overnight::tests`, `server::comm_session::comm_session_tests`):
- MAJOR-1 visible/current-session runs have no supervisor, so `--max-turn-minutes` / `--max-tokens` are REJECTED there (`visible_mode_limit_rejection`, `start_overnight_run` bails; tests `visible_mode_rejects_unenforceable_limits_but_allows_max_agents`, `starting_a_visible_run_with_unenforceable_limits_fails_before_creating_it`). `--max-agents` still applies via the spawn gate. TUI help and cancel text are honest per mode (`is_supervised`).
- MAJOR-2 `request_cancel` returns `CancelOutcome::Rejected` for terminal runs (manifest untouched); TUI and server print "already <status>; unchanged" (test `cancelling_a_finished_run_is_rejected_and_leaves_the_manifest_unchanged`, Completed/Failed/Cancelled bytes unchanged, no event written).
- MAJOR-3 `stop_until_settled` re-settles up to 5 times (60s apart) after an Unresolved stop; a repeated `/overnight cancel` of a supervised run with no live supervisor in-process re-runs settlement in the background (`resettle_in_background`). Usage is finalized only when Settled (tests `unresolved_cancel_is_re_settled_by_the_supervisor_then_finalizes_usage`, `repeated_cancel_of_an_unresolved_run_re_runs_settlement`).
- MAJOR-4 `settle_owned_work` takes the spawn gate once, then re-enumerates `owned_records` to a fixpoint; a Limit breach is persisted to the manifest before revocation and makes `ensure_worker_spawn_allowed` refuse (tests `settle_re_enumerates_owned_work_registered_during_the_drain`, `limit_stop_refuses_new_spawns_and_unresolved_work_stays_non_terminal`).
- MAJOR-5 real tests: `overnight_max_agents_cap_admits_exactly_one_of_two_concurrent_spawns` drives `spawn_swarm_agent`; `within_limits_run_has_no_limit_events_and_limits_never_authorize_dispatch` runs `run_turn_monitored` with a huge cap, asserts no limit events, then monetary denial; `run_turn_monitored_interrupts_a_hung_turn_on_the_time_limit` (cfg(test) `TURN_TIME_SCALE`: one limit minute = one second) asserts Limit cause, persisted `limit_breach`, Failed.
- MINOR-1 event renamed to `limit_reached` (matches proposal), emitted once when the breach is persisted; `event_class` colours it `warn`. Terminal event stays `run_limit_stopped`/`run_limit_unresolved`.
- MINOR-2 Limit stop with unresolved work stays non-terminal (`run_limit_unresolved`, status unchanged, no `completed_at`).
- MINOR-3 unreadable manifest during the poll records `manifest_unreadable` after 5 consecutive failures (test `unreadable_manifest_is_recorded_after_repeated_poll_failures`).
- MINOR-7 TUI `/overnight` help lists the limit flags and honest cancel semantics; explicit old-manifest (`old_manifest_without_limit_fields_loads_with_no_limits`) and cancelled round-trip (`cancelled_manifest_round_trips`) tests.

Accepted limits (not changed):
- MINOR-4 `governing_run` scans manifests for runs not registered in-process, from async code, twice per governed spawn; it now only does so after the active-run lookup misses and skips dead supervisors. The global `SPAWN_GATE` serializes governed spawns across unrelated runs (low impact).
- MINOR-5 the cap counts direct children of the run's coordinator only (`report_back_to_session_id == coordinator`). Helpers spawning helpers are not governed; matches the proposal wording "from the run's coordinator".
- MINOR-6 the token limit breaches at equality (`tokens >= limit`, message says "reached"), and the full ledger is re-read and deduplicated on each 1s poll while a token cap is set.

Re-verification after follow-up (2026-11-19): `scripts/dev_cargo.sh test -p jcode-overnight-core` 23 passed; `test -p jcode-app-core --lib -- --test-threads=1 overnight comm_session` 103 passed; `test -p jcode-tui --lib -- --test-threads=1 overnight` 27 passed; `clippy -p jcode-overnight-core -p jcode-app-core -p jcode-tui --all-targets -- -D warnings` clean; `openspec validate ... --strict` valid. 2.2 (workspace-wide clippy) left unchecked for the coordinator.

## Re-review follow-up 2 (2026-11-19, NEW-1 + minors)

- NEW-1 (major) fixed: a cancelled visible/current-session run (parent == coordinator, no supervisor) stays `CancelRequested` but `governing_run` no longer returns it (`is_supervised || !cancel_pending`), so the session's worker spawns and `--max-agents` cap are no longer refused/applied after Esc until the TUI exits. Supervised cancelled runs still refuse spawns. TUI cancel text, interrupt note and `/overnight` help now say the visible run drains nothing, stays 'cancel requested' and its worker cap no longer applies. Test: `cancelled_visible_run_no_longer_governs_or_blocks_worker_spawns`.
- m1 fixed: `stop_until_settled` re-reads `cancel_pending` before each re-settle and switches cause from `Limit` to `Cancel`. Test: `user_cancel_during_an_unresolved_limit_stop_switches_to_cancel`.
- m4 fixed: `spawn_supervisor` error path goes through `record_supervisor_failure`, which leaves the run non-terminal (no `Failed`/`completed_at`, no usage finalize, event `run_failed_unresolved`) when `stop_settlement` is `Unresolved`. Test: `supervisor_error_with_unresolved_owned_work_leaves_the_run_non_terminal`.
- Accepted limits (not changed): m2 `resettle_in_background` can race a supervisor still in preflight (two idempotent settles, duplicate events); m3 re-settle has no `register_run`, so worker usage in that window is not ledgered (finalize uses what exists); m5 a persistently unreadable manifest is recorded as `manifest_unreadable` but the poll still fails open (cancel/limits unobserved).
- Re-verification (follow-up 2): `test -p jcode-overnight-core` 23 passed; `test -p jcode-app-core --lib -- --test-threads=1 overnight comm_session` 106 passed; `test -p jcode-tui --lib -- --test-threads=1 overnight` 27 passed; `clippy -p jcode-overnight-core -p jcode-app-core -p jcode-tui --all-targets -- -D warnings` clean; `openspec validate ... --strict` valid; `rustfmt --edition 2024 --check` on owned files clean. 2.2 left unchecked.
