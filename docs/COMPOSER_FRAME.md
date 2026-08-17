# Composer frame

OpenSpec change: `add-composer-frame` (roadmap block P2, second Zentui surface
after the status footer).

The composer frame adds two persistent decorations to the prompt composer:

- **Accent rail** — a one-column `│` gutter on every composer row (input and
  composer-owned hint rows), colored by composer mode.
- **Metadata row** — one reserved row at the composer's bottom edge showing
  `model · provider( · effort)`, right-aligned.

Both default ON. Setting `style = "flat"` removes the rail and the metadata
row and yields a composer that is byte-identical to the pre-change layout.

## Configuration

```toml
[display.composer]
# "rail" (default) draws the accent rail; "flat" disables the whole frame.
style = "rail"
# Reserve the bottom composer row for session metadata. Default true.
metadata = true
```

Unknown keys are ignored; absent sections keep defaults; existing configs
parse unchanged. Colors resolve through `[display.colors]` with theme-role
fallbacks (`composerRail`, `composerRailShell`, `composerRailQueued`,
`composerRailSkill`, `composerMetadata`) — see
`TUI_COLOR_CONFIGURATION.md` for the role table.

## Behavior contract

### Rail

- Renders on **every** composer row: the input row, composer-owned hint rows
  (shell hint, Ctrl+Enter hint), and the metadata row. Overlays that float
  above the composer (command suggestions) are not composer rows.
- Mode colors mirror `input_prompt` precedence: shell (`!`) > queued
  (processing) > skill > chat.
- ASCII mode (`icon_mode = "ascii"` on the footer config) renders `|`; the
  metadata separator becomes ` | ` instead of ` · `.
- The rail never participates in typed-text selection (issue #430): the copy
  snapshot registers the full composer row with per-row left margins that skip
  rail + prompt decoration, so hit-testing the rail clamps to the start of the
  typed text and the rail glyph is never copied.

### Metadata row

- Fixed format: `model · provider( · effort)`, right-aligned. The provider
  composes as `base/upstream` when an upstream provider exists.
- Segments omit cleanly: no effort → `model · provider`; no provider →
  `model`; model unavailable → a **stable empty row** (composer height never
  shifts mid-session). The row's free space may still host the pre-existing
  right fact stack, which composites over chrome rows by design.
- Degradation order at narrow widths: drop effort, then provider, then
  truncate the model with an ellipsis. One-row guarantee: the metadata never
  wraps to a second row.
- The row is reserved while processing (fact stack stands down during turns;
  the metadata row does not) so the composer height is constant across
  idle/streaming transitions.

### Layout

- The rail insets the typed-text area by exactly one column; wrap math, cursor
  positioning, click-to-caret mapping, and the send-mode indicator all operate
  on the inset width.
- `input_height` grows by one row for the metadata reservation in both packed
  and scrolling constraint sets.

## Rollback

`style = "flat"` removes rail inset and metadata reservation. The pre-existing
`ui_tests` golden suite doubles as the rollback proof: its `TestState` fixture
defaults to the flat style, so the whole suite renders the pre-frame composer
byte-for-byte. The frame test `flat_style_reserves_nothing` additionally
asserts explicit-flat equals the fixture default frame exactly.

## Evidence

Validation commands (from the repo root):

```sh
cargo test -p jcode-base --lib composer              # config schema (4 tests)
cargo test -p jcode-tui --lib composer_frame          # unit + frame suites
cargo test -p jcode-tui --lib copy_selection          # issue #430 selection
cargo test -p jcode-tui --lib smoothness              # streaming benchmarks
cargo test -p jcode-tui --lib -- --test-threads=2     # full suite vs baseline
openspec validate add-composer-frame --strict --no-interactive
```

- Unit tests (`tui/composer_frame.rs`): 12 — mode precedence, color key +
  fallback resolution, ASCII/no-color variants, metadata formatting, omission
  and degradation order, empty-row stability, one-row guarantee.
- Frame tests (`tui/ui_tests/composer_frame.rs`): 11 — rail on every composer
  row, per-mode rail colors through real frames, right-aligned metadata,
  widths 60/80/100/120/160, packed + scrolling repeated-render byte identity,
  ASCII variant, metadata off, processing persistence, hint-row rail,
  send-mode reservation coexistence, flat rollback equality.
- Copy-safety: the full issue #430 suite (`app/tests/input_copy_selection.rs`,
  19 tests) runs against the real App with the rail ON by default — typed-text
  selection stays byte-identical, rail/prompt/metadata never copied.
- Pre-existing App tests updated for the new default geometry: three
  click-to-caret tests click past the rail column; the prompt-preview test
  viewport grew one row for the metadata reservation; the overscroll flicker
  guard tolerates the elastic row plus one sticky-preview activation row (the
  byte-identical body-row comparison remains the anti-flicker guard); the
  mid-transcript smoothness benchmark viewport grew one row (29 → 30), same
  retune the footer's chrome row needed.
- Full `jcode-tui --lib --test-threads=2` suite: failure set matches the
  pre-existing baseline captured on clean master (handterm, auto-poke /
  confidence cluster, ui::messages prompt tests, order-dependent command
  snapshot tests, flaky clipboard helpers, flaky overscroll pad sweep). No new
  failures attributable to this change.
