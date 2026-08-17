# User-message framing

OpenSpec change: `add-user-message-framing` (roadmap block P2, third and final
Zentui surface after the status footer and composer frame).

User prompts in the transcript are framed with box-drawn borders and an accent
rail so a prompt reads as a distinct visual block instead of blending into the
assistant stream. Five styles, default ON:

- **framed** (default) — full-width top/bottom border rows around each prompt
  plus a one-column accent rail `│` on every prompt row.
- **framed-copy-friendly** — borders and background band as in `framed`, but
  no rail glyphs: a one-cell leading gutter keeps decoration out of the way of
  selection.
- **compact** — accent rail only; no border rows, so transcript height is
  unchanged.
- **labeled** — rounded box (`╭─ User ─...─╮`) around each prompt with a fixed
  `User` label in the top border; the label drops when it cannot fit.
- **off** — pre-change rendering: flat numbered band, no rail, gutter, or
  borders. Byte-identical to the pre-change transcript.

Setting `style = "off"` restores the pre-change layout exactly.

## Configuration

```toml
[display.user_messages]
# framed (default) | framed-copy-friendly | compact | labeled | off
style = "framed"
```

Unknown keys are ignored (an unrecognized style falls back to `framed`);
absent sections keep defaults; existing configs parse unchanged. Colors
resolve through `[display.colors]` with theme-role fallbacks
(`userMessageBorder`, `userMessageRail`, `userMessageLabel`) — see
`TUI_COLOR_CONFIGURATION.md` for the role table.

## Behavior contract

### Where frames live

Frames are baked into `wrapped_lines` at wrap time (in `ui_prepare.rs`), not
drawn by a separate render pass. Border rows and rail/gutter spans are ordinary
prepared rows, so scroll math, bottom anchoring, packed and scrolling layouts,
copy snapshots, and streaming redisplay all treat them identically to content
rows with no schema change to `PreparedMessages`.

- The **top border** is emitted before the prompt's message-boundary anchor,
  so truncation at a message boundary never leaves a dangling border of a
  message that starts at the cut.
- **Rail/gutter** decoration replaces one column of the prompt's existing
  leading gutter (`leading_width == 1`), so prompt text is never rewrapped:
  content still wraps at the pre-change width, and the right edge stays
  aligned with other rows.
- ASCII mode (`icon_mode = "ascii"` on the footer config) renders `-`/`|`/`+`
  glyphs instead of `─`/`│`/box corners; the `labeled` style's label renders
  as plain `User` text.
- The pre-existing right-edge user bar (`│` at the row's last column) is
  unchanged; it is not part of the rail.

### Copy safety

Frame decoration never contributes copied text:

- Border rows register **zero-width copy maps**: the top border clamps to the
  prompt's first raw column, the bottom border to its last, so a drag that
  starts or ends on a border row selects the prompt text itself and can never
  copy a `─` glyph, corner, or the `User` label.
- Rail/gutter columns extend the existing per-row copy offsets, so
  hit-testing the rail clamps to the start of the prompt text, exactly like
  the prompt-number decoration always did.

## Rollback

`style = "off"` emits no borders, no rail, and no gutter. The pre-existing
`ui_tests` golden suite doubles as the rollback proof: its `TestState` fixture
defaults to the off style, so the whole suite renders the pre-frame transcript
byte-for-byte. The app-level copy test
`framed_prompt_selection_text_is_byte_identical_to_off` additionally asserts a
framed selection is byte-identical to an off selection of the same prompt.

## Evidence

Validation commands (from the repo root):

```sh
cargo test -p jcode-base --lib user_messages          # config schema (5 tests)
cargo test -p jcode-tui --lib user_message_frame      # unit + frame suites
cargo test -p jcode-tui --lib smoothness              # streaming benchmarks
cargo test -p jcode-tui --lib -- --test-threads=2     # full suite vs baseline
openspec validate add-user-message-framing --strict --no-interactive
```

- Unit tests (`tui/user_message_frame.rs`): 8 — color key + fallback
  resolution, border geometry for every style, labeled rounding and label
  drop, ASCII variants, rail/gutter/leading spans.
- Frame tests (`tui/ui_tests/user_message_frame.rs`): 11 — borders + rail
  structure, gutter (framed-copy-friendly), compact zero-height, labeled
  rounded box, off equals fixture default, multi-line prompt framing, all
  five styles at widths 60/80/100/120/160 repeated-render byte identity,
  ASCII, scrolling balance, no-rewrap guarantee, streaming stability.
- Copy-safety (`tui/app/tests/user_message_frame_copy.rs`): 4 — full-frame
  drags across border rows for framed and labeled styles copy only prompt
  text (never border/rail/label glyphs), compact rail stays clean, framed
  selection is byte-identical to off.
- Full `jcode-tui --lib --test-threads=2` suite: failure set matches the
  pre-existing baseline captured on clean master (handterm, auto-poke /
  confidence cluster, ui::messages prompt tests, order-dependent command
  snapshot tests, flaky clipboard helpers, flaky overscroll pad sweep). One
  pre-existing test fix included: `command_palette_open_does_not_move_existing_rows`
  now skips scrollbar-chrome rows that confused its exact-string row matcher.
- Frame-suite determinism hardening: the byte-identity and scrolling tests
  reset global render state between repeated renders (the flicker detector
  otherwise injects a notification row into the identical second render), and
  the scrolling balance assertion pairs only fully visible borders (a frame
  may straddle the viewport edge or have its prompt row replaced by the
  sticky preview row). Every frame test passes in its own process.

## Roadmap P2 closure

With this change landed, roadmap block **P2 (Basic Zentui port)** is complete:

1. Status footer — `70319c0` (`add-status-footer`)
2. Composer frame — `1796428` (`add-composer-frame`)
3. User-message framing — this change
