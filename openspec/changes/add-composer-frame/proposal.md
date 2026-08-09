# Add Composer Frame (Zentui Port, P2)

## Why

Roadmap block P2 (Basic Zentui port) requires porting the Zentui visual language. The second surface, after the status footer (`add-status-footer`), is the **composer/editor frame**. The Zentui reference (pi-zentui) styles the editor opencode-style: an accent rail on every composer row, colored per mode, with an in-frame metadata row showing model, provider, and thinking level.

Today jcode's composer is flat: a numbered prompt (`9>`), mode-dependent prompt glyphs (`$ `, `… `, `» `, `> `), occasional hint rows, and a right-side "fact stack" that shows provider/auth, model, directory, and context only when space permits and stands down during overscroll, processing, and at narrow widths. There is no persistent framed region that visually owns "this is where you type and what you are talking to", and at exactly the moments the composer is busiest (queued prompts, processing, small terminals) the model/provider facts disappear.

## What Changes

- Add an accent rail column on the left of every composer row (input rows and composer-owned hint rows), colored by the existing composer mode colors (normal, shell, queued/processing, skill).
- Add an optional composer metadata row at the bottom of the composer: `model · provider ( · effort)`, muted styling, right-aligned; omitted segments leave no separator artifacts.
- New config section `display.composer`: `style` (`rail` default, `flat`), `metadata` (default true), independent from `display.footer`.
- The rail is chrome decoration: it is excluded from copy-selection via the existing per-row left-margin machinery and never becomes part of the copied text.
- Width degradation: at narrow widths the metadata row drops first (effort segment, then provider, then model truncation); the rail always remains. The composer never grows beyond its reserved height.
- Capability degradation: ASCII mode renders the rail as `|`; no-color terminals render an unstyled rail.
- The composer frame is pure render state: it reads the same per-frame snapshots as today and adds no polling, mutation, or state.
- The command-suggestions overlay pass, inline UI rows, and queued-messages row keep their current rendering and layout ownership; the frame applies to the composer chunk only.

## Capabilities

### New Capabilities

- `composer-frame`: An opencode-style accent rail and optional metadata row framing the composer, with mode-colored rails, copy-safe decoration, and deterministic width/capability degradation.

### Modified Capabilities

None. The right fact stack, hint rows, send-mode indicator, command-suggestions overlay, and queued row keep current behavior. When the rail style is active, the right fact stack keeps its existing stand-down rules unchanged (it may duplicate the metadata row's facts at wide widths; both are glanceable chrome and either can be disabled independently).

## Impact

- Adjusts composer height math in `crates/jcode-tui/src/tui/ui.rs` (one additional row when metadata is shown) and horizontal layout in `draw_input` (one rail column).
- Adds rail and metadata rendering to `crates/jcode-tui/src/tui/ui_input.rs`; helpers may move to `crates/jcode-tui/src/tui/composer_frame.rs (new)`.
- Extends `DisplayConfig` in `crates/jcode-config-types/src/display.rs` with a `composer` section (additive).
- Adds color keys (rail per mode, metadata text) to the existing color configuration surface, documented in `docs/TUI_COLOR_CONFIGURATION.md`.
- Adds deterministic rendering tests: `crates/jcode-tui/src/tui/app/tests/composer_frame.rs (new)` covering modes, widths, ASCII/no-color, metadata on/off, suggestions overlay active, copy exclusion, and `flat` rollback.
- Does not change input editing, key handling, prompt numbering, queued-message behavior, or any agent state.

- touches: `crates/jcode-tui/src/tui/ui.rs`
- touches: `crates/jcode-tui/src/tui/ui_input.rs`
- touches: `crates/jcode-tui/src/tui/composer_frame.rs (new)`
- touches: `crates/jcode-tui/src/tui/app/tests/composer_frame.rs (new)`
- touches: `crates/jcode-config-types/src/display.rs`
- touches: `docs/TUI_COLOR_CONFIGURATION.md`
- touches: `docs/COMPOSER_FRAME.md (new)`

## Preconditions

- Existing composer behavior (editing, wrapping, cursor positioning, hint rows, send-mode indicator, prompt numbering, shell/queue/skill modes) is unchanged.
- Existing copy-selection over the composer (issue #430 machinery) keeps selecting only typed text, never decoration.
- The command-suggestions overlay continues to render as a later overlay pass without shifting composer rows.
- `add-status-footer` is independent; both changes combine without layout conflict because the footer owns a separate bottom chunk. Neither requires the other.

## Decisions

- **Rail style on by default, `flat` restores the current composer.** Consistent with the footer change: P2 adopts the Zentui visual language by default, and rollback is a config edit.
- **Metadata row separate from the right fact stack.** The fact stack is opportunistic (disappears during processing/overscroll/narrow widths); the metadata row is a guaranteed, layout-owned composer row. They coexist; each is independently disableable.
- **Fixed metadata format.** `$model · $provider( · $effort)` with a muted style; Zentui's full template engine is deferred as a later compatible extension, as with the footer.
- **Rail color follows composer mode, reusing existing mode colors.** No new mode semantics; the rail makes the existing mode signal persistent across every composer row.
- **No rounded-frame (minimalist) variant in v1.** The Zentui minimalist style relocates many facts into a box frame; the rail port achieves the visual-language goal with far less layout risk. A boxed variant can be proposed later if wanted.
- **One extra row maximum.** Metadata adds at most one row to the composer; the rail adds only a column. Total chrome height change is bounded and snapshot-pinned.

## Done Means

- The rail renders on every composer row in all composer modes, with mode-correct colors, verified by snapshots.
- The metadata row renders `model · provider · effort` with correct omissions (no effort when off, no provider extras when absent), verified by snapshots.
- Deterministic snapshots at widths 60/80/100/120/160, in packed and scrolling layouts, with metadata on and off, are byte-identical across repeated renders.
- ASCII and no-color variants render documented fallbacks, verified by snapshots.
- Copy-selection tests prove the rail and metadata row contribute no copied text and typed-text selection is unchanged.
- The suggestions overlay active during streaming shows no row shift relative to pre-change behavior (regression snapshot).
- `style = "flat"` produces a byte-identical composer to the pre-change baseline.
- `openspec validate add-composer-frame --strict --no-interactive` passes.

## Relationship to the roadmap

This is the second of three narrow changes closing roadmap block P2 (Basic Zentui port), after `add-status-footer` and before transcript user-message framing. Each change earns its portion of the P2 gate (deterministic snapshots, resize behavior, degraded-terminal behavior, no event-loop stalls) independently; the composer change adds no new per-frame work beyond static row rendering.
