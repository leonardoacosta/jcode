# Add Status Footer (Zentui Port, P2)

## Why

The roadmap handoff (`ROADMAP_HANDOFF.md`, block P2 — Basic Zentui port) requires porting the Zentui visual language into the jcode TUI: layout primitives, panels, status bars, command palette, model/provider/session selectors, tool cards, diff/log views, and keyboard navigation, with rendering state kept separate from agent state.

Most of that surface already exists (pickers, tool cards, diff views, info widgets, command suggestions). The most visible missing piece is a **persistent status footer**. Today jcode has only:

- a transient one-row activity status line (spinner, rate-limit countdown, connection phase), and
- floating info widgets in the transcript margins that hold model, context, git, and usage data but compete with content for space and disappear during idle/takeover states.

There is no stable, always-visible row that answers "where am I, what am I running, and what is it costing me". The Zentui reference (pi-zentui's Starship-inspired footer) establishes exactly this row: current directory, git branch/status, model/provider, context usage, tokens, and cost, with degraded behavior at narrow widths and without Nerd Fonts.

## What Changes

- Add a persistent one-row status footer as the bottom-most row of the chat column chrome, below the input/overscroll/idle rows.
- Left zone segments: working directory (basename by default, configurable depth), git branch with dirty/ahead/behind indicators, session name (off by default).
- Right zone segments: model label with provider, reasoning effort, context usage with warning/error thresholds, token counts, session cost.
- Execution-mode marker (local/remote/hybrid) shown adjacent to the directory, per the roadmap's explicit-mode principle.
- New config section `display.footer`: `style` (`segments` default, `off`), per-segment visibility toggles, `icon_mode` (`auto`/`ascii`), `path_display` (`basename`/`depth:N`/`full`).
- Segment colors resolve through the existing `display.colors` map and theme tokens, following `docs/TUI_COLOR_CONFIGURATION.md`.
- Width degradation: when the row does not fit, segments drop in a fixed priority order (cost, tokens, session name, directory depth, effort, upstream extras) until it fits on one row; the footer never wraps to two rows.
- Icon degradation: Nerd Font glyphs are used only when icon mode resolves to enabled; ASCII fallbacks otherwise. No-color terminals get unstyled text.
- The footer is pure render state: it is rebuilt per frame from existing cached snapshots (`info_widget_data()`, TTL-cached git facts, cost state) and never mutates agent state or spawns work.
- The footer row is chrome decoration: it is excluded from copy-selection snapshots and never shifts the transcript, input, or overlay layout beyond its own reserved row.

## Capabilities

### New Capabilities

- `status-footer`: A persistent, configurable status footer row rendering session-scoped directory, git, execution-mode, model/provider/effort, context, token, and cost segments with deterministic width and capability degradation.

### Modified Capabilities

None. The transient activity status line, queued row, notification row, info widgets, and idle animation keep their current behavior and layout ownership.

## Impact

- Adds one reserved row to the chat-column vertical layout in `crates/jcode-tui/src/tui/ui.rs` (new bottom chunk; both packed and scrolling constraint sets).
- Adds a footer module with segment assembly and rendering: `crates/jcode-tui/src/tui/footer.rs (new)`.
- Adds footer state mapping from existing snapshots: `crates/jcode-tui/src/tui/app/footer.rs (new)`.
- Extends `DisplayConfig` in `crates/jcode-config-types/src/display.rs` with a `footer` section (additive, defaults preserve compile and config compatibility).
- Adds color keys to the existing color configuration surface and documents them in `docs/TUI_COLOR_CONFIGURATION.md`.
- Adds deterministic rendering tests: `crates/jcode-tui/src/tui/app/tests/footer.rs (new)` covering widths 60/80/100/120/160, packed and scrolling layouts, ASCII icon mode, no-color mode, missing git/cost/context data, remote mode, and resume/reload.
- Extends the existing frame-time benchmark coverage so the footer pass is included in the no-stall gate.
- Does not change prompt composition, session profiles, tool policy, provider behavior, or any P0/P1 surface.

- touches: `crates/jcode-tui/src/tui/ui.rs`
- touches: `crates/jcode-tui/src/tui/footer.rs (new)`
- touches: `crates/jcode-tui/src/tui/app.rs`
- touches: `crates/jcode-tui/src/tui/app/footer.rs (new)`
- touches: `crates/jcode-tui/src/tui/app/tests/footer.rs (new)`
- touches: `crates/jcode-tui/src/tui/app/tests/smoothness_benchmark.rs`
- touches: `crates/jcode-config-types/src/display.rs`
- touches: `docs/TUI_COLOR_CONFIGURATION.md`
- touches: `docs/STATUS_FOOTER.md (new)`

## Preconditions

- Existing transient status line, queued, notification, overscroll, and idle-animation rows keep their current layout positions and behavior.
- Existing info widget placement and content are unchanged; the footer consumes the same snapshot data read-only.
- Existing copy-selection behavior over transcript and input regions is unchanged; the footer is never copyable.
- Existing color configuration (`display.theme`, `display.colors`) keeps resolving current keys; unknown keys remain ignored.
- Git facts come only from the existing TTL-cached `gather_git_info()` path; the footer must not introduce per-frame subprocess or filesystem probing.

## Decisions

- **Bottom-most reserved row, not a margin overlay.** The footer owns a fixed layout chunk instead of floating in the transcript margins like info widgets. This gives deterministic placement at every width, zero overlap risk with floating widgets, and matches the Starship-footer reference.
- **Default on (`segments`), config opt-out (`off`).** The point of P2 is to adopt the Zentui visual language. Rollback is a config edit, not a code revert.
- **Fixed segment set in v1.** Fully custom format-template strings (Zentui's `format` templates) are deferred; v1 offers per-segment visibility toggles only. This keeps config validation and snapshot determinism simple and leaves template parity as a compatible later extension.
- **No toolchain/runtime detection segment.** Starship's runtime modules detect project toolchains (Node, Rust, ...); jcode is an agent client, not a shell prompt, and that probing cost and scope do not belong in this change.
- **Read-only render state.** The footer assembles from `info_widget_data()`, the TTL-cached git snapshot, and cost state already maintained for other chrome. It introduces no new polling, no new state mutation, and no cross-session reads, preserving session-isolation invariants.
- **One row, never two.** Narrow terminals drop segments by priority instead of wrapping, keeping layout math and resize behavior deterministic.
- **No new slash command or menu in v1.** Configuration is via `config.toml` only; a `/footer` convenience toggle is an explicit non-goal for this change.
- **Telemetry unchanged.** Footer rendering time is folded into the existing per-frame chrome metrics; no new Herdr events or telemetry fields are added (P5 owns that contract).

## Done Means

- The footer renders deterministically in text-snapshot tests at widths 60/80/100/120/160 in both packed and scrolling layouts, with byte-identical output across repeated renders of identical state.
- Segment priority dropping produces the documented segment subsets at each tested width, verified by snapshots.
- ASCII icon mode and no-color mode produce their documented fallback renderings, verified by snapshots.
- Missing-data states (not a git repo, no reported cost, no context snapshot, unnamed session) render explicit, stable layouts without placeholders shifting the row.
- Remote sessions show the remote execution marker and the session's remote working directory; local sessions show the local marker; hybrid shows hybrid.
- The existing smoothness/frame benchmark shows no regression beyond the agreed budget with the footer enabled during streamed output (budget recorded in `docs/STATUS_FOOTER.md`).
- `openspec validate add-status-footer --strict --no-interactive` passes.
- Rollback is demonstrated: setting `display.footer.style = "off"` restores the pre-change layout exactly (verified by a snapshot diff against current behavior).

## Relationship to the roadmap

This is the first of three narrow changes closing roadmap block P2 (Basic Zentui port). The follow-ups, in dependency order, are the composer/editor frame (opencode-style accent rail with model/provider/thinking metadata) and transcript user-message framing (framed/compact/labeled styles). P2's promotion gate (deterministic snapshots, resize behavior, degraded-terminal behavior, no event-loop stalls during streamed output) is earned per change; this change earns the footer portion.
