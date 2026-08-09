# Tasks

## 1. Configuration schema

- [ ] 1.1 Add `display.composer` config section with style and metadata toggle.
  - touches: `crates/jcode-config-types/src/display.rs`
  - depends on: none
  - Done when the section deserializes with documented defaults (`rail`, metadata true), unknown keys are ignored, existing configs parse unchanged, and `style = "flat"` is representable.

- [ ] 1.2 Add composer frame color keys resolving through the existing color map.
  - touches: `crates/jcode-config-types/src/display.rs`, `docs/TUI_COLOR_CONFIGURATION.md`
  - depends on: 1.1
  - Done when per-mode rail colors and metadata text color are documented, resolve from `display.colors` when set, and fall back to theme tokens when unset.

## 2. Rail and metadata rendering

- [ ] 2.1 Implement accent rail rendering with mode colors and capability fallbacks.
  - touches: `crates/jcode-tui/src/tui/ui_input.rs`, `crates/jcode-tui/src/tui/composer_frame.rs (new)`
  - depends on: 1.1
  - Done when unit tests prove the rail renders on every composer row (input and composer-owned hint rows), colors map from composer mode to the existing mode colors, ASCII mode renders `|`, and no-color mode renders unstyled.

- [ ] 2.2 Implement the composer metadata row with fixed format and degradation order.
  - touches: `crates/jcode-tui/src/tui/composer_frame.rs (new)`
  - depends on: 2.1
  - Done when unit tests cover `model · provider( · effort)` formatting, omission of effort/provider segments when absent, the stable empty row when the model label is unavailable, the documented drop order (effort, provider, then model truncation), and the one-row guarantee.

## 3. Layout integration

- [ ] 3.1 Integrate rail inset and metadata height into composer layout math.
  - touches: `crates/jcode-tui/src/tui/ui.rs`, `crates/jcode-tui/src/tui/ui_input.rs`
  - depends on: 2.2
  - Done when composer width insets exactly one column for the rail, `input_height` includes the metadata row in both packed and scrolling constraint sets, cursor positioning and wrap math are unchanged relative to the inset width, and the send-mode indicator keeps its reservation.

- [ ] 3.2 Extend copy-selection margins to the rail and exclude the metadata row.
  - touches: `crates/jcode-tui/src/tui/ui_input.rs`, `crates/jcode-tui/src/tui/app/tests/composer_frame.rs (new)`
  - depends on: 3.1
  - Done when copy-selection tests prove the rail column and metadata row contribute no copied text and typed-text selection remains byte-identical to the pre-change baseline.

## 4. Deterministic snapshots and gate evidence

- [ ] 4.1 Add deterministic composer frame snapshot suite.
  - touches: `crates/jcode-tui/src/tui/app/tests/composer_frame.rs (new)`
  - depends on: 3.2
  - Done when snapshots cover widths 60/80/100/120/160, packed and scrolling layouts, every composer mode, metadata on/off, ASCII and no-color variants, suggestions overlay active during streaming with no row shift, and repeated-render byte identity.

- [ ] 4.2 Demonstrate rollback and document evidence.
  - touches: `docs/COMPOSER_FRAME.md (new)`
  - depends on: 4.1
  - Done when `style = "flat"` produces a byte-identical composer to the pre-change baseline snapshot and the doc records validation commands, degradation behavior, and the roadmap P2 gate evidence for this surface.

## 5. Validation

- [ ] 5.1 Run strict OpenSpec validation and focused TUI test suites.
  - touches: `openspec/changes/add-composer-frame/*`
  - depends on: 4.2
  - Done when `openspec validate add-composer-frame --strict --no-interactive` passes and the focused `jcode-tui` test suite plus the new composer frame suite pass.
