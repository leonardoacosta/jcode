# Add User Message Framing (Zentui Port, P2)

## Why

Roadmap block P2 (Basic Zentui port) requires porting the Zentui visual language. The third surface, after the status footer (`add-status-footer`) and composer frame (`add-composer-frame`), is **transcript user-message styling**. The Zentui reference (pi-zentui) offers framed, framed-copy-friendly, compact, and labeled user-message styles.

Today jcode renders user prompts as flat numbered rows (`8› prompt text`) on a full-width background band. The band is functional but minimal: no border separates one prompt from surrounding assistant output, so in dense transcripts (tool cards, diffs, streamed markdown) the user's own turns do not read as first-class framed regions the way Zentui transcripts do. The prepared-line pipeline already tracks exactly which wrapped rows belong to user prompts (`wrapped_user_prompt_starts/ends`), so the framing layer has precise anchors to draw from.

## What Changes

- Add user-message frame styles applied to the transcript's user prompt rows:
  - `framed` (default): full-width top/bottom border rows around each user prompt, with an accent rail on the left of every prompt row; existing `N›` numbering and background band are preserved inside the frame.
  - `framed-copy-friendly`: identical borders and background, no rail glyphs, a one-cell leading gutter before prompt text.
  - `compact`: no border rows; accent rail only on prompt rows (zero added height).
  - `labeled`: a rounded box around the prompt with a fixed `User` label in the top border.
  - `off`: today's exact rendering (flat numbered band, no rail or borders).
- New config section `display.user_messages`: `style` (default `framed`); border and rail colors resolve through the existing `display.colors` map with theme fallbacks.
- Border rows span the chat column width and truncate prompt text never; frames shrink with narrow widths but never add wrapping beyond the prompt's own wrapped rows.
- Framing decoration (borders, rails, gutters, label) is chrome: it is excluded from copy-selection and never becomes part of copied prompt text.
- The prepared-line cache keys include the frame style so switching styles re-renders deterministically; user rows remain static once sent (no per-frame cost during streaming).
- Capability degradation: ASCII mode draws borders as `-`/`|`-style glyphs and the label as plain text; no-color terminals render unstyled frames; the background band behavior is unchanged.

## Capabilities

### New Capabilities

- `user-message-framing`: Framed, copy-friendly, compact, and labeled frame styles for transcript user prompt rows, drawn from prepared-line anchors, with copy-safe decoration and deterministic width/capability degradation.

### Modified Capabilities

None. Prompt numbering, prompt-preview references, scroll anchoring, and assistant/tool rendering keep current behavior; only the decoration of user prompt rows changes.

## Impact

- Adjusts transcript row decoration in the user prompt row render path (`crates/jcode-tui/src/tui/ui.rs` and the prepared-line consumer) and adds framing logic: `crates/jcode-tui/src/tui/user_message_frame.rs (new)`.
- Extends prepared-line metadata so border rows participate in wrap/scroll math: `crates/jcode-tui-messages/src/prepared.rs`.
- Extends `DisplayConfig` in `crates/jcode-config-types/src/display.rs` with a `user_messages` section (additive).
- Adds color keys (frame border, rail, label) documented in `docs/TUI_COLOR_CONFIGURATION.md`.
- Adds deterministic rendering tests: `crates/jcode-tui/src/tui/app/tests/user_message_frame.rs (new)` covering every style, multi-line prompts, widths, ASCII/no-color, copy exclusion, scroll anchoring, cache re-render on style switch, and `off` rollback.
- Does not change message content, markdown rendering, tool cards, diff views, or any agent state.

- touches: `crates/jcode-tui/src/tui/ui.rs`
- touches: `crates/jcode-tui/src/tui/user_message_frame.rs (new)`
- touches: `crates/jcode-tui-messages/src/prepared.rs`
- touches: `crates/jcode-tui/src/tui/app/tests/user_message_frame.rs (new)`
- touches: `crates/jcode-config-types/src/display.rs`
- touches: `docs/TUI_COLOR_CONFIGURATION.md`
- touches: `docs/USER_MESSAGE_FRAMING.md (new)`

## Preconditions

- Existing transcript behavior (prompt numbering, wrapping, scroll anchoring, bottom anchoring during streaming, prompt preview) is unchanged apart from the added decoration rows.
- Existing copy-selection over the transcript keeps selecting prompt text without decoration.
- The prepared-line cache remains correct across width changes; frame style joins the existing cache-key inputs (which already include user colors).
- `add-status-footer` and `add-composer-frame` are independent; all three compose additively because they own disjoint surfaces (bottom row, composer, transcript user rows).

## Decisions

- **`framed` on by default, `off` restores the current band.** Consistent with the other P2 changes: adopt the Zentui visual language by default, roll back by config.
- **Borders add height; `compact` is the zero-height option.** Framed and labeled add two rows per user prompt (top/bottom border). Users on small terminals can pick `compact` for the rail without extra rows. This trade-off is documented rather than hidden.
- **Prompt numbers stay inside the frame.** `N›` numbering is referenced by prompt preview and navigation; framing decorates around it rather than replacing it.
- **No per-message custom labels.** The `labeled` style uses the fixed label `User`, matching the Zentui reference; session/agent naming in labels is deferred.
- **Static decoration only.** No animation, no per-frame recomputation: frames are derived from prepared rows and cached with them, preserving the no-stall gate.
- **Assistant and tool rows stay unframed.** Zentui styles user messages only; framing assistant output would fight tool cards, diffs, and markdown for width and is out of scope.

## Done Means

- Every style renders per spec in snapshots: framed (borders + rail + numbering + band), framed-copy-friendly (borders, gutter, no rail), compact (rail only), labeled (rounded box + `User` label), off (byte-identical to baseline).
- Multi-line prompts keep rail alignment across wrapped rows, verified by snapshots.
- Deterministic snapshots at widths 60/80/100/120/160, packed and scrolling layouts, byte-identical across repeated renders.
- ASCII and no-color variants render documented fallbacks, verified by snapshots.
- Copy-selection tests prove borders, rails, gutters, and labels are never copied and prompt-text selection is unchanged.
- Scroll/bottom anchoring during streamed output is unchanged with frames enabled (regression test).
- Switching styles re-renders deterministically via the prepared-line cache key, verified by test.
- `openspec validate add-user-message-framing --strict --no-interactive` passes.

## Relationship to the roadmap

This is the third of three narrow changes closing roadmap block P2 (Basic Zentui port), after `add-status-footer` and `add-composer-frame`. With all three landed, the P2 gate evidence (deterministic snapshots, resize behavior, degraded-terminal behavior, no event-loop stalls during streamed output) is complete for the ported visual language, and remaining P2 surfaces (layout primitives, panels, selectors, tool cards, diff/log views, keyboard navigation) are already present and verified by their existing test suites.
