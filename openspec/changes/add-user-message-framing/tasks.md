# Tasks

## 1. Configuration schema

- [ ] 1.1 Add `display.user_messages` config section with the frame style enum.
  - touches: `crates/jcode-config-types/src/display.rs`
  - depends on: none
  - Done when the section deserializes with `framed` default, all five styles are representable, unknown keys are ignored, and existing configs parse unchanged.

- [ ] 1.2 Add user-message frame color keys resolving through the existing color map.
  - touches: `crates/jcode-config-types/src/display.rs`, `docs/TUI_COLOR_CONFIGURATION.md`
  - depends on: 1.1
  - Done when border, rail, and label color keys are documented, resolve from `display.colors` when set, and fall back to theme tokens when unset.

## 2. Frame derivation and rendering

- [ ] 2.1 Implement frame derivation from prepared user prompt anchors for all five styles.
  - touches: `crates/jcode-tui/src/tui/user_message_frame.rs (new)`, `crates/jcode-tui-messages/src/prepared.rs`
  - depends on: 1.1
  - Done when unit tests cover framed, framed-copy-friendly, compact, labeled, and off derivation for single-row, multi-row, and wrapped prompts, including border row insertion positions and rail/gutter columns.

- [ ] 2.2 Render frames in the transcript user row path with capability fallbacks.
  - touches: `crates/jcode-tui/src/tui/ui.rs`, `crates/jcode-tui/src/tui/user_message_frame.rs (new)`
  - depends on: 2.1
  - Done when unit tests prove borders span the chat column width, prompt number and `›` render inside the frame unchanged, ASCII mode draws `-`/`|`/`+` glyphs and a plain `User` label, no-color mode renders unstyled frames, and the background band is preserved inside frames.

## 3. Cache, copy, and scroll integration

- [ ] 3.1 Include the frame style in prepared-cache key inputs.
  - touches: `crates/jcode-tui-messages/src/prepared.rs`, `crates/jcode-tui-messages/src/cache.rs`
  - depends on: 2.1
  - Done when a style switch deterministically re-renders user rows through the existing cache path and identical inputs produce identical rows.

- [ ] 3.2 Extend copy-selection margins to frame decoration.
  - touches: `crates/jcode-tui/src/tui/user_message_frame.rs (new)`, `crates/jcode-tui/src/tui/app/tests/user_message_frame.rs (new)`
  - depends on: 2.2
  - Done when copy-selection tests prove border rows, rail/gutter columns, and the `User` label contribute no copied text and prompt-text selection is byte-identical to the pre-change baseline.

- [ ] 3.3 Verify scroll anchoring with decoration rows.
  - touches: `crates/jcode-tui/src/tui/app/tests/user_message_frame.rs (new)`
  - depends on: 3.1
  - Done when scroll offset, bottom anchoring during streamed output, and prompt preview references behave identically to pre-change behavior with frames enabled.

## 4. Deterministic snapshots and gate evidence

- [ ] 4.1 Add deterministic user-message framing snapshot suite.
  - touches: `crates/jcode-tui/src/tui/app/tests/user_message_frame.rs (new)`
  - depends on: 3.3
  - Done when snapshots cover every style at widths 60/80/100/120/160, packed and scrolling layouts, multi-line prompts, single-row prompts, a pinned-todo transcript case, ASCII and no-color variants, and repeated-render byte identity.

- [ ] 4.2 Demonstrate rollback and document evidence.
  - touches: `docs/USER_MESSAGE_FRAMING.md (new)`
  - depends on: 4.1
  - Done when `off` produces byte-identical user rows to the pre-change baseline snapshot and the doc records style semantics, config schema, validation commands, degradation behavior, and the roadmap P2 gate evidence for this surface.

## 5. Validation

- [ ] 5.1 Run strict OpenSpec validation and focused TUI test suites.
  - touches: `openspec/changes/add-user-message-framing/*`
  - depends on: 4.2
  - Done when `openspec validate add-user-message-framing --strict --no-interactive` passes and the focused `jcode-tui` and `jcode-tui-messages` test suites plus the new framing suite pass.
