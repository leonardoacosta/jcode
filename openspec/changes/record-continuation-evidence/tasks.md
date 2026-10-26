# Tasks: record continuation evidence

This is the sole executable checklist for this change. All tasks are unimplemented. Read [proposal.md](proposal.md), [design.md](design.md) and the [delta spec](specs/continuation-evidence/spec.md) before execution. Proposal authoring does not authorize implementation, Jev inference, external publication or a shared-daemon restart.

## Execution prerequisites and recipes

| Prerequisite | Exact target / safe configuration | Authoring evidence and dependent work |
| --- | --- | --- |
| Native authoring CLI | OpenSpec 1.13.2, current checkout, `spec-driven` | CLI/root/schema observed; proposal validation is separate from code verification |
| Rust toolchain and dependencies | Repository workspace/lockfile, no new package or network requirement | Implementation builds and cache availability not exercised here; missing dependencies block build tasks, not source drafting |
| Test coordination | Run Cargo through `selfdev` action `test`; use its compile lock | Do not bypass a busy shared compile or run a release/signoff build |
| Runtime data | Existing isolated test-home helpers, synthetic session/call IDs, temporary files only | Do not enable capture in the real user profile or read private transcripts |
| Providers / credentials | None for initial acceptance; replay uses real local controllers and synthetic native events | No API key, OAuth change, live Jev route or paid call is required |
| Platform privacy | Unix private-directory/file tests; other capture platforms explicitly unavailable until verified | Do not change existing permissions or claim Windows capture passed from Unix results |

Use these exact Cargo commands as the `command` value of the native `selfdev` test tool. `--offline --locked` prevents dependency downloads; report a missing cache as a prerequisite blocker, not a passing check. Run each command only after its tests exist, and confirm the named tests appear with `-- --list` before treating a filtered pass as evidence.

```json
{"action":"test","command":"cargo test --offline --locked -p jcode-todo-policy continuation_ -- --nocapture","intent":"Verify the continuation receipt and replay contract"}
```

Additional commands used below:

```bash
cargo test --offline --locked -p jcode-base continuation_trace -- --nocapture
cargo test --offline --locked -p jcode-tui continuation_ -- --nocapture
cargo test --offline --locked -p jcode-tui auto_poke -- --nocapture
cargo test --offline --locked -p jcode-sdk continuation_ -- --nocapture
cargo test --offline --locked -p jcode-todo-policy
cargo check --offline --locked -p jcode-tui -p jcode-sdk
cargo fmt --all -- --check
```

For every code slice: add its failing named tests, demonstrate the failure, implement the smallest change, rerun its checks, inspect the scoped diff and commit only verified owned files with the configured identity. Never fix or stage the unrelated MCP/swarm changes as a prerequisite shortcut. Preserve final review and archive requirements from the native execution workflow.

## Requirement and scenario coverage

The names below match the delta exactly. Each task includes its own tests; this table is a map, not a second checklist.

| Requirement | Scenarios covered | Task slices |
| --- | --- | --- |
| Versioned provenance | Available native provenance; Stale or legacy identity; Success is not acceptance | 1.1-1.3, 4.2-4.3 |
| Exact engine inputs | Stateful policy capture; Scheduler no-op capture; Missing context | 1.1-1.3, 2.1-2.2, 4.1-4.4 |
| Noninterfering capture | Capture parity; Effects without follow-up; Capture failure isolation | 2.1-2.3, 3.4, 4.1-4.4, 6.2 |
| Private opt-in storage | Disabled capture; Private enabled trace; Unsafe storage boundary | 3.1-3.4, 6.2 |
| Bounded trace integrity | Retention boundary; Oversized input; Corrupt or unknown trace | 3.1, 3.3-3.4 |
| Isolated engine replay | Shared policy match; TUI adapter boundary; Untrusted replay input | 2.1-2.3, 4.4, 5.3 |
| Sanitized baseline coverage | Repetition and necessary check; Waiting and cancellation; Error and identity controls; Incomplete baseline controls | 5.1-5.2 |
| Observation-only claims | Baseline without measurement; Unmatched comparison | 1.3, 5.2, 6.1-6.3 |

## 1. Versioned receipt and fixture contract

Files: create `crates/jcode-todo-policy/src/continuation.rs` and `crates/jcode-todo-policy/src/continuation_tests.rs`; modify `crates/jcode-todo-policy/src/lib.rs` to export the module and wire its tests. Follow existing `followup.rs`/`followup_tests.rs` patterns.

- [ ] 1.1 Add failing `continuation_receipt_metadata_only`, `continuation_fixture_unknown_version`, `continuation_claim_is_not_observation` and `continuation_receipt_redacted_replay_is_incomplete` tests using the two distinct v1 types defined in design.md; run the policy `continuation_` recipe and retain the expected missing-contract failure.
- [ ] 1.2 Implement `ContinuationReceiptV1`, `ContinuationReplayFixtureV1`, controller/provenance/coverage/effect/result types and strict validation. The runtime type must not have fields for raw task/goal/plan text, messages, fingerprints, tool payloads or command arguments. Verify all four tests pass and unknown/invalid input never becomes `matched`.
- [ ] 1.3 Add a fixture with distinct sentinel strings in todo text, goal text, message, fingerprint, tool output, arguments and error prose; assert no sentinel appears in serialized production metadata. Document the fixture-only versus production-type boundary in module docs and verify through `continuation_receipt_metadata_only`.

## 2. Pure policy state and offline replay

Files: modify `crates/jcode-todo-policy/src/followup.rs`, `crates/jcode-todo-policy/src/continuation.rs` and `crates/jcode-todo-policy/src/continuation_tests.rs`. No filesystem/network code belongs in the pure policy.

- [ ] 2.1 Add failing `continuation_policy_replay_matches`, `continuation_policy_replay_state_is_required`, `continuation_policy_capture_does_not_ack` and `continuation_policy_tui_engine_is_unsupported` tests. Use complete synthetic state for the nine actual policy fields, explicit no-follow-up with effects, and a mismatched expected decision; verify the policy recipe fails before implementation.
- [ ] 2.2 Implement read-only `PolicyReplayStateV1` projection, a crate-private fresh-instance replay constructor and `replay_policy_fixture` using the real `FollowUpPolicy::decide` once. Never load a trace into a live controller. Verify exact kind/message/requested-effect equality, `mismatched` for differing expectations and explicit incomplete/unsupported/invalid results.
- [ ] 2.3 Compare trace off/on on identical fresh engine states, including disabled, open work, digest, gate budget, final response, reset and no-op cases. Assert unchanged decision/messages/state/effects and zero acknowledgement/I/O/provider calls; run the full `jcode-todo-policy` suite and document engine-specific limits with the tests.

## 3. Private bounded local capture and default-off configuration

Files: modify `crates/jcode-todo-policy/src/store.rs`, `crates/jcode-base/src/todo.rs` (including its focused test boundary), `crates/jcode-config-types/src/lib.rs`, `crates/jcode-base/src/config/default_file.rs` and `crates/jcode-base/src/config_tests.rs`. Do not add a database, user command or migration.

- [ ] 3.1 Add failing `continuation_trace_disabled_is_no_io`, `continuation_trace_private_path`, `continuation_trace_limits`, `continuation_trace_unsafe_boundary` and `continuation_trace_corrupt_file` tests in the existing storage/config test boundaries. Use temporary homes and verify the `jcode-base` continuation-trace recipe fails before implementation.
- [ ] 3.2 Add `features.continuation_trace`, default false, and document its metadata-only/local nature in the existing generated-default surface. Implement the safe-session path under `todos/continuation-traces/`, strict metadata-only serialization, Unix 0700/0600 creation and bounded atomic read/write. Verify absent configuration and disabled capture create no trace files and existing directories/data are unchanged.
- [ ] 3.3 Enforce at most 64 records and 512 KiB for the entire serialized file, with bounded reads and coverage/drop metadata. Exercise the 65th record, byte-cap pressure and a single oversized record; verify oldest retention/omission is explicit, output remains within both caps and no incomplete record is replayed successfully.
- [ ] 3.4 Reject unsafe IDs, symlink destinations, unavailable ownership/privacy, corrupt data and unsupported platform capture without repair or raw diagnostics. Assert capture failure leaves original files and requested policy effects unchanged; rerun the base recipe and config tests touched by 3.2. Document unsupported-platform behavior rather than widening ACLs.

## 4. Actual TUI observation and test-only replay

Files: modify `crates/jcode-tui/src/tui/app/input.rs`, `crates/jcode-tui/src/tui/app.rs`, `crates/jcode-tui/src/tui/app/tui_lifecycle.rs`, `crates/jcode-tui/src/tui/app/remote/server_events.rs` and `crates/jcode-tui/src/tui/app/tests/auto_poke_cycles.rs`; add `crates/jcode-tui/src/tui/app/tests/continuation_evidence.rs` only if it keeps the existing module pattern clearer. Reuse `with_temp_jcode_home` and `create_test_app`.

- [ ] 4.1 Enumerate the actual reads and outcomes of `schedule_auto_poke_followup_if_needed` and add failing `continuation_tui_capture_parity`, `continuation_tui_pending_is_noop`, `continuation_tui_effects_are_not_acknowledged_by_capture` and `continuation_tui_runtime_identity_unknown` tests. Run the TUI `continuation_` recipe and record failures before adding hooks.
- [ ] 4.2 Implement read-only `TuiReplayStateV1`/context projection and metadata capture at the existing branch boundary. Obtain controller process identity from existing `jcode_build_meta` helpers; keep daemon/checkout identity separate and unknown when unavailable. Verify capture on/off schedules exactly the same messages/kinds/effects in the same order and never resets/re-arms the controller or applies an effect itself.
- [ ] 4.3 Add `continuation_tui_interleaved_refs` and `continuation_tui_tool_success_is_not_acceptance` tests using synthetic native events. Preserve same-session/call correlation, unknown legacy IDs and typed failures; assert `PASS` output, claims and tool success cannot create an acceptance verdict or leak payload strings. Implement only the necessary bounded event-reference adapter and rerun the TUI recipe.
- [ ] 4.4 Implement the test-only TUI replay adapter using the real scheduler, fixed clock/eligibility and isolated fixture state, not copied rules. Exercise queued/blocked/cancelled, missing context, final-response and no-follow-up cases with `continuation_tui_replay_fixture`; run both the TUI `continuation_` and existing `auto_poke` recipes. Do not make the pure crate depend on TUI or add a live state importer.

## 5. Balanced baseline and Rust SDK boundary

Files: keep sanitized fixtures with the policy/TUI tests above; modify `crates/jcode-sdk/src/lib.rs` and `crates/jcode-sdk/src/sdk_tests/parity.rs` for the export test. No raw session fixture, harness endpoint or TypeScript wire change is included.

- [ ] 5.1 Add and run `continuation_baseline_repeated_and_required`, `continuation_baseline_wait_and_stop`, `continuation_baseline_stale_and_error` and `continuation_baseline_incomplete_and_version` tests. Include unchanged/metadata-only repetition alongside a genuine newly required check, changed evidence, queued/blocked work, stop precedence, stale identity, interleaved tool errors, malformed/version-mismatched/incomplete inputs and budget boundaries. All fixtures must be reviewed synthetic data, not extracted private transcripts.
- [ ] 5.2 Record each engine's actual baseline case outcomes and limitations in fixture/module documentation, explicitly distinguishing derived fixtures from observed production receipts. Verify no case treats fewer turns alone as improvement, no raw metadata receipt is promoted to complete replay, and engine differences remain visible. Rerun the policy/TUI `continuation_` recipes with all expected case names present.
- [ ] 5.3 Export the pure receipt/fixture/replay contract through the existing Rust SDK and add `continuation_sdk_exports`. Run its SDK recipe plus the TUI/SDK compile check. Verify legacy snapshot and harness wire shapes remain unchanged and there is no new provider access or public TUI replay implementation in the pure SDK.

## 6. Integration evidence and native closeout inputs

- [ ] 6.1 Run the full policy suite, TUI continuation/auto-poke regression filters, base storage/config filters, SDK export check, TUI/SDK compile check and formatter recipe under coordinated test execution. Confirm filtered tests actually exist, and record exact commands, source revision, package/test counts, failures and offline-cache/platform limitations. Do not report an unrelated baseline failure as a pass or silently repair it here.
- [ ] 6.2 Perform the isolated end-to-end diagnostic check: enable capture only in a temporary test home, feed the native controller fixtures, read the resulting metadata JSON and verify bounds/privacy/provenance/coverage and unchanged native decisions/effect counts. Disable capture and verify no new trace writes. Observe results through the actual storage and TUI entry points, not a separate mock serializer; no shared daemon or provider is needed.
- [ ] 6.3 Review the final diff independently under the native execution workflow, repair actionable findings and rerun affected gates/review. Verify only this feature's code/tests/docs are included, no raw fixture data or external side effects occurred, and the metadata-versus-complete-fixture boundary is intact. Record review and acceptance evidence in this change before final task completion.

After implementation tasks pass, native `apply` owns verified scoped commits, archival and required persistence. Archive only after the final gates/review/commits pass; then update roadmap links to actual archived artifacts while preserving this ID. Do not add a circular "archive this change" implementation checkbox. Do not mark all of P01 Complete merely because this evidence slice closes. Publication and consequential external actions retain their normal authorization boundaries.
