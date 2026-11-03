## 1. Implementation
- [x] 1.1 Introduce a typed candidate struct (kind, name, description) in `state_ui_input_helpers.rs` and migrate `command_candidates`/`rank_suggestions`
- [x] 1.2 Use `Skill.description` for local skill rows; sanitize control characters
- [x] 1.3 Add optional descriptions to the remote skill payload; default to "unknown" for legacy peers
- [x] 1.4 Render kind label with distinct style and ellipsis truncation by display cells

## 2. Verification
- [x] 2.1 Unit tests: descriptions, legacy-peer unknown, collision precedence, control text, widths 0/1/20/40/80
- [x] 2.2 `CARGO_TARGET_DIR=/home/nyaptor/.jcode/target scripts/dev_cargo.sh test -p jcode-tui skill_startup --lib` and separate `CARGO_TARGET_DIR=/home/nyaptor/.jcode/target scripts/dev_cargo.sh test -p jcode-tui ssh_remote --lib`
- [x] 2.3 Visual check with isolated debug tester frames and real PTY output, local and remote
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
Task 2.3 was open at this checkpoint and is satisfied by the real isolated PTY
acceptance below. Independent review and workflow finalization remain
coordinator-owned, not established by these worker test results.

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
above. Visual acceptance was still open at this checkpoint and is recorded below.
Final independent re-review is coordinator-owned.

### Actual local and remote PTY visual acceptance

On 2026-11-03 at 04:05 UTC, scratch-only harness
`/home/nyaptor/.jcode/scratch/selector-visual-harness.py` launched a rebuilt binary
against its own daemon, socket, data/config/runtime directories, and two real
80×30 PTY debug testers. Local mode used ordinary same-host skill discovery.
Remote mode set `JCODE_SSH_REMOTE=selector-fixture` and an explicit isolated
`--remote-working-dir`, exercising server-authoritative skill descriptions.
This checks remote-mode protocol/rendering, not a real SSH network transport.

Binary: `/home/nyaptor/.jcode/target/selfdev/jcode`.
SHA-256: `48bc410e0d1dd87cd8a313ce99022980c4b02f5df584f5010d3ca45bfe364db4`.

Artifact root: `/home/nyaptor/.jcode/scratch/selector-visual-lgcsfkgl/`.
For each `local` and `remote` mode:
- `*-frame.json`: current `input_preview: "/audit"`, `input_len: 6`,
  `has_suggestions: true`, terminal size `[80,30]`.
- `*-pty.bin`: actual tester terminal stdout captured before process stop.
- `*-screen.txt`: cursor-position-aware reconstruction of that terminal output.
- `*-state.json`: `messages: 0`, `processing: false`, `input: "/audit"`.
- `acceptance.json`: both reconstructed screens contain exactly one row 20:
  `skill /audit-fixture  Audit fixture checks selectors`.

The harness used `client:frame` through the isolated daemon debug socket to force
a current draw after setting input. The tester-file `frame` route alone only
reads a prior capture and does not force an idle redraw. Local frame ID 0 is valid
because its input witness is `/audit`, not the prior empty input. Remote frame ID
is 1. No prompt was submitted and no provider inference was requested. Testers were
stopped (`testers.json` is `[]`) and only the owned daemon process group was stopped.

Reconstruction command:
`python3 /home/nyaptor/.jcode/scratch/selector-visual-reconstruct.py`.
Fresh final-binary replay remains part of coordinator integration if the binary
changes after this evidence. Source files were unchanged for this acceptance.
