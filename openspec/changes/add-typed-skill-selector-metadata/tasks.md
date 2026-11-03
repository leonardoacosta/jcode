## 1. Implementation
- [x] 1.1 Introduce a typed candidate struct (kind, name, description) in `state_ui_input_helpers.rs` and migrate `command_candidates`/`rank_suggestions`
- [x] 1.2 Use `Skill.description` for local skill rows; sanitize control characters
- [x] 1.3 Add optional descriptions to the remote skill payload; default to "unknown" for legacy peers
- [x] 1.4 Render kind label with distinct style and ellipsis truncation by display cells

## 2. Verification
- [x] 2.1 Unit tests: descriptions, legacy-peer unknown, collision precedence, control text, widths 0/1/20/40/80
- [x] 2.2 `CARGO_TARGET_DIR=/home/nyaptor/.jcode/target scripts/dev_cargo.sh test -p jcode-tui skill_startup --lib` and separate `CARGO_TARGET_DIR=/home/nyaptor/.jcode/target scripts/dev_cargo.sh test -p jcode-tui ssh_remote --lib`
- [ ] 2.3 Visual check with `debug_socket` tester frames, local and remote
- [x] 2.4 `openspec validate add-typed-skill-selector-metadata --strict`

## Evidence and handoff

See `design.md` for design, scope boundaries, readiness findings, and RED evidence.
Fresh checks on 2026-11-03, from the shared authorized worktree:

| Command after `CARGO_TARGET_DIR=/home/nyaptor/.jcode/target scripts/dev_cargo.sh` | Observed result |
| --- | --- |
| `test -p jcode-tui skill_selector_metadata --lib` | 4 passed |
| `test -p jcode-tui skill_startup --lib` | 3 passed |
| `test -p jcode-tui ssh_remote --lib` | 5 passed after adapting bootstrap test to coordinator's read-only Ping before Subscribe |
| `test -p jcode-tui suggestions --lib` | 39 passed |
| `test -p jcode-app-core client_state_tests --lib` | 20 passed, including real live History sender names/descriptions from project skills, empty persisted fallback |
| `test -p jcode-protocol --lib` | 102 passed, including optional description-map legacy decoding and roundtrip |

`git diff --check` passed. Strict OpenSpec validation passed after adding design.
No shared daemon/debug, provider, Git, worktree, or archive mutations performed.
Task 2.3 remains open for coordinator's isolated rebuilt-binary frames. Independent
review and workflow finalization are also coordinator-owned, not established by
these worker test results.

### Final review and full-suite regression repairs

The full TUI suite exposed two selector regressions. Long descriptions hid the
existing `↑`/`+more` indicators. The renderer now reserves their suffix width with
the existing suffix-preserving ellipsis helper. The overlay layout test expected
slash-first rows. Its row detector now recognizes the required `cmd`/`skill`
prefixes, retaining all row-position assertions.

Independent review found that stripping controls from invocation names could
create a second `/help` skill row. Candidate admission now rejects control-bearing
local/remote names before deduplication rather than rewriting invocation text.
Both new regressions failed before the repair: missing `+2 more` suffix and two
`/help` rows instead of one.

Final focused checks after these source repairs, 2026-11-03 03:48 UTC:
- `test -p jcode-tui skill_selector_metadata --lib`: 6 passed.
- `test -p jcode-tui command_palette_open_does_not_move_existing_rows --lib`: 1 passed.
- `test -p jcode-tui test_command_suggestion_render_window_scrolls_with_selection --lib`: 1 passed.
- `test -p jcode-tui suggestions --lib`: 40 passed.
- `git diff --check` and strict OpenSpec validation: passed.

These commands used the same `CARGO_TARGET_DIR` and `scripts/dev_cargo.sh` prefix
above. Visual acceptance remains open. Final independent re-review is coordinator-owned.
