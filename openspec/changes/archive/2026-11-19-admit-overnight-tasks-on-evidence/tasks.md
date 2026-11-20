## 1. Implementation

- [x] 1.1 Add admission event types to `jcode-overnight-core`
- [x] 1.2 Gate the loop in `run_supervisor` on the admission decision
- [x] 1.3 Implement wait-for-signal using bg/swarm completion notifications plus wake boundary
- [x] 1.4 Use selected-route auth classification and existing strict deterministic task-admission policy; preserve original scope/owner/limits, cancellation and authoritative monetary reservations without probing other accounts or creating another admission engine.
- [x] 1.5 Tests: idle run issues no extra turns; completion signal admits at most one otherwise-authorized turn; selected-auth failure, unknown authority, unsupported monetary capability and unresolved predecessor all block without effects.

## 2. Verification

- [x] 2.1 `cargo test -p jcode-app-core` passes for the new tests
- [x] 2.2 `cargo clippy --workspace --all-targets -- -D warnings`
- [x] 2.3 `openspec validate admit-overnight-tasks-on-evidence --strict`

## Worker evidence, 2026-11-20

- 1.1: `crates/jcode-overnight-core/src/admission.rs` (`AdmissionVerdict`, `AdmissionEvent`, `TaskGate`, `decide_admission`). `scripts/dev_cargo.sh test -p jcode-overnight-core`: 27 passed, 0 failed (includes 4 admission tests).
- 1.2/1.3/1.4: `run_supervisor` in `overnight.rs` records an `admission_decision` event before each turn, waits in `SignalWatch::wait` (bus completion events for the coordinator session, queued user message, wake boundary, pending stop) when `waiting`, and blocks with `run_blocked` on a denying task gate or selected-route auth failure. Selected route only: `route_auth_block` resolves the session/provider key and checks that one `AuthStatus::state_for_provider`. Task policy goes through existing `revalidate_task_admission`/`ensure_task_dispatch`, so unsupported monetary capability and unresolved authority still deny.
- 1.5 checked after review fixes: supervisor-loop tests in `overnight/admission_gate.rs`: `idle_run_waits_and_one_completion_admits_exactly_one_turn`, `running_card_never_respins_and_a_changed_card_admits_exactly_one_turn` (M1), `bus_overflow_alone_is_not_evidence` (M2), `auth_failure_blocks_the_run_with_zero_turns_and_one_blocked_decision` (m2), `unclassified_route_blocks_without_a_turn` (m1), `cancel_requested_at_launch_wins_over_an_auth_block` and `block_run_revokes_and_drains_owned_workers_before_going_terminal`/`block_run_with_unsettled_workers_stays_non_terminal` (M4), `unknown_authority_unsupported_monetary_and_open_predecessor_dispatch_nothing` (live loop, zero provider calls), `unreadable_manifest_while_waiting_is_tolerated_then_cancel_still_stops` (m8), `wake_boundary_admits_the_morning_report_turn`, `task_gate_decision_denies_despite_signal_boundary_and_cards` (m5).
- 2.1: `JCODE_REMOTE_CARGO=0 scripts/dev_cargo.sh test -p jcode-app-core --lib -- --test-threads=1 overnight`: 65 passed, 0 failed.
- 2.3: `openspec validate admit-overnight-tasks-on-evidence --strict`: valid.
- 2.2 left unchecked: workspace clippy not run. Scoped `clippy -p jcode-overnight-core -p jcode-app-core --all-targets -- -D warnings` clean after fixing one unused assignment (see coordinator for workspace run).

- Review fixes: M1 `eligible_since` digests active cards, so an unchanged running card admits nothing; M2 bus `Lagged` returns `Wait::Rescan` and admission requires recorded evidence; M3 `gather_preflight` no longer calls `fetch_all_provider_usage` (usage is not queried at preflight, selected route auth only), which keeps 1.4 true; M4 `block_run` checks `pending_stop` first and settles owned work via `settle_owned_work` before a terminal `Blocked`. Checks: app-core `--lib -- --test-threads=1 overnight` 76 passed; overnight-core 27 passed; scoped clippy `-D warnings` clean; openspec validate valid. 2.2 (workspace clippy) still unchecked.
- Coordinator gate 2026-11-20: workspace clippy -D warnings no errors; overnight-core 27, app-core overnight 76 pass; fixes M1-M4 verified by gate only (no second independent review).
