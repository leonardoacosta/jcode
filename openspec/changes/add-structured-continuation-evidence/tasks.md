# Tasks

- [x] Define `ContinuationEvidence` with size bound and redaction in `jcode-todo-policy`.
- [x] Build the record at each `AUTO_POKE_DECISION` site in `crates/jcode-tui/src/tui/app/input.rs` and log it as one JSON line.
- [x] Tests: one record per decision; failed check recorded; oversized/sensitive input truncated; decisions identical with and without recording.
- [x] Run `cargo test -p jcode-todo-policy` and `cargo test -p jcode-tui continuation`.
- [x] Run `openspec validate add-structured-continuation-evidence --strict`.

## Acceptance detail

- [x] Add fixed-enum reasons, todo counts, optional fingerprint-change observation, wait/dependency flags, and explicit scheduler predicate results without raw text.
- [x] Verify bounded/redacted model, unknown external-check availability, and pure-policy state/decision parity.
- [x] Verify TUI one-per-decision, wait, failed scheduler-predicate, and queue/cap/fingerprint parity tests.
- [x] Integrate typed required-check identity/outcomes and verify a failed required command remains `fail` without reading command output or model claims.
- [ ] Verify actual session log emission against a rebuilt isolated runtime.
- [ ] Coordinator: final fresh gates, independent review, scoped commit, archive, and required persistence.

## Observed evidence

2026-11-03 UTC, base `e5b0c74262039cdd48a45b155b58d483aa5967d4`.
No Git, archive, provider, shared daemon, or worktree mutation by this lane.

- RED: `JCODE_REMOTE_CARGO=0 scripts/dev_cargo.sh test -p jcode-todo-policy continuation_evidence --lib` failed with missing evidence types before implementation.
- GREEN: the same focused model command passed 3 tests. After adding worst-case fixed-schema bounds, `JCODE_REMOTE_CARGO=0 scripts/dev_cargo.sh test -p jcode-todo-policy` passed all 76 tests.
- RED: `JCODE_REMOTE_CARGO=0 scripts/dev_cargo.sh test -p jcode-tui continuation_evidence --lib` failed with missing `schedule_auto_poke_followup_with_evidence` before its implementation.
- Initial TUI GREEN attempts were blocked by peer-owned `History.skill_descriptions` and `Pong.server_runtime` initializer integration. After the coordinator/selector owner fixed those sources, `JCODE_REMOTE_CARGO=0 scripts/dev_cargo.sh test -p jcode-tui continuation --lib` passed 18 tests, including all 4 new evidence tests. `JCODE_REMOTE_CARGO=0 scripts/dev_cargo.sh test -p jcode-tui auto_poke --lib` passed 32 tests. Actual `test result: ok` totals were inspected, not inferred from wrapper exit status.
- `openspec validate add-structured-continuation-evidence --strict` passed.
- Final owned `rustfmt --check --edition 2024 --config skip_children=true` and `git diff --check` passed after snapshot-binding and hermetic TUI fixture edits.

## Checkpoint acceptance gap (resolved for typed command evidence)

`required_checks_available=false` is truthful but does not satisfy the failed
required-command scenario. Scheduler predicates are recorded separately and are
not required-command outcomes. `TodoGoal.feedback_loop` has no typed check
identity binding. Foreground bash returns process exit codes in formatted output,
not metadata. `ToolDone.error` describes tool failure, not required-command status.
See `design.md` for the exact source boundary and minimal follow-through. Keep the
failed-required-check and overall acceptance tasks unchecked until that source is
integrated and tested. Do not weaken the proposal or delta spec to hide the gap.

The resumed implementation below closes this typed-command gap. Actual isolated
session-log emission remains open.

## Typed required-check follow-through

2026-11-03 UTC, resumed from `9df91d357` on the existing assigned worktree.
No Git, archive, provider, shared runtime, config, or worktree mutation by this lane.
Existing `tasks.md` remains the sole task ledger.

- Native bash accepts optional validated `required_check_id`; invalid bindings
  fail before execution. Existing authorization and destructive-command gates stay intact.
- Direct and reload-persistable foreground process completions populate existing
  `ToolOutput.metadata.process` from actual child exit status.
- Additive optional typed `ToolDone.required_check` preserves outcomes for normal
  streamed and manual server tool execution. Local and remote TUI feed a shared
  max-eight per-turn accumulator. Failures are sticky per explicit ID.
- Diagnostics pseudonymize names as `check_1` etc., with no stdout/prose parsing.
  Background/promoted/incomplete/null/error results never establish a pass.
- Reset at actual local/remote turn starts, external-turn adoption, and session switch.
  `FollowUpPolicy`, scheduler control flow, queue/cap/fingerprint behavior unchanged.
- RED observed missing `RequiredCheckOutcome`, missing bounded accumulator, missing
  typed wire field, and missing bash extraction before implementation. Logs are
  `required-check-{red,policy-red,wire-red,bash-red}.log` under the agent scratch directory.
- GREEN: task-types focused contract 1; all todo-policy 78; all protocol 103;
  native bash required-check cases 3 (real failed process in both foreground
  implementations, success/signal, invalid-ID nonexecution, explicit/promoted background).
  All native bash tests 44 and replay tests 17 passed. Fresh final results follow.
- Real TUI registry bash execution (`printf 'Everything passed'; exit 7`) transferred
  through local observation and serialized typed ToolDone into remote handling;
  diagnostic result remained `fail`. Continuation suite passed 20 including real-process,
  external-turn reset, bounded/redacted recorder, and prior queue/cap/fingerprint parity.
- `python3 scripts/check_dependency_boundaries.py` and `git diff --check` passed.
- Concurrent dev-wrapper edits briefly blocked reruns with `remote: command not found`
  and unmatched-quote errors after Cargo-gate waits. Those attempts are not passes.
- Actual isolated runtime session-log emission, final independent review, Git/archive,
  and required persistence remain unchecked and coordinator-owned.

### Fresh final verification

All Cargo commands used `JCODE_REMOTE_CARGO=0 CARGO_TARGET_DIR=/home/nyaptor/.jcode/target scripts/dev_cargo.sh`:

- `test -p jcode-task-types --lib`: 8 passed.
- `test -p jcode-todo-policy --lib`: 78 passed, including max-eight plus worst-case
  scalar-count record under 2,048 bytes, pending/null/error nonpasses, sticky failure,
  ID deduplication, stable per-turn pseudonyms, and unchanged pure policy.
- `test -p jcode-app-core required_check --lib`: 4 passed, including actual startup
  failure, actual nonzero/signal/zero process, invalid-binding nonexecution, and background.
- `test -p jcode-tui continuation --lib -- --test-threads=1`: 21 passed. Includes real
  registry-executed failed bash through local and serialized remote paths, sibling
  completion and later success cannot erase failure, external-turn reset, and failed
  required-check parity for queues/caps/fingerprints across repeated decisions.
- `test -p jcode-tui auto_poke --lib -- --test-threads=1`: 32 passed.
- `test -p jcode-app-core replay::tests --lib`: 17 passed.
- A parallel `continuation` run had 20 pass and the neighboring existing
  `test_restore_session_with_selfdev_reload_tool_result_queues_continuation` fail.
  Inspection found it uses shared test-home/session state without the evidence-test
  environment lock. Serial whole-filter rerun passed all 21; no unrelated fixture
  source was changed. Treat the shared-environment race as a verification limitation.
- `openspec validate add-structured-continuation-evidence --strict`, dependency
  boundary checker, owned-source rustfmt check (`skip_children=true,reorder_modules=false`),
  and `git diff --check` passed.
- Final `test -p jcode-protocol --lib`: 103 passed.
- Final `test -p jcode-tui tool_done --lib -- --test-threads=1`: 2 passed.
- Final `test -p jcode-app-core tool::bash::tests --lib -- --test-threads=1`:
  45 passed, including existing destructive-command permissions tests.
  The preceding parallel whole-bash run had 43 pass/2 fail in existing
  `bash_refuses_to_delete_the_home_directory` and
  `indirect_dispatch_paths_cannot_bypass_the_gate`. Both independently mutate
  process-global `HOME` without locking (same file, lines 954 and 1060 before
  appended tests). Inspection established competing environment writes; serial
  whole-filter rerun passed 45. No gate logic or unrelated fixtures changed.
- Source freeze after these gates. Diagnostic identities are explicitly turn-local
  aliases, not hashes, raw names, or cross-turn identifiers. Actual isolated
  session-log runtime check and final independent boundary review remain required.

### Integrated caller and batch review remediation

- Integrated root build exposed exhaustive ToolDone patterns in
  `src/cli/{commands.rs,acp.rs}`. Both now explicitly ignore added diagnostic fields;
  ACP legacy fixture supplies absent observations. NDJSON/ACP no-secret regression
  tests passed 2; NDJSON filter passed 4; root `check -p jcode --all-targets` passed
  before the additive plural batch extension. A final rerun follows.
- Reviewer identified native batch dropping child process metadata. Added bounded
  `RequiredCheckBatch` and optional plural `ToolDone.required_checks`, preserving
  singular bash compatibility. Aggregate only actual native bash child results,
  explicit normalized IDs, max-eight sticky dedup and overflow. Unrelated child
  output/metadata cannot assert success. Re-bound at receiver as well.
- RED: `test -p jcode-app-core required_check_batch --lib` failed for absent extractor.
  GREEN: actual failed batch child command with PASS stdout remains `fail`, later
  same-ID zero cannot overwrite it; local plus serialized wire/remote evidence pass.
  Ten actual child completions cap at eight with truncation. Existing nested batch
  guard still rejects nested dispatch; no nesting support or permission change.
- Refreshed `test -p jcode-app-core required_check --lib`: 6 passed before final
  arbitrary-metadata negative test. Refreshed serial TUI `continuation`: 23 passed,
  including oversized remote aggregate rebound/redaction. Additional final gates
  recorded below when complete.
- Broad parallel ACP tests had 22 pass/1 existing skill-catalog fixture failure
  (`optimization` unexpectedly present beside `interrogate`). Serial rerun pending;
  no unrelated skill selector source modified.
- Final aggregate gates: task-types 9 passed; todo-policy 78 passed; protocol 104
  passed; batch tests 18 passed, including unrelated-tool forged process/aggregate
  metadata cannot establish success. Root `check -p jcode --all-targets` passed.
  Serial ACP remains 22 pass/1 existing skill-catalog fixture mismatch (`optimization`
  ambient entry beside `interrogate`), so that broader filter is not claimed passing.
- Frozen runtime route inspection: server `tool:bash` debug action directly returns
  registry ToolOutput metadata without ToolDone/Done publication. Input-shell (`!`)
  sends only command text to raw shell and produces InputShellResult, not a bound
  native bash result. Current TUI debug/replay surface has no server-event injection
  or scheduler trigger. A real bound-bash TUI runtime path therefore requires a
  model turn (isolated mock provider or authorized provider), not merely debug-tool
  execution. Unit real-process/local/wire tests do not substitute for session logs.
  Coordinator owns the actual runtime test and any fixture scope decision.
