# Design and execution evidence

## Scope and compatibility

`CommandCandidate` carries `kind`, `name`, and an owned `description`. The two
kinds are `NativeCommand` and `Skill`. Native subcommand builders keep their
existing static tuples and convert at the shared ranking boundary. Rendering,
completion selection, caches, and `TuiState` consume the typed entries.

Native commands enter the deduplication set first. Remote skill names enter
before local skills when a remote catalog exists, so a same-named local skill
cannot invent a description for a legacy remote skill. Local discovery and
project-overlay behavior remain unchanged.

The existing remote payload is `History.skills: Vec<String>`, not a separate
skill-list event. Keep that field unchanged for old clients. Add optional
`History.skill_descriptions: HashMap<String, String>`, defaulting to an empty
map and omitted when empty. A missing description renders as `unknown`.
The live history sender obtains names and descriptions from the same effective
skill-registry snapshot. Model-catalog-only and persisted fallback responses
still omit skills and descriptions together.

Candidate admission rejects local and remote skill names containing Unicode
control characters before deduplication. Invocation names are never rewritten.
Skill candidate construction removes Unicode control characters from descriptions.
The renderer adds `cmd` or `skill` with separate colors, then uses the existing
display-cell-aware ellipsis helpers, reserving suffix space for `↑` and `+more`
navigation indicators before truncating metadata.
Widths zero and one produce an empty row and an ellipsis respectively.

## Execution binding and boundaries

- Checkout: authorized shared worktree for `github.com/leonardoacosta/jcode`,
  starting at `e5b0c7426`, branch `feat/roadmap-first-wave`.
- Source graph absent. Source searches identified helper, renderer, `TuiState`,
  `History` sender/receiver, cache, and fixture integration boundaries.
- Scope authority: selector recovery assignment, 2026-11-03. Runtime-owned
  `events.rs`, `app.rs`, `tui_lifecycle.rs`, `remote/server_events.rs`, and
  `client_state.rs` were serialized with the coordinator, then released.
- No provider changes, Git mutations, archive, worktree changes, shared-daemon
  changes, or shared debug sessions performed.
- Coordinator owns isolated rebuilt-binary local/remote visual frames and
  final review, commit, archive, and delivery gates. Those are not established
  by unit tests and remain open here.

## Readiness assessment

Initial-dispatch assessment under `preflight-check-v2-shadow`:
G1 2 (explicit scope and authority), G2 1 (local tests available, coordinator
runtime frames pending), G3 1 (local implementation authorized, terminal delivery
reserved), G4 2 (shared daemon excluded, isolated runtime coordinator-owned),
G5 2 (exact overlapping paths serialized), G6 2 (requirements mapped below).
Advisory score: 10/12, 83.33%. Local implementation eligible. Runtime acceptance
and terminal closeout remain coordinator-owned, not marked complete.

## Verification record

All Cargo commands run from this worktree through `scripts/dev_cargo.sh` with
`CARGO_TARGET_DIR=/home/nyaptor/.jcode/target`.

- RED: `test -p jcode-protocol skill_selector_metadata --lib` failed with E0026
  for the missing `History.skill_descriptions` field before its implementation.
- RED: `test -p jcode-tui skill_selector_metadata --lib` failed with E0609/E0433
  for missing candidate fields/kind, and E0061 for the missing width argument.
  Integration constructor failures were also present and repaired.
- GREEN: `test -p jcode-tui skill_selector_metadata --lib`: 4 passed. Covers local
  descriptions, native collision precedence, legacy remote `unknown`, control
  sanitation, History description refresh, metadata-only catalog preservation,
  kind styles, and Unicode cell widths 0/1/20/40/80.
- `openspec validate add-typed-skill-selector-metadata --strict`: passed.

The invalid two-filter recipe is replaced with separate `skill_startup` and
`ssh_remote` commands. Final regression command results are recorded in
`tasks.md` after they finish. Existing profile-package warnings and an unrelated
`format_tool_summary` dead-code warning occur during these tests.
