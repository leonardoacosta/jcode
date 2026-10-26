# Tasks: reuse continuation logs and establish an offline baseline

This is the sole executable checklist for `record-continuation-evidence`. All tasks remain unimplemented. Read the [proposal](proposal.md), [design](design.md) and [delta spec](specs/continuation-evidence/spec.md) first. This revision authorizes planning only, not implementation, private-log ingestion, inference, gateway changes, publication or a shared-daemon restart. The change ID stays the same.

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

- [ ] 1.1 Confirm the actual target revision and source log boundaries. Document existing `AUTO_POKE_DECISION` events and logger context, which native branches can return without an inference request, and the two distinct controller entry points. Verify that source comments suggesting parity do not substitute for observed engine-specific outcomes.
- [ ] 1.2 Assess the available OmniRoute/Jcode evidence for one bounded repeated-validation diagnosis. Separate source-verified capability from observed deployed metadata, unknown/unavailable access and demonstrated missing evidence. Check actual route coverage, capture/retention and safe correlation/provenance only where authorized, without reading or copying private request bodies, enabling logs, changing routes or credentials. Do not attribute a request by timestamp alone, infer a stop from no request, or call source-pinned upstream behavior deployed proof. If metadata/access is unavailable, record the limitation and continue offline work.
- [ ] 1.3 Classify the result as sufficient existing evidence, one demonstrated gap, or inconclusive. A gap must identify the concrete decision/observation, missing field or event and why existing logs/context cannot answer that bounded question. Verify that model confidence, a successful `ToolDone` and `PASS` text are not acceptance proof. Record no logging addition for sufficient or inconclusive coverage; no new journal is a fallback.

## 2. Build the engine-specific synthetic baseline

Files: extend `crates/jcode-todo-policy/src/followup_tests.rs` and `crates/jcode-tui/src/tui/app/tests/auto_poke_cycles.rs` using their existing helpers and direct production entry points. Use adjacent existing TUI event tests only if a case genuinely needs native event delivery; no copied decision function, new replay API or SDK export.

- [ ] 2.1 Add and run `continuation_baseline_repeated_and_changed_work` and `continuation_baseline_required_checks_and_limits` in the pure-policy tests. Use complete synthetic `TodoSnapshot` and controller state to cover unchanged/metadata-only repetition, genuinely changed work/evidence, required completion gates, stop/disable behavior and gate-budget boundaries. Assert actual follow-up kind, effects and next state through `FollowUpPolicy::decide`; confirm the existing policy suite still passes.
- [ ] 2.2 Add and run `continuation_baseline_auto_poke_repeated_and_required` and `continuation_baseline_auto_poke_wait_stop_and_errors` in the TUI tests. Invoke `schedule_auto_poke_followup_if_needed` through isolated apps to cover repetition versus newly required work, queued/busy waits, disabled/stop paths and relevant error/stale-state paths supported by the real boundary. Check generated/queued messages, flags/counters, ordering and read-versus-ack effects. Do not fabricate tool-error inputs for the pure policy if it does not consume them, require engine parity or translate an unattributed log into complete engine state.
- [ ] 2.3 Document case inputs, engine identity, actual expected outcomes and evidence limits beside the tests. Where logs have missing/stale/conflicting identity, state that attribution is unknown rather than inventing fixtures from them. Include a negative example showing why absent traffic, claimed confidence or tool success alone cannot establish accepted completion. Verify all fixture data is reviewed synthetic, contains no private log payload, and runs without live gateway/provider access. A known undesirable baseline is not an improvement claim.

## 3. Close only a demonstrated logging gap, if any

Files: no production edit by default. A confirmed gap may justify a bounded metadata event/field at the existing call site in `crates/jcode-tui/src/tui/app/input.rs` or existing `crates/jcode-logging/src/lib.rs` context machinery, with tests in that owner package. A different boundary requires a documented exact owner and a scope check first.

- [ ] 3.1 Apply the result of 1.3: for sufficient or inconclusive coverage, record that no logging change is required/permitted by this slice and continue to final verification. Only for a demonstrated gap, define the smallest bounded metadata field/event and a focused named test showing the missing observation. Verify existing session/task context cannot already supply it. Do not prescribe fields before this check or expand into provider transport, deployment, capture or retention changes.
- [ ] 3.2 If justified, implement only that existing-log addition and run its focused tests, first demonstrating the missing observation and then the emitted metadata. Verify no private free text/body/arguments/output, credentials or transcript enter the added fields and no existing data permissions, capture/retention settings or services change. If safe correlation cannot be established, leave it unknown rather than adding raw content or a new store. If 3.1 skips the addition, record this task as not applicable with the evidence, not as an executed code test.
- [ ] 3.3 For an addition, rerun the existing and new policy/TUI recipes and focused logger tests. Assert unchanged controller decisions, messages, counters, ordering and effects, including existing read/ack separation and an error/disabled-log case. Logging cannot become a decision input or cause clearing, retrying or sending work. Without an addition, verify the scoped diff contains no production change and run the baseline suites only. Do not claim unexecuted branch checks passed.

## 4. Verify and close this evidence slice truthfully

- [ ] 4.1 Review the coverage map below, source references, fixture privacy and scoped diff. Confirm there is no new receipt schema/store, capture flag/path, raw log importer/exporter, public replay API, SDK change, gateway dependency or duplicate executable ledger. Record source-only versus runtime-verified findings and any unavailable evidence explicitly; no exact live-session replay or general privacy guarantee for existing logs is claimed.
- [ ] 4.2 Run the applicable offline recipes, native strict OpenSpec validation and link checks. Report exact tests/cases, results and blockers, including zero-match filters, missing cache and untested platforms. Do not equate artifact validity, tool success or fewer turns with acceptance or phase completion. Commit only verified owned changes with configured identity and preserve unrelated work; retain native review/archive gates for later execution closeout.

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
