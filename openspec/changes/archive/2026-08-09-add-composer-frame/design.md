# Design

## Context

The composer is drawn by `draw_input` in `crates/jcode-tui/src/tui/ui_input.rs` into chunk 7 of the chat column layout (`draw_inner` in `ui.rs`). Composer height (`input_height`) is computed from `wrapped_input_line_count` plus hint rows and send-mode reservation. Prompt decoration today is a per-row prefix: prompt number plus a mode-dependent glyph (`> `, `$ `, `… `, `» `) colored by composer mode (normal, shell, queued/processing, skill).

Metadata about the current model/provider/context is currently shown by the right fact stack (`draw_right_fact_stack`), which composites fact rows into free space near the input and explicitly stands down during overscroll, during processing, and when space is tight. Copy-selection over the composer is already solved: per-row left margins exclude prompt decoration from hit-testing and copied text (issue #430).

The Zentui reference (pi-zentui) styles the editor with an accent rail on every interior row and a metadata row inside the frame. This change ports that treatment to the jcode composer using existing mode colors and snapshot data.

## Goals / Non-Goals

**Goals:**

- A persistent accent rail framing every composer row, colored by composer mode.
- A guaranteed metadata row (`model · provider · effort`) that does not disappear during processing, overscroll, or queueing.
- Deterministic degradation: narrow widths shed metadata segments in a fixed order; ASCII/no-color terminals get documented fallbacks.
- Copy safety: the rail and metadata row are never copied; typed-text selection is byte-identical to today.
- Config-gated rollback: `display.composer.style = "flat"` restores the exact current composer.

**Non-Goals:**

- The Zentui "minimalist" rounded box frame (relocating session name, cost, git, timers into a box).
- Zentui's `metadataFormat` template engine; v1 uses one fixed format.
- Framing the queued-messages row, inline UI rows, or the command-suggestions overlay.
- Changes to editing behavior, key handling, cursor positioning, or prompt numbering.
- Interaction changes to the right fact stack's stand-down rules.

## Architecture

### 1. Rail rendering

`draw_input` renders into a sub-area inset by one column on the left; the freed column draws a rail glyph (`│` Nerd/capable terminals, `|` ASCII) on every composer row, including hint rows owned by the composer. Rail color maps from composer mode to the existing mode colors (`user_color()`, `shell_mode_color()`, `queued_color()`, `accent_color()`), so mode signaling becomes persistent instead of glyph-only.

**Why:** Reusing existing mode colors adds no new semantics, and insetting the draw area leaves wrap math, cursor math, and scroll logic operating on the same effective width minus one constant.

### 2. Metadata row

When `display.composer.metadata` is true, the composer reserves one additional bottom row showing `model · provider( · effort)`, right-aligned, muted style. Assembly is a pure mapping from the same per-frame data the right fact stack uses (`info_widget_data()`), so the two surfaces never disagree. Omission rules: no effort segment when thinking is off/none; no upstream/provider extras when absent; no row at all when the model label itself is unavailable (e.g., pre-auth states render an empty muted row to keep composer height stable).

Width degradation order: drop effort, then drop provider, then truncate the model label. The row never wraps.

**Why:** A layout-owned row survives exactly the states (processing, overscroll, queued, narrow) where the opportunistic fact stack stands down, which is the functional gap in current chrome.

### 3. Height and layout integration

`input_height` computation gains `metadata_height` (1 when shown, 0 otherwise), applied identically in both packed and scrolling constraint sets. Cursor positioning, `wrapped_input_line_count`, and suggestion-offset math operate on the text rows only; the metadata row is appended after them and never participates in text layout. The send-mode indicator keeps its reservation.

**Why:** Bounding the change to one additive row keeps the packed/scrolling decision, overscroll reveal, and inline-UI spacing math untouched.

### 4. Configuration schema

Additive `display.composer` section:

- `style`: `rail` (default) | `flat`.
- `metadata`: bool (default true).
- Color keys resolve through the existing color map with theme fallbacks: rail per mode (`composerRail`, `composerRailShell`, `composerRailQueued`, `composerRailSkill`), metadata text (`composerMetadata`).

Unknown keys ignored per existing config leniency. The section is independent of `display.footer`; the two changes compose additively.

### 5. Copy-selection integration

The rail column joins the existing left-margin registration: each composer row's left margin extends to include the rail, so hit-testing and copied text exclude it exactly as they exclude the prompt number and glyph today. The metadata row registers no selectable spans.

**Why:** The #430 machinery already solves decoration exclusion; the rail is one more decoration column.

## Session isolation and security

The metadata row shows only the connected session's own model/provider/effort snapshot. No credentials, keys, or prompt content are rendered. The frame adds no state and no cross-session reads.

## Validation strategy

- Unit tests: rail presence per mode and color mapping; metadata format and omission rules; degradation order; height math with metadata on/off.
- Deterministic frame snapshots: widths 60/80/100/120/160, packed and scrolling, metadata on/off, each composer mode, ASCII and no-color, suggestions overlay active during streaming (no row shift), repeated-render byte identity.
- Copy-selection regression: rail/metadata never copied; typed text byte-identical.
- Rollback: `style = "flat"` byte-identical to the pre-change baseline snapshot.

## Risks and mitigations

- **One-column width loss for input text.** At 60+ columns this is negligible; below that the composer already truncates. Snapshot-pinned at width 60 to catch surprises.
- **Visual duplication with the right fact stack and/or status footer.** Accepted: fact stack is opportunistic chrome and the footer is a separate row; each surface is independently disableable, and docs recommend disabling composer metadata if the footer is preferred, or vice versa.
- **Rail glyph confusion in shell mode.** Shell mode keeps its `$ ` prompt glyph; the rail only adds the persistent left frame, so mode signaling is additive, not changed.
