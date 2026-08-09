# Tasks

## 1. Configuration schema

- [ ] 1.1 Add `display.footer` config section with style, per-segment visibility, icon mode, path display, and context thresholds.
  - touches: `crates/jcode-config-types/src/display.rs`
  - depends on: none
  - Done when the new section deserializes with documented defaults, unknown keys are ignored, existing config files parse unchanged, and `style = "off"` is representable.

- [ ] 1.2 Add footer color keys resolving through the existing color map with theme-token fallbacks.
  - touches: `crates/jcode-config-types/src/display.rs`, `crates/jcode-tui/src/tui/footer.rs (new)`, `docs/TUI_COLOR_CONFIGURATION.md`
  - depends on: 1.1
  - Done when every footer color key is documented, resolves from `display.colors` when set, and falls back to its theme token when unset.

## 2. Footer snapshot and rendering

- [ ] 2.1 Implement `FooterSnapshot` assembly from existing cached state.
  - touches: `crates/jcode-tui/src/tui/app/footer.rs (new)`, `crates/jcode-tui/src/tui/app.rs`
  - depends on: 1.1
  - Done when unit tests prove assembly is pure (no mutation, no subprocess, no filesystem access), covers `$HOME` collapse and path depth modes, local/remote/hybrid markers, stale context marking, and omission of git/cost/name segments when data is absent.

- [ ] 2.2 Implement segment rendering with fixed priority drop order and truncation.
  - touches: `crates/jcode-tui/src/tui/footer.rs (new)`
  - depends on: 2.1
  - Done when unit tests cover the documented drop order (session name, cost, tokens, effort, upstream extras, directory depth, git counts), smart truncation of branch/context labels, one-row guarantee at every tested width, and ASCII vs Nerd Font glyph selection.

## 3. Layout integration

- [ ] 3.1 Add the footer chunk to the chat-column vertical layout in both constraint sets.
  - touches: `crates/jcode-tui/src/tui/ui.rs`
  - depends on: 2.2
  - Done when the footer occupies the bottom row when enabled, reserves zero height when `off`, participates correctly in packed vs scrolling layout decisions, and overlay short-circuits and close-backs render correctly.

- [ ] 3.2 Wire footer rendering into the chrome pass with timing capture and no copy-selection rows.
  - touches: `crates/jcode-tui/src/tui/ui.rs`, `crates/jcode-tui/src/tui/app.rs`
  - depends on: 3.1
  - Done when the footer render is inside the chrome timing block, frame debug capture records the footer pass, and copy-selection tests prove the footer contributes no selectable rows.

## 4. Deterministic snapshots and gate evidence

- [ ] 4.1 Add deterministic footer snapshot suite.
  - touches: `crates/jcode-tui/src/tui/app/tests/footer.rs (new)`
  - depends on: 3.2
  - Done when snapshots cover widths 60/80/100/120/160, packed and scrolling layouts, repeated-render byte identity, ASCII icon mode, no-color mode, missing git/cost/context data, unnamed session, remote mode with remote directory, and post-resume state.

- [ ] 4.2 Extend the frame benchmark and record the rollback path.
  - touches: `crates/jcode-tui/src/tui/app/tests/smoothness_benchmark.rs`, `docs/STATUS_FOOTER.md (new)`
  - depends on: 4.1
  - Done when the benchmark covers footer-enabled streamed output within the recorded budget, the doc records validation commands, expected results, degradation behavior, the no-stall budget, and the `style = "off"` rollback snapshot diff against the pre-change baseline.

## 5. Documentation and validation

- [ ] 5.1 Add footer documentation.
  - touches: `docs/STATUS_FOOTER.md (new)`
  - depends on: 4.2
  - Done when the doc covers segment semantics, config schema with defaults, drop priority order, icon/color degradation, and the roadmap P2 gate evidence produced by this change.

- [ ] 5.2 Run strict OpenSpec validation and focused TUI test suites.
  - touches: `openspec/changes/add-status-footer/*`
  - depends on: 5.1
  - Done when `openspec validate add-status-footer --strict --no-interactive` passes and the focused `jcode-tui` test suite plus the new footer suite pass.
