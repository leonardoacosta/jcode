# Design

## Context

Transcript content is built by the prepared-line pipeline (`crates/jcode-tui-messages/src/prepared.rs`), which wraps messages into display rows and records, for each user prompt, the wrapped start/end row indices (`wrapped_user_prompt_starts/ends`) and the flattened prompt texts used by prompt preview. The TUI renders those rows in `crates/jcode-tui/src/tui/ui.rs`, applying the user background band, prompt number, and `›` glyph. The prepared cache is keyed on inputs that already include user colors (`hash_text_for_cache`), so adding a style key follows an established pattern.

Copy-selection over the transcript uses the shared copy-snapshot machinery; decoration rows such as prompt numbers are already excluded via per-row margins. Scroll anchoring, bottom anchoring during streaming, and prompt preview all operate on prepared row indices.

The Zentui reference (pi-zentui) styles user messages as framed, framed-copy-friendly, compact, or labeled. This change ports those styles onto jcode's prepared user rows.

## Goals / Non-Goals

**Goals:**

- User prompts read as framed regions: framed (borders + rail), framed-copy-friendly (borders, no rail), compact (rail only), labeled (rounded box + `User` label), or off (current band).
- Deterministic rendering: styles derive from prepared rows; identical state produces byte-identical output; style switching re-renders via the cache key.
- Copy safety: borders, rails, gutters, and labels are never copied; prompt-text selection is byte-identical to today.
- Documented degradation: narrow widths shrink frames without adding wraps; ASCII and no-color terminals get documented fallbacks.
- Zero streaming cost: user rows are static once sent; framing adds no per-frame work.

**Non-Goals:**

- Framing assistant messages, tool cards, or diffs.
- Per-message or per-agent custom labels.
- Zentui's native-delegation mode (jcode's `off` style covers it).
- Changes to prompt numbering, prompt preview, or scroll behavior beyond decoration rows.
- Animated or stateful frames (e.g., highlight-on-recent).

## Architecture

### 1. Frame derivation from prepared rows

For each user prompt, the prepared pipeline exposes `[start, end)` wrapped row indices. The framing layer (`user_message_frame.rs`) consumes those anchors plus the frame style and emits:

- `framed`: one top border row before `start`, rail glyph prepended to rows `start..end`, one bottom border row after `end - 1`.
- `framed-copy-friendly`: borders identical; rows get a one-cell gutter, no rail.
- `compact`: rail only, no border rows.
- `labeled`: rounded top border with an embedded `User` label, side rails, rounded bottom border.
- `off`: rows unchanged.

Border rows are inserted into the display row stream at the same layer that already maps prepared rows to screen rows, so scroll math sees them as ordinary content rows. The prompt number and `›` glyph render inside the frame on the first prompt row, unchanged.

**Why:** Deriving frames from prepared anchors keeps wrap correctness: frames always span exactly the prompt's wrapped rows at the current width, and re-wrap on resize automatically re-frames.

### 2. Cache and re-render determinism

The frame style joins the prepared-cache key inputs alongside the existing color inputs. Switching styles therefore invalidates exactly like a color change: prepared rows rebuild, frames re-derive, and identical inputs produce identical rows. User rows are static once sent, so this cost is bounded to style switches and resizes, never streaming frames.

**Why:** Piggybacking on the established cache-key pattern avoids a parallel invalidation mechanism and its bug class.

### 3. Copy-selection integration

Border rows register no selectable spans. Rail and gutter columns extend the existing per-row left margins for user prompt rows, exactly as the composer's prompt decoration is excluded today. The `User` label in the labeled style is part of the border row and is likewise unselectable.

**Why:** The existing margin machinery already solves decoration exclusion; framing adds decoration columns/rows to it rather than new rules.

### 4. Configuration schema

Additive `display.user_messages` section:

- `style`: `framed` (default) | `framed-copy-friendly` | `compact` | `labeled` | `off`.
- Color keys through the existing color map with theme fallbacks: frame border (`userMessageBorder`), rail (`userMessageRail`), label (`userMessageLabel`); the existing user band/text colors continue to apply inside the frame.

Unknown keys ignored per existing leniency. Independent of `display.footer` and `display.composer`; all three compose additively.

### 5. Degradation and capability handling

- **Narrow widths:** border rows render at the chat column width; rails and borders never cause prompt text to wrap beyond its own wrapped rows (rails replace one column of band padding, matching today's gutter).
- **ASCII mode:** borders draw with `-`, `|`, `+`; the label renders as plain `User` text in the top border.
- **No-color:** frames render unstyled; the user background band behavior is unchanged.
- **Very short prompts:** single-row prompts still get both border rows (framed/labeled), keeping the style visually consistent.

## Session isolation and security

Framing decorates the connected session's own transcript rows only. No content, credentials, or cross-session data is introduced. Frames carry no interactive targets.

## Validation strategy

- Unit tests: frame derivation per style from prepared anchors (single-row, multi-row, wrapped prompts), cache-key inclusion, gutter/rail margin registration.
- Deterministic frame snapshots: every style at widths 60/80/100/120/160, packed and scrolling layouts, multi-line prompts, ASCII and no-color, repeated-render byte identity.
- Copy-selection regression: decoration never copied; prompt-text selection byte-identical.
- Streaming regression: bottom anchoring and auto-scroll during streamed output unchanged with frames enabled.
- Style-switch test: changing the style re-renders deterministically through the cache.
- Rollback: `off` byte-identical to the pre-change baseline snapshot.

## Risks and mitigations

- **Height inflation with framed style.** Two extra rows per user prompt shortens visible transcript at small heights. Mitigation: `compact` documented as the zero-height option; snapshots pin layout at 60x15.
- **Interaction with pinned panes/overlays that also inject rows.** Frames derive from prepared anchors only, and pinned/overlay rows are separate streams; snapshot coverage includes a pinned-todo case to catch interference.
- **Cache invalidation misses a style switch.** The style is a cache-key input by construction, and the style-switch test guards it.
