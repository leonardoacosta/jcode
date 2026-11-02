# Tasks

- [ ] Define `ContinuationEvidence` with size bound and redaction in `jcode-todo-policy`.
- [ ] Build the record at each `AUTO_POKE_DECISION` site in `crates/jcode-tui/src/tui/app/input.rs` and log it as one JSON line.
- [ ] Tests: one record per decision; failed check recorded; oversized/sensitive input truncated; decisions identical with and without recording.
- [ ] Run `cargo test -p jcode-todo-policy` and `cargo test -p jcode-tui continuation`.
- [ ] Run `openspec validate add-structured-continuation-evidence --strict`.
