# Tasks: reuse continuation logs and establish an offline baseline

This is the sole executable checklist for `record-continuation-evidence`. Read the [proposal](proposal.md), [design](design.md) and [delta spec](specs/continuation-evidence/spec.md) first. Proposal/schema validity alone does not authorize implementation, private-log ingestion, inference, gateway changes, publication or a shared-daemon restart. Execution follows the user's explicit Apply request; the change ID stays the same.

## Execution prerequisites and recipes

| Prerequisite | Bounded target | Authoring evidence / limitation |
| --- | --- | --- |
| Native authoring | OpenSpec 1.13.2, current checkout, `spec-driven` | Root/schema observed; validity is not implementation approval |
| Existing evidence | OmniRoute log capability and current Jcode logging sources | Gateway deployment/version, enabled capture, retention, route coverage and reliable cross-log correlation are not verified here. Unavailable access stays unknown and does not block synthetic tests |
| Rust checks | Workspace lockfile and already-cached dependencies; native `selfdev` action `test` compile lock | Toolchain/cache and implementation checks have not been exercised by authoring. Do not download dependencies or bypass a busy shared build |
| Test isolation | Existing `todo`/`done`/`snapshot` helpers and TUI `with_temp_jcode_home`/`create_test_app` | Reviewed synthetic state only; no real user profile, private log body, credentials, provider or gateway availability required |

Use these commands as the `command` argument to `selfdev` action `test` during separately authorized execution. Confirm named tests exist with `-- --list` before counting a filtered pass. `--offline --locked` forbids dependency downloads; a missing cache is a blocker for that check, not a pass.

```bash
cargo test --offline --locked -p jcode-todo-policy followup::tests -- --list
cargo test --offline --locked -p jcode-tui auto_poke -- --list
cargo test --offline --locked -p jcode-todo-policy continuation_baseline_ -- --list
cargo test --offline --locked -p jcode-tui continuation_baseline_ -- --list
cargo test --offline --locked -p jcode-todo-policy followup::tests -- --nocapture
cargo test --offline --locked -p jcode-tui auto_poke -- --nocapture
cargo test --offline --locked -p jcode-todo-policy continuation_baseline_ -- --nocapture
cargo test --offline --locked -p jcode-tui continuation_baseline_ -- --nocapture
cargo fmt --all -- --check
```

For an optional logging edit, also list and run its focused tests in its actual owner package. Use `cargo check --offline --locked -p jcode-tui` if the native call site changes, and the corresponding existing logger-package tests if the shared logger changes. Do not add SDK/public replay interfaces, capture configuration or a new fixture framework to satisfy these recipes. Add regression assertions against observed current decisions and effects, including undesirable outcomes; do not silently fix the controller to make an aspirational baseline pass. Future behavior fixes require their own authorized scope and failing regression test.

## 1. Establish existing coverage before adding anything

Files: update the evidence/limitations sections of `design.md` after inspecting `crates/jcode-tui/src/tui/app/input.rs`, `crates/jcode-todo-policy/src/followup.rs`, `crates/jcode-base/src/logging.rs` and `crates/jcode-logging/src/lib.rs`. Do not create another checklist or log store.

- [x] 1.1 Confirm the actual target revision and source log boundaries. Document existing `AUTO_POKE_DECISION` events and logger context, which native branches can return without an inference request, and the two distinct controller entry points. Verify that source comments suggesting parity do not substitute for observed engine-specific outcomes.
- [x] 1.2 Assess the available OmniRoute/Jcode evidence for one bounded repeated-validation diagnosis. Separate source-verified capability from observed deployed metadata, unknown/unavailable access and demonstrated missing evidence. Check actual route coverage, capture/retention and safe correlation/provenance only where authorized, without reading or copying private request bodies, enabling logs, changing routes or credentials. Do not attribute a request by timestamp alone, infer a stop from no request, or call source-pinned upstream behavior deployed proof. If metadata/access is unavailable, record the limitation and continue offline work.
- [x] 1.3 Classify the result as sufficient existing evidence, one demonstrated gap, or inconclusive. A gap must identify the concrete decision/observation, missing field or event and why existing logs/context cannot answer that bounded question. Verify that model confidence, a successful `ToolDone` and `PASS` text are not acceptance proof. Record no logging addition for sufficient or inconclusive coverage; no new journal is a fallback.

## 2. Build the engine-specific synthetic baseline

Files: extend `crates/jcode-todo-policy/src/followup_tests.rs` and `crates/jcode-tui/src/tui/app/tests/auto_poke_cycles.rs` using their existing helpers and direct production entry points. Use adjacent existing TUI event tests only if a case genuinely needs native event delivery; no copied decision function, new replay API or SDK export.

- [x] 2.1 Add and run `continuation_baseline_repeated_and_changed_work` and `continuation_baseline_required_checks_and_limits` in the pure-policy tests. Use complete synthetic `TodoSnapshot` and controller state to cover unchanged/metadata-only repetition, genuinely changed work/evidence, required completion gates, stop/disable behavior and gate-budget boundaries. Assert actual follow-up kind, effects and next state through `FollowUpPolicy::decide`; confirm the existing policy suite still passes.
- [x] 2.2 Add and run `continuation_baseline_auto_poke_repeated_and_required` and `continuation_baseline_auto_poke_wait_stop_and_errors` in the TUI tests. Invoke `schedule_auto_poke_followup_if_needed` through isolated apps to cover repetition versus newly required work, queued/busy waits, disabled/stop paths and relevant error/stale-state paths supported by the real boundary. Check generated/queued messages, flags/counters, ordering and read-versus-ack effects. Do not fabricate tool-error inputs for the pure policy if it does not consume them, require engine parity or translate an unattributed log into complete engine state.
- [x] 2.3 Document case inputs, engine identity, actual expected outcomes and evidence limits beside the tests. Where logs have missing/stale/conflicting identity, state that attribution is unknown rather than inventing fixtures from them. Include a negative example showing why absent traffic, claimed confidence or tool success alone cannot establish accepted completion. Verify all fixture data is reviewed synthetic, contains no private log payload, and runs without live gateway/provider access. A known undesirable baseline is not an improvement claim.

## 3. Close only a demonstrated logging gap, if any

Files: no production edit by default. A confirmed gap may justify a bounded metadata event/field at the existing call site in `crates/jcode-tui/src/tui/app/input.rs` or existing `crates/jcode-logging/src/lib.rs` context machinery, with tests in that owner package. A different boundary requires a documented exact owner and a scope check first.

- [x] 3.1 Apply the result of 1.3: for sufficient or inconclusive coverage, record that no logging change is required/permitted by this slice and continue to final verification. Only for a demonstrated gap, define the smallest bounded metadata field/event and a focused named test showing the missing observation. Verify existing session/task context cannot already supply it. Do not prescribe fields before this check or expand into provider transport, deployment, capture or retention changes.
- [x] 3.2 If justified, implement only that existing-log addition and run its focused tests, first demonstrating the missing observation and then the emitted metadata. Verify no private free text/body/arguments/output, credentials or transcript enter the added fields and no existing data permissions, capture/retention settings or services change. If safe correlation cannot be established, leave it unknown rather than adding raw content or a new store. If 3.1 skips the addition, record this task as not applicable with the evidence, not as an executed code test.
- [x] 3.3 For an addition, rerun the existing and new policy/TUI recipes and focused logger tests. Assert unchanged controller decisions, messages, counters, ordering and effects, including existing read/ack separation and an error/disabled-log case. Logging cannot become a decision input or cause clearing, retrying or sending work. Without an addition, verify the scoped diff contains no production change and run the baseline suites only. Do not claim unexecuted branch checks passed.

## 4. Verify and close this evidence slice truthfully

- [x] 4.1 Review the coverage map below, source references, fixture privacy and scoped diff. Confirm there is no new receipt schema/store, capture flag/path, raw log importer/exporter, public replay API, SDK change, gateway dependency or duplicate executable ledger. Record source-only versus runtime-verified findings and any unavailable evidence explicitly; no exact live-session replay or general privacy guarantee for existing logs is claimed.
- [ ] 4.2 Run the applicable offline recipes, native strict OpenSpec validation and link checks. Report exact tests/cases, results and blockers, including zero-match filters, missing cache and untested platforms. Do not equate artifact validity, tool success or fewer turns with acceptance or phase completion. Commit only verified owned changes with configured identity and preserve unrelated work; retain native review/archive gates for later execution closeout.

## Execution evidence and remaining gate

Execution baseline: `a9f44ba3e`, 2026-10-26, Rust/Cargo 1.98.1. Commands below ran through native `selfdev` test coordination on the owned test changes with the separately fingerprinted, pre-existing MCP edits preserved. The source/deployment assessment is in [design.md](design.md#execution-evidence-2026-10-26).

| Task / gate | Observed result |
| --- | --- |
| 1.1-1.3 | Existing TUI events/context verified from source. No discovered OmniRoute CLI/configured root; deployed properties remain unknown. Live attribution is inconclusive, not a demonstrated missing native observation |
| 3.1-3.2 | No logging addition. **3.2 is not applicable** by the documented branch decision; no logging code or logging-branch test was executed |
| Test discovery | `cargo test --offline --locked -p jcode-todo-policy followup::tests -- --list`: 13 named tests, including both policy baseline cases. `cargo test --offline --locked -p jcode-tui auto_poke -- --list`: 31 named tests, including both TUI baseline cases |
| Formatting | `cargo fmt --all -- --check`: passed on final implementation bytes |
| Policy baseline / regression | `cargo test --offline --locked -p jcode-todo-policy`: **70 passed**, 0 failed; this includes the two new direct-policy cases |
| TUI baseline / regression | `cargo test --offline --locked -p jcode-tui auto_poke -- --nocapture`: **31 passed**, 0 failed; 2,836 other TUI unit tests were not selected. Both new isolated-app cases ran |
| Policy strict lint | `cargo clippy --offline --locked -p jcode-todo-policy --all-targets --all-features -- -D warnings`: passed |
| Initial combined strict lint | `cargo clippy --offline --locked -p jcode-todo-policy -p jcode-tui --all-targets --all-features -- -D warnings`: **failed initially**, 14 diagnostics in six files unchanged from `a9f44ba3e`; none in the new baseline test files. Resolved by the bounded test-only repairs below |
| Scope / 3.3 | Only the existing test files and execution documentation changed. No production controller/logger/config/API, capture/retention, gateway or daemon change. Unrelated MCP files and `swarm-prompt.md` match preflight byte fingerprints |

Fresh final-byte results above are native tasks `74143924al` (format/tests/policy lint, exit 0) and `749673zsde` (combined lint, exit 101). Earlier fixture assumptions about notice ownership, armed versus enabled state, and queued-work precedence were corrected against native source without changing production rules. One introduced TUI lint was fixed before these reruns. Baseline fixtures and engine-specific unsupported dimensions are documented beside the tests, not inferred from production logs.

| Existing strict-lint blocker | Diagnostics |
| --- | --- |
| `crates/jcode-tui/src/tui/cloud_git.rs` | 3 octal-looking escape diagnostics |
| `crates/jcode-tui/src/tui/cloud_status.rs` | 4 octal-looking escape diagnostics |
| `crates/jcode-tui/src/tui/app/auth_tests.rs` | 2 collapsible-if diagnostics |
| `crates/jcode-tui/src/tui/app/commands_cloud.rs` | 1 type-complexity diagnostic |
| `crates/jcode-tui/src/tui/app/tests/remote_startup_input_03/part_01.rs` | 1 unnecessary-mutable-reference diagnostic |
| `crates/jcode-tui/src/tui/ui_tests/basic/reload_render.rs` | Unused import, cloned-reference slice, and unnecessary vector: 3 diagnostics |

Independent review of `a9f44ba3e..9f6710cc2` completed on 2026-10-26. A separate non-implementing worker inspected all six requirements/sixteen scenarios, the actual five-file commit, native callers/helpers, both final gate outputs/status records and full baseline/final/working bytes of the six lint-affected files. It found **no additional blocking or actionable defect in the scoped implementation**, confirmed the fourteen unchanged-source diagnostics, and explicitly withheld overall Apply approval. The four named baseline tests ran; unsupported native-tool/provenance dimensions remain documented rather than fabricated. No extra Cargo calls, repository edits or gate waivers were part of that review.

Task **4.2 remains open** pending independent re-review and lifecycle/persistence closeout, not a failed lint gate. The initial 14-diagnostic result above is historical. Following the continuation instruction, minimal, behavior-preserving repairs were confined to the six existing test sections: explicit NUL bytes, two collapsed test conditions, a private test-case alias, an immutable test borrow, and three render-test lint repairs. No lint was suppressed; production prefixes of the shared files and all unrelated byte fingerprints remain unchanged.

Native final-byte gate `970431uhib` passed (exit 0):

| Command / discovered test filter | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Passed |
| `cargo test --offline --locked -p jcode-todo-policy` | 70 passed |
| `cargo test --offline --locked -p jcode-tui --lib -- --list` | All selected filters discovered before execution |
| `cargo test --offline --locked -p jcode-tui --lib auto_poke -- --nocapture` | 31 passed, including both native baseline cases |
| Same TUI command with each filter: `snapshot_handles_rename_newline_and_independent_counts`, `complete_sample_and_individually_invalid_fields`, `partial_invalid_and_cpu_deltas`, `acceptance_refuses_unsent_state_without_consuming_it`, `safe_dispatch_rechecks_new_unsent_work_without_spawning`, `muse_subscription_canonical_success_publishes_untagged_event_and_updates_local_state`, `muse_subscription_old_success_arriving_after_new_attempt_leaves_new_pending_intact`, `test_paste_again_expands_matching_placeholder_not_newer_same_sized_paste` | 1 passed per filter, 8 additional tests |
| Same TUI command with `reload_overflow_preflight` | 2 passed |
| `cargo clippy --offline --locked -p jcode-todo-policy -p jcode-tui --all-targets --all-features -- -D warnings` | Passed; all 14 diagnostics resolved without production changes or suppressions |

These are **70 policy + 41 selected TUI tests**, not a claim that the complete TUI suite ran. Repaired-range independent review must still pass before completing 4.2 and archiving. Main tracks `origin/main`, with eight pre-existing local commits ahead of that ref at execution start; no publication or explicit local-only scope has been inferred from the continuation reminder.

## Requirement and scenario coverage

This table maps every requirement and scenario to execution tasks. Names match the delta exactly; the table is not a second checklist.

| Requirement | Scenarios | Execution tasks |
| --- | --- | --- |
| Existing-log reuse and coverage assessment | Existing sources assessed first; Local stop is not inferred from gateway absence; Source coverage is bounded | 1.1-1.3, 4.1 |
| Evidence limits and provenance | Deployment facts lack evidence; Correlation cannot be verified; Success signals are not proof | 1.2-1.3, 2.3, 4.1 |
| Engine-specific offline regression baseline | Repeated verification and required checks; Changed evidence and continuation states; Missing metadata, errors, and limits | 2.1-2.3, 4.2 |
| Behavioral noninterference | Pure policy remains unchanged; TUI scheduling remains unchanged; Offline test boundary | 2.1-2.2, 3.3, 4.1-4.2 |
| Demonstrated-gap logging only | Reproducible evidence gap; No demonstrated gap | 1.3, 3.1-3.3, 4.1 |
| Privacy without a new trace store | No private transcript material; Existing gateway privacy is unverified | 1.2, 2.3, 3.2, 4.1 |
