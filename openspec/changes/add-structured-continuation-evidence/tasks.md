# Tasks

- [x] Define `ContinuationEvidence` with size bound and redaction in `jcode-todo-policy`.
- [x] Build the record at each `AUTO_POKE_DECISION` site in `crates/jcode-tui/src/tui/app/input.rs` and log it as one JSON line.
- [ ] Tests: one record per decision; failed check recorded; oversized/sensitive input truncated; decisions identical with and without recording.
- [x] Run `cargo test -p jcode-todo-policy` and `cargo test -p jcode-tui continuation`.
- [x] Run `openspec validate add-structured-continuation-evidence --strict`.

## Acceptance detail

- [x] Add fixed-enum reasons, todo counts, optional fingerprint-change observation, wait/dependency flags, and explicit scheduler predicate results without raw text.
- [x] Verify bounded/redacted model, unknown external-check availability, and pure-policy state/decision parity.
- [x] Verify TUI one-per-decision, wait, failed scheduler-predicate, and queue/cap/fingerprint parity tests.
- [ ] Integrate typed required-check identity/outcomes and verify a failed required command remains `fail` without reading command output or model claims.
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

## Acceptance gap

`required_checks_available=false` is truthful but does not satisfy the failed
required-command scenario. Scheduler predicates are recorded separately and are
not required-command outcomes. `TodoGoal.feedback_loop` has no typed check
identity binding. Foreground bash returns process exit codes in formatted output,
not metadata. `ToolDone.error` describes tool failure, not required-command status.
See `design.md` for the exact source boundary and minimal follow-through. Keep the
failed-required-check and overall acceptance tasks unchecked until that source is
integrated and tested. Do not weaken the proposal or delta spec to hide the gap.
