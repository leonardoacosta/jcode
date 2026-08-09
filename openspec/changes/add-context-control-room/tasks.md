# Tasks

## 1. Context snapshot and persistence

- [ ] 1.1 Add context snapshot types for semantic hierarchy and execution substrate.
  - touches: `crates/jcode-session-types/src/context.rs (new)`, `crates/jcode-session-types/src/lib.rs`, `crates/jcode-app-core/src/server/context.rs (new)`
  - depends on: none
  - Done when organization, project, workspace/worktree, initiative, task/run, Jcode session, and Herdr pane rows can represent value, stable ID, provenance, confidence, copyability, focusability, and unavailable reason.

- [ ] 1.2 Implement context snapshot assembly from persisted Jcode identity, current session/client state, path-derived fallback, and optional Herdr metadata.
  - touches: `crates/jcode-app-core/src/server/context.rs (new)`, `crates/jcode-app-core/src/server/client_session.rs`, `crates/jcode-app-core/src/server/state.rs`
  - depends on: 1.1
  - Done when unit tests prove persisted identity wins over cwd/Herdr, `$HOME` reconnect does not clobber project context, Herdr absence degrades cleanly, and unavailable fields are explicit.

- [ ] 1.3 Persist additive semantic context identity across reload and reconnect.
  - touches: `crates/jcode-app-core/src/server/context.rs (new)`, `crates/jcode-app-core/src/server/client_session.rs`, `crates/jcode-app-core/src/server/client_session_tests.rs`
  - depends on: 1.2
  - Done when reload/reconnect tests preserve semantic IDs independently from refreshed execution-substrate metadata.

## 2. TUI Control Room overlay

- [ ] 2.1 Add Control Room overlay state, renderer, and deterministic rendering fixtures.
  - touches: `crates/jcode-tui/src/tui/app.rs`, `crates/jcode-tui/src/tui/app/control_room.rs (new)`, `crates/jcode-tui/src/tui/control_room.rs (new)`, `crates/jcode-tui/src/tui/app/tests/control_room.rs (new)`
  - depends on: 1.1
  - Done when rendering tests show the approved sections, provenance labels, unavailable rows, footer hints, small-terminal scrolling, and no persistent rail/side-panel reuse.

- [ ] 2.2 Wire `Alt+O` toggle and overlay input ownership.
  - touches: `crates/jcode-tui/src/tui/app/input.rs`, `crates/jcode-tui/src/tui/app/local.rs`, `crates/jcode-tui/src/tui/app/remote.rs`, `crates/jcode-tui/src/tui/app/tests/control_room.rs (new)`
  - depends on: 2.1
  - Done when tests prove `Alt+O` opens/closes from normal input, `Esc` closes, draft input is preserved, keys do not leak into prompts while open, and higher-priority overlays retain ownership.

- [ ] 2.3 Implement overlay navigation, copy action, safe focus action, and feedback.
  - touches: `crates/jcode-tui/src/tui/app/control_room.rs (new)`, `crates/jcode-tui/src/tui/app/helpers/clipboard_helper.rs`, `crates/jcode-tui/src/tui/app/tests/control_room.rs (new)`
  - depends on: 2.2
  - Done when tests cover row/section navigation, copyable and non-copyable rows, focusable and non-focusable rows, and no implicit pane/session spawning.

## 3. Documentation and regression validation

- [ ] 3.1 Add context architecture documentation.
  - touches: `docs/CONTEXT_ARCHITECTURE.md (new)`
  - depends on: 1.3
  - Done when the doc explains hierarchy, Jcode-owned identity, Herdr execution substrate, persistence boundaries, provenance labels, and rejected UI alternatives.

- [ ] 3.2 Add integration and test evidence documentation.
  - touches: `docs/CONTEXT_CONTROL_ROOM_EVIDENCE.md (new)`
  - depends on: 2.3
  - Done when the doc records validation commands, expected results, degraded Herdr behavior, keybinding/overlay collision coverage, and known limitations.

- [ ] 3.3 Run focused app-core context/session validation.
  - touches: `crates/jcode-app-core/src/server/context.rs (new)`, `crates/jcode-app-core/src/server/client_session_tests.rs`, `crates/jcode-app-core/src/protocol_tests/context.rs (new)`
  - depends on: 1.3
  - Verification recipe: run `cargo test -p jcode-app-core context` and `cargo test -p jcode-app-core client_session`; expected result: both commands exit 0 with context snapshot, persistence, reload, reconnect, and Herdr-degraded cases passing.

- [ ] 3.4 Run focused TUI overlay/keybinding validation.
  - touches: `crates/jcode-tui/src/tui/app.rs`, `crates/jcode-tui/src/tui/app/input.rs`, `crates/jcode-tui/src/tui/app/tests/control_room.rs (new)`
  - depends on: 2.3
  - Verification recipe: run `cargo test -p jcode-tui control_room` and `cargo test -p jcode-tui session_picker usage_overlay keybinding`; expected result: both commands exit 0 with Control Room and existing overlay/keybinding behavior passing.

- [ ] 3.5 Run final repository feature validation and persistence.
  - touches: `openspec/changes/add-context-control-room/proposal.md`, `openspec/changes/add-context-control-room/design.md`, `openspec/changes/add-context-control-room/specs/context-control-room/spec.md`, `openspec/changes/add-context-control-room/tasks.md`
  - depends on: 3.1, 3.2, 3.3, 3.4
  - Verification recipe: run `cargo fmt --all -- --check`, `openspec validate add-context-control-room --strict --no-interactive`, and `git diff --check`; expected result: all commands exit 0 and only owned paths are staged/committed.
